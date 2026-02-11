//! `/albion give <target> <silver|fame> <amount>` — grant resources.

use super::super::state::PluginState;
use pumpkin::command::args::{Arg, ConsumedArgs, FindArg};
use pumpkin::command::args::players::PlayersArgumentConsumer;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::{TextComponent, color::NamedColor};

const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";
const ARG_TYPE: &str = "type";

pub struct GiveExecutor(pub PluginState);

impl CommandExecutor for GiveExecutor {
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
                    sender
                        .send_message(
                            TextComponent::text(
                                "Usage: /albion give <target> <silver|fame> <amount>",
                            )
                            .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };

            let type_str = match args.get(ARG_TYPE) {
                Some(Arg::Simple(s)) => *s,
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Specify type: silver or fame")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };

            let amount: i64 = match args.get(ARG_AMOUNT) {
                Some(Arg::Num(Ok(n))) => match n {
                    pumpkin::command::args::bounded_num::Number::I64(x) => *x,
                    pumpkin::command::args::bounded_num::Number::I32(x) => *x as i64,
                    _ => 0,
                },
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Invalid amount").color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
                }
            };

            let (col, col_name) = match type_str {
                "silver" => ("silver", "Silver"),
                "fame" => ("fame", "Fame"),
                _ => {
                    sender
                        .send_message(
                            TextComponent::text("Type must be 'silver' or 'fame'")
                                .color_named(NamedColor::Red),
                        )
                        .await;
                    return Ok(0);
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
                let sql = format!(
                    "INSERT INTO albion_profiles (uuid, silver, fame, mastery, flags) \
                     VALUES ($1, 0, 0, '{{}}', '{{}}') \
                     ON CONFLICT (uuid) DO UPDATE SET {col} = albion_profiles.{col} + $2, \
                     updated_at = NOW()"
                );

                let result = self.0.block_on(async {
                    let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
                    sqlx::query(&sql)
                        .bind(uuid)
                        .bind(amount)
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| e.to_string())
                });

                match result {
                    Ok(_) => {
                        sender
                            .send_message(
                                TextComponent::text(format!(
                                    "Gave {} {} to {}",
                                    amount, col_name, player.gameprofile.name
                                ))
                                .color_named(NamedColor::Green),
                            )
                            .await;
                    }
                    Err(e) => {
                        self.0.record_error(&e);
                        sender
                            .send_message(
                                TextComponent::text(format!(
                                    "Failed to give to {}: {e}",
                                    player.gameprofile.name
                                ))
                                .color_named(NamedColor::Red),
                            )
                            .await;
                    }
                }
            }
            Ok(1)
        })
    }
}
