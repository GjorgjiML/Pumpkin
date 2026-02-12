//! City Selector – Lobby compass menu to travel to city servers via Velocity/BungeeCord.
//!
//! - Compass in inventory opens an inventory GUI menu.
//! - Cities: Ratsku, Vatsku, Ratuskuu, AJkaz (configurable server names).
//! - Uses BungeeCord plugin message "Connect" for proxy transfer.

#![allow(improper_ctypes_definitions)]

mod config;
mod transfer;

use config::CitySelectorConfig;
use pumpkin::plugin::api::events::player::player_container_click::PlayerContainerClickEvent;
use pumpkin::plugin::api::events::player::player_interact_event::PlayerInteractEvent;
use pumpkin::plugin::api::events::player::player_join::PlayerJoinEvent;
use pumpkin::plugin::api::{Context, EventPriority};
use pumpkin::plugin::{Cancellable, Plugin, PluginMetadata};
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::CustomNameImpl;
use pumpkin_util::permission::PermissionLvl;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{Clearable, Inventory, split_stack};
use pumpkin_world::item::ItemStack;
use std::any::Any;
use pumpkin_util::text::color::NamedColor;
use std::collections::HashMap;
use std::sync::Arc;
use std::{future::Future, pin::Pin};
use tokio::sync::Mutex;
use uuid::Uuid;

const COMPASS_ITEM_ID: u16 = 1034; // pumpkin_data Item::COMPASS.id
const MENU_SERVER_SLOTS: [usize; 4] = [10, 12, 14, 16];
const MENU_CITY_NAMES: [&str; 4] = ["Ratsku", "Vatsku", "Ratuskuu", "AJkaz"];

#[unsafe(no_mangle)]
pub static PUMPKIN_API_VERSION: u32 = pumpkin::plugin::PLUGIN_API_VERSION;

#[unsafe(no_mangle)]
pub static METADATA: PluginMetadata<'static> = PluginMetadata {
    name: "city_selector",
    version: env!("CARGO_PKG_VERSION"),
    authors: "AlbinoMC",
    description: "Compass menu to travel from lobby to city servers (Ratsku, Vatsku, Ratuskuu, AJkaz) via Velocity",
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin() -> Box<dyn Plugin> {
    Box::new(CitySelectorPlugin::new())
}

pub struct CitySelectorPlugin {
    runtime: Arc<tokio::runtime::Runtime>,
    state: Option<PluginState>,
}

#[derive(Clone)]
struct PluginState {
    config: CitySelectorConfig,
    /// Server display name -> Velocity server id
    servers: HashMap<String, String>,
    menu_entries: Vec<(String, String)>,
    open_menus: Arc<Mutex<HashMap<Uuid, OpenMenuSession>>>,
}

#[derive(Clone)]
struct OpenMenuSession {
    sync_id: i32,
    slot_to_server: HashMap<i16, String>,
}

struct CityMenuScreenFactory {
    inventory: Arc<dyn Inventory>,
}

impl ScreenHandlerFactory for CityMenuScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<pumpkin_inventory::player::player_inventory::PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_generic_9x3(sync_id, player_inventory, self.inventory.clone()).await;
            let handler: SharedScreenHandler = Arc::new(Mutex::new(handler));
            Some(handler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::text("City Selector")
    }
}

struct CityMenuInventory {
    slots: Vec<Arc<Mutex<ItemStack>>>,
}

impl CityMenuInventory {
    fn new(entries_count: usize) -> Self {
        let mut slots: Vec<Arc<Mutex<ItemStack>>> = (0..27)
            .map(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
            .collect();

        let icons = [
            &pumpkin_data::item::Item::COMPASS,
            &pumpkin_data::item::Item::MAP,
            &pumpkin_data::item::Item::BOOK,
            &pumpkin_data::item::Item::CLOCK,
        ];

        for idx in 0..entries_count.min(MENU_SERVER_SLOTS.len()) {
            let slot = MENU_SERVER_SLOTS[idx];
            // Create item with custom name showing city name
            let named_stack = ItemStack::new_with_component(
                1,
                icons[idx],
                vec![(
                    DataComponent::CustomName,
                    Some(Box::new(CustomNameImpl {
                        name: MENU_CITY_NAMES[idx],
                    })),
                )],
            );
            slots[slot] = Arc::new(Mutex::new(named_stack));
        }

        Self { slots }
    }
}

impl Clearable for CityMenuInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for stack in &self.slots {
                *stack.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

impl Inventory for CityMenuInventory {
    fn size(&self) -> usize {
        self.slots.len()
    }

