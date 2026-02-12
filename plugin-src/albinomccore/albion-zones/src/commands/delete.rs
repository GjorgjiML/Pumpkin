//! `/zoneadmin delete <name>` — delete a zone by name (OP only).

use crate::state::PluginState;
use pumpkin::command::args::{Arg, ConsumedArgs};
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

const ARG_NAME: &str = "name";

pub struct ZoneDeleteExecutor(pub PluginState);

impl CommandExecutor for ZoneDeleteExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = match args.get(ARG_NAME) {
                Some(Arg::Simple(s)) => (*s).to_owned(),
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Usage: /zoneadmin delete <name>")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };

            if !self.0.zone_engine.zone_exists(&name) {
                sender
                    .send_message(
                        TextComponent::text(format!("Zone '{name}' not found."))
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            }

            // Delete from DB
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

            let result = self.0.block_on(async {
                sqlx::query("DELETE FROM albion_zone_regions WHERE name = $1")
                    .bind(&name)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())
            });

            match result {
                Ok(_) => {
                    self.0.zone_engine.remove_region(&name);

                    sender
                        .send_message(
                            TextComponent::text(format!("Zone '{name}' deleted."))
                                .color_named(NamedColor::Green),
                        )
                        .await;
                    log::info!("albion_zones: Deleted zone '{name}'");
                }
                Err(e) => {
                    sender
                        .send_message(
                            TextComponent::text(format!("Failed to delete zone: {e}"))
                                .color_named(NamedColor::Red),
                        )
                        .await;
                }
            }

            Ok(1)
        })
    }
}
