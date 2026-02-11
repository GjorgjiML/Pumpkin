//! AlbionMC Core Plugin for Pumpkin
//!
//! Thin orchestration layer — delegates to albion-db, albion-types, and modular commands/events.

#![allow(improper_ctypes_definitions)]

mod commands;
mod events;
mod service;
mod state;

use pumpkin::plugin::api::events::player::player_join::PlayerJoinEvent;
use pumpkin::plugin::api::{Context, EventPriority};
use pumpkin::plugin::{Plugin, PluginMetadata};
use pumpkin_util::permission::PermissionLvl;
use std::sync::{Arc, RwLock};
use std::{future::Future, pin::Pin};

use albion_db::{connect, ensure_config, load_db_url, run_migrations};
use state::PluginState;

// ---------------------------------------------------------------------------
// Plugin exports — required by Pumpkin's native loader
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static PUMPKIN_API_VERSION: u32 = pumpkin::plugin::PLUGIN_API_VERSION;

#[unsafe(no_mangle)]
pub static METADATA: PluginMetadata<'static> = PluginMetadata {
    name: "albion_core",
    version: env!("CARGO_PKG_VERSION"),
    authors: "AlbionMC",
    description: "Core AlbionMC plugin: player profiles, Postgres persistence, admin commands",
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin() -> Box<dyn Plugin> {
    Box::new(AlbionCorePlugin::new())
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct AlbionCorePlugin {
    state: PluginState,
}

impl AlbionCorePlugin {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("albion-rt")
            .build()
            .expect("albion_core: failed to create tokio runtime");

        Self {
            state: PluginState {
                runtime: Arc::new(runtime),
                db_pool: Arc::new(RwLock::new(None)),
                db_latency_ms: Arc::new(RwLock::new(Vec::new())),
                last_error: Arc::new(RwLock::new(None)),
            },
        }
    }

    fn init_db(&self, context: &Context) -> Result<(), String> {
        let data_folder = context.get_data_folder();
        let config_path = data_folder.join("config.toml");

        let db_url = load_db_url(&config_path)?;
        ensure_config(&config_path, &db_url)?;

        let pool = self
            .state
            .block_on(connect(&db_url))?;

        let migration_sql = include_str!("../migrations/001_init_profiles.sql");
        self.state
            .block_on(run_migrations(&pool, migration_sql))?;

        *self.state.db_pool.write().unwrap() = Some(pool);
        log::info!("albion_core: PostgreSQL connected and migrations applied");
        Ok(())
    }
}

impl Plugin for AlbionCorePlugin {
    fn on_load(
        &mut self,
        context: Arc<Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        if let Err(e) = self.init_db(&context) {
            log::error!("albion_core: Failed to init DB: {e}");
            return Box::pin(async move { Err(e) });
        }

        Box::pin(async move {
            context
                .register_permission(pumpkin_util::permission::Permission::new(
                    "albion_core:admin",
                    "Admin commands for AlbionMC",
                    pumpkin_util::permission::PermissionDefault::Op(PermissionLvl::Four),
                ))
                .await
                .ok();

            let tree = commands::build_tree(self.state.clone());
            context.register_command(tree, "albion_core:admin").await;

            let join_handler = Arc::new(events::PlayerJoinHandler {
                state: self.state.clone(),
            });
            context
                .register_event::<PlayerJoinEvent, _>(
                    join_handler,
                    EventPriority::Normal,
                    false,
                )
                .await;

            context
                .register_service(
                    "albion_profile_service",
                    Arc::new(service::ProfileService::from(&self.state)),
                )
                .await;

            log::info!("albion_core: Loaded successfully");
            Ok(())
        })
    }

    fn on_unload(
        &mut self,
        _context: Arc<Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            if let Some(pool) = self.state.db_pool.write().unwrap().take() {
                self.state.block_on(async { pool.close().await });
                log::info!("albion_core: Database connection closed");
            }
            Ok(())
        })
    }
}