    fn is_empty(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move {
            for stack in &self.slots {
                if !stack.lock().await.is_empty() {
                    return false;
                }
            }
            true
        })
    }

    fn get_stack(&self, slot: usize) -> Pin<Box<dyn Future<Output = Arc<Mutex<ItemStack>>> + Send + '_>> {
        Box::pin(async move {
            self.slots
                .get(slot)
                .cloned()
                .unwrap_or_else(|| Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
        })
    }

    fn remove_stack(&self, slot: usize) -> Pin<Box<dyn Future<Output = ItemStack> + Send + '_>> {
        Box::pin(async move {
            if let Some(stack) = self.slots.get(slot) {
                let mut guard = stack.lock().await;
                let taken = guard.clone();
                *guard = ItemStack::EMPTY.clone();
                taken
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }

    fn remove_stack_specific(
        &self,
        slot: usize,
        amount: u8,
    ) -> Pin<Box<dyn Future<Output = ItemStack> + Send + '_>> {
        Box::pin(async move {
            if slot >= self.slots.len() {
                return ItemStack::EMPTY.clone();
            }
            split_stack(&self.slots, slot, amount).await
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if let Some(item) = self.slots.get(slot) {
                *item.lock().await = stack;
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl CitySelectorPlugin {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .thread_name("city-selector-rt")
            .build()
            .expect("city_selector: failed to create tokio runtime");

        Self {
            runtime: Arc::new(runtime),
            state: None,
        }
    }
}

impl Plugin for CitySelectorPlugin {
    fn on_load(
        &mut self,
        context: Arc<Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        let data_folder = context.get_data_folder();
        let config_path = data_folder.join("config.toml");
        let config = match CitySelectorConfig::load(&config_path) {
            Ok(c) => c,
            Err(e) => {
                log::error!("city_selector: Failed to load config: {e}");
                return Box::pin(async move { Err(e) });
            }
        };

        let servers: HashMap<String, String> = [
            ("Ratsku".into(), config.servers.ratsku.clone()),
            ("Vatsku".into(), config.servers.vatsku.clone()),
            ("Ratuskuu".into(), config.servers.ratuskuu.clone()),
            ("AJkaz".into(), config.servers.ajkaz.clone()),
        ]
        .into_iter()
        .collect();

        let plugin_state = PluginState {
            config: config.clone(),
            servers,
            menu_entries: vec![
                ("Ratsku".into(), config.servers.ratsku.clone()),
                ("Vatsku".into(), config.servers.vatsku.clone()),
                ("Ratuskuu".into(), config.servers.ratuskuu.clone()),
                ("AJkaz".into(), config.servers.ajkaz.clone()),
            ],
            open_menus: Arc::new(Mutex::new(HashMap::new())),
        };
        self.state = Some(plugin_state.clone());

        Box::pin(async move {
            context
                .register_permission(pumpkin_util::permission::Permission::new(
                    "city_selector:use",
                    "Use compass menu and /cityselector go",
                    pumpkin_util::permission::PermissionDefault::Allow,
                ))
                .await
                .ok();

            context
                .register_permission(pumpkin_util::permission::Permission::new(
                    "city_selector:admin",
                    "Reload config",
                    pumpkin_util::permission::PermissionDefault::Op(PermissionLvl::Four),
                ))
                .await
                .ok();

            let tree = transfer::build_command_tree(Arc::new(plugin_state.clone()));
            context.register_command(tree, "city_selector:use").await;

            let join_handler = Arc::new(JoinHandler {
                state: plugin_state.clone(),
            });
            context
                .register_event::<PlayerJoinEvent, _>(join_handler, EventPriority::Normal, false)
                .await;

            let interact_handler = Arc::new(InteractHandler {
                state: plugin_state.clone(),
            });
            context
                .register_event::<PlayerInteractEvent, _>(
                    interact_handler,
                    EventPriority::Normal,
                    true,
                )
                .await;

            let container_click_handler = Arc::new(ContainerClickHandler {
                state: plugin_state.clone(),
            });
            context
                .register_event::<PlayerContainerClickEvent, _>(
                    container_click_handler,
                    EventPriority::Normal,
                    true,
                )
                .await;

            log::info!("city_selector: Loaded (Ratsku, Vatsku, Ratuskuu, AJkaz)");
            Ok(())
        })
    }

    fn on_unload(
        &mut self,
        _context: Arc<Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            log::info!("city_selector: Unloaded");
            Ok(())
        })
    }
}

struct JoinHandler {
    state: PluginState,
}

impl pumpkin::plugin::EventHandler<PlayerJoinEvent> for JoinHandler {
    fn handle<'a>(
        &'a self,
        server: &'a Arc<pumpkin::server::Server>,
        event: &'a PlayerJoinEvent,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if !self.state.config.lobby.give_compass_on_join {
                return;
            }
            let player = &event.player;
            let inventory = player.inventory();
            // Lobby behavior: clean inventory and give exactly one compass.
            inventory.clear().await;
            let compass = pumpkin_world::item::ItemStack::new(1, &pumpkin_data::item::Item::COMPASS);
            inventory.set_stack(0, compass).await;
            player
                .send_system_message(
                    &TextComponent::text("Use your Compass to open the City Selector menu.")
                        .color_named(NamedColor::Green),
                )
                .await;
            let _ = server;
        })
    }
}

