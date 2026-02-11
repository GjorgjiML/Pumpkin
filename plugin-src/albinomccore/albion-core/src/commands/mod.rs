//! Albion admin commands.

mod admin;
mod give;
mod profile;

use super::state::PluginState;
use pumpkin::command::args::players::PlayersArgumentConsumer;
use pumpkin::command::tree::builder::{argument, literal};
use pumpkin::command::tree::CommandTree;
use pumpkin::command::args::bounded_num::BoundedNumArgumentConsumer;
use pumpkin::command::args::simple::SimpleArgConsumer;

const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";
const ARG_TYPE: &str = "type";

pub fn build_tree(state: PluginState) -> CommandTree {
    CommandTree::new(["albion"], "AlbionMC admin commands")
        .then(
            literal("profile")
                .execute(profile::ProfileExecutor(state.clone()))
                .then(
                    argument(ARG_TARGET, PlayersArgumentConsumer)
                        .execute(profile::ProfileExecutor(state.clone())),
                ),
        )
        .then(
            literal("give").then(
                argument(ARG_TARGET, PlayersArgumentConsumer).then(
                    argument(ARG_TYPE, SimpleArgConsumer).then(
                        argument(
                            ARG_AMOUNT,
                            BoundedNumArgumentConsumer::<i64>::new().min(0).name(ARG_AMOUNT),
                        )
                        .execute(give::GiveExecutor(state.clone())),
                    ),
                ),
            ),
        )
        .then(literal("admin").execute(admin::AdminExecutor(state)))
}
