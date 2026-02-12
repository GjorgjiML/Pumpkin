//! `/zoneadmin pos1` and `/zoneadmin pos2` — set selection corners (OP only).

use crate::state::PluginState;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

// ── pos1 ──

pub struct Pos1Executor(pub PluginState);

impl CommandExecutor for Pos1Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                sender
                    .send_message(
                        TextComponent::text("Only players can set positions.")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let pos = player.position();
            self.0.set_pos1(pos);

            sender
                .send_message(
                    TextComponent::text(format!(
                        "Pos1 set to ({:.1}, {:.1}, {:.1})",
                        pos.x, pos.y, pos.z
                    ))
                    .color_named(NamedColor::Green),
                )
                .await;

            // Show selection status
            let sel = self.0.get_selection();
            if sel.is_complete() {
                sender
                    .send_message(
                        TextComponent::text(
                            "Selection complete! Use /zoneadmin create <name> <green|yellow|red|black>",
                        )
                        .color_named(NamedColor::Gold),
                    )
                    .await;
            } else {
                sender
                    .send_message(
                        TextComponent::text("Now set pos2 with /zoneadmin pos2")
                            .color_named(NamedColor::Yellow),
                    )
                    .await;
            }

            Ok(1)
        })
    }
}

// ── pos2 ──

pub struct Pos2Executor(pub PluginState);

impl CommandExecutor for Pos2Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                sender
                    .send_message(
                        TextComponent::text("Only players can set positions.")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let pos = player.position();
            self.0.set_pos2(pos);

            sender
                .send_message(
                    TextComponent::text(format!(
                        "Pos2 set to ({:.1}, {:.1}, {:.1})",
                        pos.x, pos.y, pos.z
                    ))
                    .color_named(NamedColor::Green),
                )
                .await;

            let sel = self.0.get_selection();
            if sel.is_complete() {
                sender
                    .send_message(
                        TextComponent::text(
                            "Selection complete! Use /zoneadmin create <name> <green|yellow|red|black>",
                        )
                        .color_named(NamedColor::Gold),
                    )
                    .await;
            } else {
                sender
                    .send_message(
                        TextComponent::text("Now set pos1 with /zoneadmin pos1")
                            .color_named(NamedColor::Yellow),
                    )
                    .await;
            }

            Ok(1)
        })
    }
}
