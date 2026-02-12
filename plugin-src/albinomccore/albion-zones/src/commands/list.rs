//! `/zoneadmin list` — list all defined zones (OP only).

use crate::state::PluginState;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

pub struct ZoneListExecutor(pub PluginState);

impl CommandExecutor for ZoneListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let regions = self.0.zone_engine.all_regions();

            if regions.is_empty() {
                sender
                    .send_message(
                        TextComponent::text("No zones defined. Use /zoneadmin pos1, pos2, create.")
                            .color_named(NamedColor::Yellow),
                    )
                    .await;
                return Ok(1);
            }

            let mut msg = format!("--- Zones ({}) ---\n", regions.len());
            for r in &regions {
                msg.push_str(&format!(
                    "  {} [{}] ({:.0},{:.0},{:.0})-({:.0},{:.0},{:.0}) PvP:{} Death:{:?}\n",
                    r.name,
                    r.risk,
                    r.min_x, r.min_y, r.min_z,
                    r.max_x, r.max_y, r.max_z,
                    if r.pvp_enabled { "ON" } else { "OFF" },
                    r.death_rule,
                ));
            }

            sender
                .send_message(TextComponent::text(msg).color_named(NamedColor::Aqua))
                .await;
            Ok(1)
        })
    }
}
