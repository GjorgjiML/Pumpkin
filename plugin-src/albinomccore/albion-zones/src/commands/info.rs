//! `/zone info [target]` — show current zone information for a player.

use crate::bossbar;
use crate::death;
use crate::state::PluginState;
use pumpkin::command::args::players::PlayersArgumentConsumer;
use pumpkin::command::args::{Arg, ConsumedArgs, FindArg};
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

const ARG_TARGET: &str = "target";

pub struct ZoneInfoExecutor(pub PluginState);

impl CommandExecutor for ZoneInfoExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let players: Vec<_> = match PlayersArgumentConsumer::find_arg(args, ARG_TARGET) {
                Ok(p) => p.to_vec(),
                _ => {
                    if let Some(Arg::Players(p)) = args.get(ARG_TARGET) {
                        p.to_vec()
                    } else if let Some(p) = sender.as_player() {
                        vec![p]
                    } else {
                        sender
                            .send_message(
                                TextComponent::text("Usage: /zone info [target]")
                                    .color_named(NamedColor::Red),
                            )
                            .await;
                        return Ok(0);
                    }
                }
            };

            let engine = &self.0.zone_engine;

            for player in players {
                let pos = player.position();
                let lookup = engine.zone_at(pos.x, pos.y, pos.z);
                let loot = death::compute_death_loot(engine, pos.x, pos.y, pos.z);

                let pvp_str = if lookup.pvp_enabled { "ON" } else { "OFF" };
                let death_str = match loot.rule {
                    crate::config::DeathRule::Safe => "Safe (no item loss)".to_owned(),
                    crate::config::DeathRule::Partial => {
                        format!("Partial ({}% inventory drop)", loot.drop_percent)
                    }
                    crate::config::DeathRule::FullLoot => {
                        format!(
                            "Full Loot (100% drop, {}% trashed)",
                            loot.trash_percent
                        )
                    }
                };

                let msg = format!(
                    "--- Zone Info ---\n\
                     Player: {}\n\
                     Zone: {}\n\
                     Risk: {} — {}\n\
                     PvP: {pvp_str}\n\
                     Death: {death_str}\n\
                     Position: ({:.0}, {:.0}, {:.0})",
                    player.gameprofile.name,
                    lookup.name,
                    lookup.risk,
                    lookup.risk.label(),
                    pos.x,
                    pos.y,
                    pos.z,
                );

                sender
                    .send_message(
                        TextComponent::text(msg)
                            .color_named(bossbar::risk_to_text_color(lookup.risk)),
                    )
                    .await;
            }
            Ok(1)
        })
    }
}
