//! Force Save Plugin for Pumpkin
//!
//! Registers `/save` to force-save all world and player data to disk without stopping the server.

#![allow(improper_ctypes_definitions)]

use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::dispatcher::CommandError;
use pumpkin::command::tree::CommandTree;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin::plugin::{Plugin, PluginMetadata};
use pumpkin_util::permission::PermissionLvl;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::future::Future;
use std::pin::Pin;

// ---------------------------------------------------------------------------
// Plugin exports — required by Pumpkin's native loader
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static PUMPKIN_API_VERSION: u32 = pumpkin::plugin::PLUGIN_API_VERSION;

#[unsafe(no_mangle)]
pub static METADATA: PluginMetadata<'static> = PluginMetadata {
    name: "force_save",
    version: env!("CARGO_PKG_VERSION"),
    authors: "Pumpkin",
    description: "Force-save world and player data to disk (/save)",
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin() -> Box<dyn Plugin> {
    Box::new(ForceSavePlugin)
}

// ---------------------------------------------------------------------------
// Command executor
// ---------------------------------------------------------------------------

struct SaveExecutor;

impl CommandExecutor for SaveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            // Only operators (and console/RCON) can run /save
            if !sender.has_permission_lvl(PermissionLvl::One) {
                return Err(CommandError::CommandFailed(TextComponent::text(
                    "Only operators can run this command.",
                )));
            }
            sender
                .send_message(
                    TextComponent::text("Saving world and player data...").color_named(NamedColor::Yellow),
                )
                .await;
            server.force_save_all().await;
            sender
                .send_message(
                    TextComponent::text("Save completed.").color_named(NamedColor::Green),
                )
                .await;
            Ok(1)
        })
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

struct ForceSavePlugin;

impl Plugin for ForceSavePlugin {
    fn on_load(
        &mut self,
        context: std::sync::Arc<pumpkin::plugin::api::Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            context
                .register_permission(pumpkin_util::permission::Permission::new(
                    "force_save:save",
                    "Run /save to force-save world data (ops only)",
                    pumpkin_util::permission::PermissionDefault::Op(PermissionLvl::One),
                ))
                .await
                .ok();

            let tree = CommandTree::new(["save"], "Force-save world and player data to disk")
                .execute(SaveExecutor);
            context.register_command(tree, "force_save:save").await;

            log::info!("force_save: Loaded (/save)");
            Ok(())
        })
    }

    fn on_unload(
        &mut self,
        _context: std::sync::Arc<pumpkin::plugin::api::Context>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
}