struct InteractHandler {
    state: PluginState,
}

impl pumpkin::plugin::EventHandler<PlayerInteractEvent> for InteractHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<pumpkin::server::Server>,
        event: &'a mut PlayerInteractEvent,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if !event.action.is_right_click() {
                return;
            }
            let item_guard = event.item.lock().await;
            if item_guard.is_empty() || item_guard.item.id != COMPASS_ITEM_ID {
                return;
            }
            drop(item_guard);

            event.set_cancelled(true);
            let player = &event.player;

            let mut slot_to_server = HashMap::new();
            for (idx, (_, server_id)) in self.state.menu_entries.iter().enumerate() {
                if idx >= MENU_SERVER_SLOTS.len() {
                    break;
                }
                slot_to_server.insert(MENU_SERVER_SLOTS[idx] as i16, server_id.clone());
            }

            let menu_inventory: Arc<dyn Inventory> =
                Arc::new(CityMenuInventory::new(self.state.menu_entries.len()));
            let screen_factory = CityMenuScreenFactory {
                inventory: menu_inventory,
            };

            if let Some(sync_id) = player.open_handled_screen(&screen_factory).await {
                self.state.open_menus.lock().await.insert(
                    player.gameprofile.id,
                    OpenMenuSession {
                        sync_id: i32::from(sync_id),
                        slot_to_server,
                    },
                );
                player
                    .send_system_message(
                        &TextComponent::text("City menu opened. Click a city to travel.")
                            .color_named(NamedColor::Green),
                    )
                    .await;
            }
        })
    }
}

struct ContainerClickHandler {
    state: PluginState,
}

impl pumpkin::plugin::EventHandler<PlayerContainerClickEvent> for ContainerClickHandler {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<pumpkin::server::Server>,
        event: &'a mut PlayerContainerClickEvent,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let player_id = event.player.gameprofile.id;
            let mut open_menus = self.state.open_menus.lock().await;
            let Some(session) = open_menus.get(&player_id).cloned() else {
                return;
            };

            if session.sync_id != event.sync_id {
                return;
            }

            event.set_cancelled(true);

            let Some(server_id) = session.slot_to_server.get(&event.slot).cloned() else {
                return;
            };

            open_menus.remove(&player_id);
            drop(open_menus);

            event.player.close_handled_screen().await;
            transfer::transfer_player(&event.player, &server_id).await;
        })
    }
}
