//! AlbionMC Zones Plugin for Pumpkin
//!
//! Implements Green/Yellow/Red/Black risk zones with:
//! - Admin-defined rectangular regions (pos1/pos2 selection)
//! - Bossbar zone indicators (color-coded by risk)
//! - PvP toggle per zone
//! - Death rules: safe / partial loot / full loot + trash chance
//! - Newbie protection (time-gated access to dangerous zones)
//!
//! Connects to albion_core's database for player profiles and newbie checks.

#![allow(improper_ctypes_definitions)]

mod bossbar;
mod commands;
mod config;
mod death;
mod events;
mod events_death;
mod events_pvp;
mod newbie;
mod service;
mod state;
mod zone_engine;

use pumpkin::plugin::api::events::player::player_death::PlayerDeathEvent;
use pumpkin::plugin::api::events::player::player_attack::PlayerAttackEvent;
use pumpkin::plugin::api::events::player::player_join::PlayerJoinEvent;
use pumpkin::plugin::api::events::player::player_leave::PlayerLeaveEvent;
use pumpkin::plugin::api::events::player::player_move::PlayerMoveEvent;
use pumpkin::plugin::api::{Context, EventPriority};
use pumpkin::plugin::{Plugin, PluginMetadata};
use pumpkin_util::permission::PermissionLvl;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::{future::Future, pin::Pin};

use albion_db::{connect, run_migrations};
use config::{DeathRule, RiskLevel, ZonesConfig};
use state::{AdminSelection, PluginState};
use zone_engine::{ZoneEngine, ZoneRegion};

// ---------------------------------------------------------------------------
// Plugin exports required by Pumpkin native loader
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static PUMPKIN_API_VERSION: u32 = pumpkin::plugin::PLUGIN_API_VERSION;

#[unsafe(no_mangle)]
pub static METADATA: PluginMetadata<'static> = PluginMetadata {
    name: "albion_zones",
    version: env!("CARGO_PKG_VERSION"),
    authors: "AlbionMC",
    description: "AlbionMC zones: Green/Yellow/Red/Black risk zones with death rules",
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin() -> Box<dyn Plugin> {
    Box::new(AlbionZonesPlugin::new())
}

// ---------------------------------------------------------------------------
// Plugin struct
// ---------------------------------------------------------------------------

pub struct AlbionZonesPlugin {
    runtime: Arc<tokio::runtime::Runtime>,
    state: Option<PluginState>,
}

impl AlbionZonesPlugin {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("albion-zones-rt")
            .build()
            .expect("albion_zones: failed to create tokio runtime");

        Self {
            runtime: Arc::new(runtime),
            state: None,
        }
    }
}

