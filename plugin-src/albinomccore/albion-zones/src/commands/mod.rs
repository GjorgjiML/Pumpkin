//! Zone commands.
//!
//! Player commands (albion_zones:use):
//!   /zone info [target]   — show current zone info
//!   /zone confirm          — confirm dangerous zone entry
//!
//! Admin commands (albion_zones:admin) — OP only:
//!   /zone pos1             — set corner 1 at your position
//!   /zone pos2             — set corner 2 at your position
//!   /zone create <name> <risk>  — create zone from selection
//!   /zone delete <name>    — delete a zone
//!   /zone list             — list all defined zones

mod admin;
mod confirm;
mod create;
mod delete;
mod info;
mod list;
mod pos;

use crate::state::PluginState;
use pumpkin::command::args::players::PlayersArgumentConsumer;
use pumpkin::command::args::simple::SimpleArgConsumer;
use pumpkin::command::tree::builder::{argument, literal};
use pumpkin::command::tree::CommandTree;

const ARG_TARGET: &str = "target";
const ARG_NAME: &str = "name";
const ARG_RISK: &str = "risk";

/// Build the player-facing command tree (`/zone`).
pub fn build_player_tree(state: PluginState) -> CommandTree {
    CommandTree::new(["zone"], "AlbionMC zone commands")
        .then(
            literal("info")
                .execute(info::ZoneInfoExecutor(state.clone()))
                .then(
                    argument(ARG_TARGET, PlayersArgumentConsumer)
                        .execute(info::ZoneInfoExecutor(state.clone())),
                ),
        )
        .then(literal("confirm").execute(confirm::ZoneConfirmExecutor(state)))
}

/// Build the admin command tree (`/zoneadmin`).
pub fn build_admin_tree(state: PluginState) -> CommandTree {
    CommandTree::new(["zoneadmin"], "AlbionMC zone admin commands")
        .then(literal("pos1").execute(pos::Pos1Executor(state.clone())))
        .then(literal("pos2").execute(pos::Pos2Executor(state.clone())))
        .then(
            literal("create").then(
                argument(ARG_NAME, SimpleArgConsumer).then(
                    argument(ARG_RISK, SimpleArgConsumer)
                        .execute(create::ZoneCreateExecutor(state.clone())),
                ),
            ),
        )
        .then(
            literal("delete").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    .execute(delete::ZoneDeleteExecutor(state.clone())),
            ),
        )
        .then(literal("list").execute(list::ZoneListExecutor(state.clone())))
        .then(literal("admin").execute(admin::ZoneAdminExecutor(state)))
}
