//! `/zoneadmin admin` — admin overview of zone system (OP only).

use crate::state::PluginState;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

pub struct ZoneAdminExecutor(pub PluginState);

impl CommandExecutor for ZoneAdminExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let engine = &self.0.zone_engine;
            let tracked = self
                .0
                .player_zones
                .read()
                .map_or(0, |m| m.len());
            let online = server.get_all_players().len();
            let zone_count = engine.zone_count();

            let mut msg = String::from("--- AlbionMC Zones Admin ---\n");
            msg.push_str(&format!("Defined zones: {zone_count}\n"));
            msg.push_str(&format!("Tracked players: {tracked}/{online}\n"));
            msg.push_str(&format!(
                "Newbie protection: {}h required\n",
                engine.newbie_required_hours
            ));
            msg.push_str(&format!(
                "Trash chance: {}%\n",
                engine.trash_chance_percent
            ));
            msg.push_str(&format!(
                "Wilderness: {} — PvP: {}\n\n",
                engine.wilderness.risk,
                if engine.wilderness.pvp_enabled { "ON" } else { "OFF" },
            ));

            // Show selection
            let sel = self.0.get_selection();
            match (&sel.pos1, &sel.pos2) {
                (Some(p1), Some(p2)) => {
                    msg.push_str(&format!(
                        "Selection: ({:.0},{:.0},{:.0}) to ({:.0},{:.0},{:.0}) [READY]\n\n",
                        p1.x, p1.y, p1.z, p2.x, p2.y, p2.z,
                    ));
                }
                (Some(p1), None) => {
                    msg.push_str(&format!(
                        "Selection: pos1=({:.0},{:.0},{:.0}), pos2=NOT SET\n\n",
                        p1.x, p1.y, p1.z,
                    ));
                }
                (None, Some(p2)) => {
                    msg.push_str(&format!(
                        "Selection: pos1=NOT SET, pos2=({:.0},{:.0},{:.0})\n\n",
                        p2.x, p2.y, p2.z,
                    ));
                }
                (None, None) => {
                    msg.push_str("Selection: NONE\n\n");
                }
            }

            // List zones
            for region in &engine.all_regions() {
                msg.push_str(&format!(
                    "  {} [{}] ({:.0},{:.0},{:.0})-({:.0},{:.0},{:.0}) PvP:{} Death:{:?}\n",
                    region.name,
                    region.risk,
                    region.min_x, region.min_y, region.min_z,
                    region.max_x, region.max_y, region.max_z,
                    if region.pvp_enabled { "ON" } else { "OFF" },
                    region.death_rule,
                ));
            }

            // Player zone counts
            if let Ok(zones) = self.0.player_zones.read() {
                if !zones.is_empty() {
                    msg.push_str("\nPlayers per zone:\n");
                    let mut counts = std::collections::HashMap::new();
                    for pz in zones.values() {
                        *counts.entry(pz.current_zone_name.clone()).or_insert(0_u32) += 1;
                    }
                    for (name, count) in &counts {
                        msg.push_str(&format!("  {name}: {count}\n"));
                    }
                }
            }

            sender
                .send_message(TextComponent::text(msg).color_named(NamedColor::Aqua))
                .await;
            Ok(1)
        })
    }
}