/// Load all zone regions from the database.
async fn load_zones_from_db(pool: &sqlx::PgPool) -> Result<Vec<ZoneRegion>, String> {
    let rows = sqlx::query_as::<_, (
        String,  // name
        String,  // risk
        bool,    // pvp_enabled
        String,  // death_rule
        i16,     // partial_drop_percent
        f64, f64, f64,  // min_x, min_y, min_z
        f64, f64, f64,  // max_x, max_y, max_z
    )>(
        "SELECT name, risk, pvp_enabled, death_rule, partial_drop_percent, \
         min_x, min_y, min_z, max_x, max_y, max_z \
         FROM albion_zone_regions ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("load zones: {e}"))?;

    let mut regions = Vec::with_capacity(rows.len());
    for (name, risk_str, pvp, dr_str, pdp, x1, y1, z1, x2, y2, z2) in rows {
        let risk = RiskLevel::from_str_loose(&risk_str).unwrap_or(RiskLevel::Green);
        let death_rule = match dr_str.as_str() {
            "partial" => DeathRule::Partial,
            "full_loot" => DeathRule::FullLoot,
            _ => DeathRule::Safe,
        };
        regions.push(ZoneRegion::new(
            name,
            risk,
            pvp,
            death_rule,
            pdp as u8,
            x1, y1, z1,
            x2, y2, z2,
        ));
    }
    Ok(regions)
}

impl Plugin for AlbionZonesPlugin {
    fn on_load(
        &mut self,
        context: Arc<Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        // Load config
        let data_folder = context.get_data_folder();
        let config_path = data_folder.join("config.toml");
        let zones_config = match ZonesConfig::load(&config_path) {
            Ok(c) => c,
            Err(e) => {
                log::error!("albion_zones: Failed to load config: {e}");
                return Box::pin(async move { Err(e) });
            }
        };

        // Connect to DB
        let db_url = zones_config.database.url.clone();
        let pool = match self.runtime.block_on(connect(&db_url)) {
            Ok(p) => p,
            Err(e) => {
                log::error!("albion_zones: DB connect failed: {e}");
                return Box::pin(async move { Err(e) });
            }
        };

        // Run migrations
        let migration_sql = include_str!("../migrations/001_init_zones.sql");
        if let Err(e) = self.runtime.block_on(run_migrations(&pool, migration_sql)) {
            log::error!("albion_zones: Migration failed: {e}");
            return Box::pin(async move { Err(e) });
        }

        // Load existing zones from DB
        let regions = match self.runtime.block_on(load_zones_from_db(&pool)) {
            Ok(r) => r,
            Err(e) => {
                log::error!("albion_zones: Failed to load zones from DB: {e}");
                return Box::pin(async move { Err(e) });
            }
        };
        log::info!("albion_zones: Loaded {} zone regions from database", regions.len());

        // Build zone engine
        let engine = ZoneEngine::new(
            zones_config.wilderness,
            zones_config.death.trash_chance_percent,
            zones_config.newbie_protection.required_hours,
            zones_config.death.partial_drop_percent,
            regions,
        );

        let plugin_state = PluginState {
            runtime: Arc::clone(&self.runtime),
            db_pool: Arc::new(RwLock::new(Some(pool))),
            zone_engine: Arc::new(engine),
            player_zones: Arc::new(RwLock::new(HashMap::new())),
            admin_selection: Arc::new(RwLock::new(AdminSelection::default())),
        };
        self.state = Some(plugin_state.clone());

        Box::pin(async move {
            // Register permissions
            context
                .register_permission(pumpkin_util::permission::Permission::new(
                    "albion_zones:use",
                    "Basic zone commands (info, confirm)",
                    pumpkin_util::permission::PermissionDefault::Allow,
                ))
                .await
                .ok();

            context
                .register_permission(pumpkin_util::permission::Permission::new(
                    "albion_zones:admin",
                    "Zone admin commands (pos1, pos2, create, delete, list)",
                    pumpkin_util::permission::PermissionDefault::Op(PermissionLvl::Four),
                ))
                .await
                .ok();

            // Register player commands: /zone info, /zone confirm
            let player_tree = commands::build_player_tree(plugin_state.clone());
            context.register_command(player_tree, "albion_zones:use").await;

            // Register admin commands: /zoneadmin pos1/pos2/create/delete/list/admin
            let admin_tree = commands::build_admin_tree(plugin_state.clone());
            context.register_command(admin_tree, "albion_zones:admin").await;

            // Register event handlers
            let join_handler = Arc::new(events::ZoneJoinHandler {
                state: plugin_state.clone(),
            });
            context
                .register_event::<PlayerJoinEvent, _>(
                    join_handler,
                    EventPriority::Normal,
                    false,
                )
                .await;

            let leave_handler = Arc::new(events::ZoneLeaveHandler {
                state: plugin_state.clone(),
            });
            context
                .register_event::<PlayerLeaveEvent, _>(
                    leave_handler,
                    EventPriority::Normal,
                    false,
                )
                .await;

            let move_handler = Arc::new(events::ZoneMoveHandler {
                state: plugin_state.clone(),
            });
            context
                .register_event::<PlayerMoveEvent, _>(
                    move_handler,
                    EventPriority::Normal,
                    true,
                )
                .await;

            let death_handler = Arc::new(events_death::ZoneDeathHandler {
                state: plugin_state.clone(),
            });
            context
                .register_event::<PlayerDeathEvent, _>(
                    death_handler,
                    EventPriority::Normal,
                    true,
                )
                .await;

            let pvp_handler = Arc::new(events_pvp::ZonePvpHandler {
                state: plugin_state.clone(),
            });
            context
                .register_event::<PlayerAttackEvent, _>(
                    pvp_handler,
                    EventPriority::High,
                    true,
                )
                .await;

            // Register zone service for other plugins
            context
                .register_service(
                    "albion_zone_service",
                    Arc::new(service::ZoneService::new(&plugin_state)),
                )
                .await;

            log::info!("albion_zones: Loaded successfully");
            Ok(())
        })
    }

    fn on_unload(
        &mut self,
        _context: Arc<Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            if let Some(ref state) = self.state {
                if let Some(pool) = state.db_pool.write().unwrap().take() {
                    state.block_on(async { pool.close().await });
                    log::info!("albion_zones: Database connection closed");
                }
                if let Ok(mut map) = state.player_zones.write() {
                    map.clear();
                }
            }
            log::info!("albion_zones: Unloaded");
            Ok(())
        })
    }
}
