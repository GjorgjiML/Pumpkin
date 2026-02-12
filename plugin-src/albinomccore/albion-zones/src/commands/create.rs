//! `/zoneadmin create <name> <risk>` — create a zone from the current pos1/pos2 selection (OP only).

use crate::config::{DeathRule, RiskLevel};
use crate::state::PluginState;
use crate::zone_engine::ZoneRegion;
use pumpkin::command::args::{Arg, ConsumedArgs};
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

const ARG_NAME: &str = "name";
const ARG_RISK: &str = "risk";

pub struct ZoneCreateExecutor(pub PluginState);

impl CommandExecutor for ZoneCreateExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            // Parse name
            let name = match args.get(ARG_NAME) {
                Some(Arg::Simple(s)) => (*s).to_owned(),
                _ => {
                    sender
                        .send_message(
                            TextComponent::text(
                                "Usage: /zoneadmin create <name> <green|yellow|red|black>",
                            )
                            .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };

            // Parse risk level
            let risk_str = match args.get(ARG_RISK) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Specify risk: green, yellow, red, or black")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };

            let Some(risk) = RiskLevel::from_str_loose(risk_str) else {
                sender
                    .send_message(
                        TextComponent::text(format!(
                            "Unknown risk level '{risk_str}'. Use: green, yellow, red, black"
                        ))
                        .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            // Check selection
            let sel = self.0.get_selection();
            if !sel.is_complete() {
                sender
                    .send_message(
                        TextComponent::text(
                            "Set both positions first! /zoneadmin pos1 then /zoneadmin pos2",
                        )
                        .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }
            let p1 = sel.pos1.unwrap();
            let p2 = sel.pos2.unwrap();

            // Check duplicate name
            if self.0.zone_engine.zone_exists(&name) {
                sender
                    .send_message(
                        TextComponent::text(format!("Zone '{name}' already exists!"))
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }

            // Determine rules from risk level
            let (pvp_enabled, death_rule) = match risk {
                RiskLevel::Green => (false, DeathRule::Safe),
                RiskLevel::Yellow => (true, DeathRule::Partial),
                RiskLevel::Red | RiskLevel::Black => (true, DeathRule::FullLoot),
            };

            let partial_drop = self.0.zone_engine.default_partial_drop;

            // Build the region
            let region = ZoneRegion::new(
                name.clone(),
                risk,
                pvp_enabled,
                death_rule,
                partial_drop,
                p1.x, p1.y, p1.z,
                p2.x, p2.y, p2.z,
            );

            // Save to DB
            let pool = {
                let guard = self.0.db_pool.read().unwrap();
                guard.as_ref().cloned()
            };
            let Some(pool) = pool else {
                sender
                    .send_message(
                        TextComponent::text("Database not connected")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let creator_uuid = sender.as_player().map(|p| p.gameprofile.id);

            let result = self.0.block_on(async {
                sqlx::query(
                    "INSERT INTO albion_zone_regions \
                     (name, risk, pvp_enabled, death_rule, partial_drop_percent, \
                      min_x, min_y, min_z, max_x, max_y, max_z, created_by) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                )
                .bind(&region.name)
                .bind(format!("{}", region.risk).to_lowercase())
                .bind(region.pvp_enabled)
                .bind(match region.death_rule {
                    DeathRule::Safe => "safe",
                    DeathRule::Partial => "partial",
                    DeathRule::FullLoot => "full_loot",
                })
                .bind(i16::from(region.partial_drop_percent))
                .bind(region.min_x)
                .bind(region.min_y)
                .bind(region.min_z)
                .bind(region.max_x)
                .bind(region.max_y)
                .bind(region.max_z)
                .bind(creator_uuid)
                .execute(&pool)
                .await
                .map_err(|e| e.to_string())
            });

            match result {
                Ok(_) => {
                    // Add to live engine
                    self.0.zone_engine.add_region(region.clone());
                    // Clear selection
                    self.0.clear_selection();

                    let msg = format!(
                        "Zone '{}' created! [{}] ({:.0},{:.0},{:.0}) to ({:.0},{:.0},{:.0}) — PvP: {} — Death: {:?}",
                        name, risk,
                        region.min_x, region.min_y, region.min_z,
                        region.max_x, region.max_y, region.max_z,
                        if pvp_enabled { "ON" } else { "OFF" },
                        death_rule,
                    );
                    sender
                        .send_message(TextComponent::text(msg).color_named(NamedColor::Green))
                        .await;

                    log::info!("albion_zones: Created zone '{name}' [{risk}]");
                }
                Err(e) => {
                    sender
                        .send_message(
                            TextComponent::text(format!("Failed to save zone: {e}"))
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    log::error!("albion_zones: Failed to create zone '{name}': {e}");
                }
            }

            Ok(1)
        })
    }
}
