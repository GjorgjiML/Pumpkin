//! WordGuardPMC — WorldGuard-like region protection for [Pumpkin](https://pumpkinmc.org/).
//!
//! This crate is structured in a decentralized way:
//! - **[region](region)** — Region and flag types
//! - **[selection](selection)** — Wand selection state
//! - **[store](store)** — Region storage
//! - **[handlers](handlers)** — Block break and wand interact handlers
//! - **[commands](commands)** — /wg command tree and executors
//!
//! See [README](https://github.com/GjorgjiML/worldguardalbino) for usage and setup.

mod commands;
mod handlers;
mod region;
mod selection;
mod store;

use std::sync::Arc;

use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin::plugin::api::{Context, EventPriority};
use pumpkin::plugin::api::events::block::block_break::BlockBreakEvent;
use pumpkin::plugin::api::events::player::player_interact_event::PlayerInteractEvent;
use pumpkin_util::permission::PermissionLvl;
use tokio::sync::RwLock;

use handlers::{BlockBreakHandler, WandInteractHandler, WordGuardRef};
use store::RegionStore;

#[plugin_method]
async fn on_load(&mut self, server: Arc<Context>) -> Result<(), String> {
    server
        .register_permission(pumpkin_util::permission::Permission::new(
            "wordguardpmc:bypass",
            "Bypass all region protection",
            pumpkin_util::permission::PermissionDefault::Op(PermissionLvl::Four),
        ))
        .await
        .ok();
    server
        .register_permission(pumpkin_util::permission::Permission::new(
            "wordguardpmc:admin",
            "Create and manage regions",
            pumpkin_util::permission::PermissionDefault::Op(PermissionLvl::Two),
        ))
        .await
        .ok();

    let tree = commands::build_tree(WordGuardRef {
        regions: self.regions.clone(),
        selections: self.selections.clone(),
    });
    server.register_command(tree, "wordguardpmc:admin").await;

    let break_handler = Arc::new(BlockBreakHandler {
        regions: self.regions.clone(),
    });
    server
        .register_event::<BlockBreakEvent, _>(break_handler, EventPriority::Normal, true)
        .await;

    let interact_handler = Arc::new(WandInteractHandler {
        selections: self.selections.clone(),
    });
    server
        .register_event::<PlayerInteractEvent, _>(
            interact_handler,
            EventPriority::Normal,
            false,
        )
        .await;

    log::info!("wordguardpmc: Loaded (region protection, /wg commands)");
    Ok(())
}

#[plugin_method]
async fn on_unload(&mut self, _server: Arc<Context>) -> Result<(), String> {
    Ok(())
}

#[plugin_impl]
pub struct WordGuardPMCPlugin {
    regions: Arc<RwLock<RegionStore>>,
    selections: Arc<RwLock<selection::SelectionStore>>,
}

impl WordGuardPMCPlugin {
    pub fn new() -> Self {
        Self {
            regions: Arc::new(RwLock::new(RegionStore::new())),
            selections: Arc::new(RwLock::new(selection::SelectionStore::default())),
        }
    }
}

impl Default for WordGuardPMCPlugin {
    fn default() -> Self {
        Self::new()
    }
}
