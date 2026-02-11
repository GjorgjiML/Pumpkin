//! Command tree and executors for /wg (OP-only).

mod executors;

use pumpkin::command::tree::{CommandTree, builder::{argument, literal, require}};
use pumpkin::command::args::{simple::SimpleArgConsumer, players::PlayersArgumentConsumer};
use pumpkin_util::permission::PermissionLvl;

use crate::handlers::WordGuardRef;

pub const ARG_ID: &str = "id";
pub const ARG_PLAYER: &str = "player";
pub const ARG_FLAG: &str = "flag";
pub const ARG_VALUE: &str = "value";

pub fn build_tree(plugin: WordGuardRef) -> CommandTree {
    let op_only = require(|sender| sender.has_permission_lvl(PermissionLvl::One));
    CommandTree::new(["wg", "wordguard"], "WordGuardPMC region commands")
        .then(
            op_only
                .then(
                    literal("define")
                        .then(
                            argument(ARG_ID, SimpleArgConsumer)
                                .execute(executors::DefineExecutor(plugin.clone())),
                        ),
                )
                .then(
                    literal("remove")
                        .then(
                            argument(ARG_ID, SimpleArgConsumer)
                                .execute(executors::RemoveExecutor(plugin.clone())),
                        ),
                )
                .then(
                    literal("flag")
                        .then(
                            argument(ARG_ID, SimpleArgConsumer)
                                .then(
                                    argument(ARG_FLAG, SimpleArgConsumer)
                                        .then(
                                            argument(ARG_VALUE, SimpleArgConsumer)
                                                .execute(executors::FlagExecutor(plugin.clone())),
                                        ),
                                ),
                        ),
                )
                .then(
                    literal("addowner")
                        .then(
                            argument(ARG_ID, SimpleArgConsumer)
                                .then(
                                    argument(ARG_PLAYER, PlayersArgumentConsumer)
                                        .execute(executors::AddOwnerExecutor(plugin.clone())),
                                ),
                        ),
                )
                .then(
                    literal("addmember")
                        .then(
                            argument(ARG_ID, SimpleArgConsumer)
                                .then(
                                    argument(ARG_PLAYER, PlayersArgumentConsumer)
                                        .execute(executors::AddMemberExecutor(plugin.clone())),
                                ),
                        ),
                )
                .then(literal("list").execute(executors::ListExecutor(plugin.clone())))
                .then(literal("wand").execute(executors::WandExecutor(plugin))),
        )
}
