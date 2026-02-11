//! `/albion profile [target]` — view player silver and fame.

use super::super::state::PluginState;
use pumpkin::command::args::{Arg, ConsumedArgs, FindArg};
use pumpkin::command::args::players::PlayersArgumentConsumer;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use std::time::Instant;

const ARG_TARGET: &str = "target";

pub struct ProfileExecutor(pub PluginState);

impl CommandExecutor for ProfileExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
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
                                TextComponent::text("Usage: /albion profile [target]")
                                    .color_named(NamedColor::Red),
                            )
                            .await;
                        return Ok(0);
                    }
                }
            };

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

            for player in players {
                let uuid = player.gameprofile.id;
                let start = Instant::now();

                let result = self.0.block_on(async {
                    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
                    sqlx::query_as::<_, (i64, i64)>(
                        "SELECT silver, fame FROM albion_profiles WHERE uuid = $1",
                    )
                    .bind(uuid)
                    .fetch_optional(&mut *conn)
                    .await
                    .map_err(|e| e.to_string())
                });

                self.0.record_latency(start.elapsed().as_secs_f64() * 1000.0);

                match result {
                    Ok(Some((silver, fame))) => {
                        let msg = format!(
                            "{}: Silver: {}, Fame: {}",
                            player.gameprofile.name, silver, fame
                        );
                        sender
                            .send_message(
                                TextComponent::text(msg).color_named(NamedColor::Green),
                            )
                            .await;
                    }
                    Ok(None) => {
                        sender
                            .send_message(
                                TextComponent::text(format!(
                                    "{}: No profile found",
                                    player.gameprofile.name
                                ))
                                .color_named(NamedColor::Yellow),
                            )
                            .await;
                    }
                    Err(e) => {
                        self.0.record_error(&e);
                        sender
                            .send_message(
                                TextComponent::text(format!("Error: {e}"))
                                    .color_named(NamedColor::Red),
                            )
                            .await;
                    }
                }
            }
            let _ = server;
            Ok(1)
        })
    }
}
