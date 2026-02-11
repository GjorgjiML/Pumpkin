//! Event handlers: block break protection and selection wand.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use pumpkin::plugin::api::events::block::block_break::BlockBreakEvent;
use pumpkin::plugin::api::events::player::player_interact_event::PlayerInteractEvent;
use pumpkin::plugin::EventHandler;
use pumpkin_data::item::Item;
use pumpkin_util::permission::PermissionLvl;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::selection::SelectionStore;
use crate::store::RegionStore;

/// Shared handle to region and selection state (used by commands and handlers).
#[derive(Clone)]
pub struct WordGuardRef {
    pub regions: Arc<RwLock<RegionStore>>,
    pub selections: Arc<RwLock<SelectionStore>>,
}

pub async fn context_has_bypass(server: &Arc<pumpkin::server::Server>, uuid: Uuid) -> bool {
    let perm = server.permission_manager.read().await;
    let lvl = server
        .get_player_by_uuid(uuid)
        .map_or(PermissionLvl::Zero, |p| p.permission_lvl.load());
    perm.has_permission(&uuid, "wordguardpmc:bypass", lvl)
        .await
}

pub struct BlockBreakHandler {
    pub regions: Arc<RwLock<RegionStore>>,
}

impl EventHandler<BlockBreakEvent> for BlockBreakHandler {
    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<pumpkin::server::Server>,
        event: &'a mut BlockBreakEvent,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let Some(ref player) = event.player else {
                return;
            };
            let world = player.living_entity.entity.world.load();
            let dimension_id = world.dimension.id;
            let uuid = player.gameprofile.id;

            if context_has_bypass(server, uuid).await {
                return;
            }

            let regions = self.regions.read().await;
            let Some((_key, region)) = regions.get_region_at(dimension_id, &event.block_position)
            else {
                return;
            };
            if !region.flags.block_break {
                return;
            }
            if region.can_build(&uuid) {
                return;
            }
            event.cancelled = true;
        })
    }

    fn handle<'a>(
        &'a self,
        _server: &'a Arc<pumpkin::server::Server>,
        _event: &'a BlockBreakEvent,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }
}

pub struct WandInteractHandler {
    pub selections: Arc<RwLock<SelectionStore>>,
}

impl EventHandler<PlayerInteractEvent> for WandInteractHandler {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<pumpkin::server::Server>,
        event: &'a PlayerInteractEvent,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let Some(clicked_pos) = event.clicked_pos else {
                return;
            };
            let item_guard = event.item.lock().await;
            if item_guard.get_item() != &Item::STICK {
                return;
            }
            drop(item_guard);

            let dimension_id = event.player.living_entity.entity.world.load().dimension.id;
            let uuid = event.player.gameprofile.id;
            let is_first = event.action.is_left_click();

            let mut sel = self.selections.write().await;
            sel.set_pos(uuid, dimension_id, clicked_pos, is_first);
            drop(sel);

            let num = if is_first { "1" } else { "2" };
            let msg = format!(
                "Position {} set to ({}, {}, {})",
                num,
                clicked_pos.0.x,
                clicked_pos.0.y,
                clicked_pos.0.z,
            );
            let _ = event
                .player
                .send_system_message(&TextComponent::text(msg).color_named(NamedColor::Green))
                .await;
        })
    }

    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<pumpkin::server::Server>,
        _event: &'a mut PlayerInteractEvent,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }
}
