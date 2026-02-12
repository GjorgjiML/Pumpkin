//! `/zone confirm` — confirm entry into dangerous (Red/Black) zones.
//!
//! Players under newbie protection can run this to acknowledge the risk
//! and bypass the automatic movement block for the current session.

use crate::state::PluginState;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::TextComponent;

pub struct ZoneConfirmExecutor(pub PluginState);

impl CommandExecutor for ZoneConfirmExecutor {
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
                        TextComponent::text("Only players can run this command.")
                            .color_named(NamedColor::Red),
                    )
                    .await;
                return Ok(0);
            };

            let uuid = player.gameprofile.id;
            self.0.set_danger_confirmed(&uuid);

            sender
                .send_message(
                    TextComponent::text(
                        "WARNING: You have confirmed entry to dangerous zones!\n\
                         You WILL lose items on death in Red/Black zones.\n\
                         This confirmation lasts until you disconnect.",
                    )
                    .color_named(NamedColor::Red),
                )
                .await;

            sender
                .send_message(
                    TextComponent::text("Dangerous zone access granted for this session.")
                        .color_named(NamedColor::Gold),
                )
                .await;

            log::info!(
                "albion_zones: {} confirmed dangerous zone entry",
                player.gameprofile.name
            );

            Ok(1)
        })
    }
}
