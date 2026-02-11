//! `/albion admin` — server observability dashboard.

use super::super::state::PluginState;
use pumpkin::command::args::ConsumedArgs;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin_util::text::{TextComponent, color::NamedColor};

pub struct AdminExecutor(pub PluginState);

impl CommandExecutor for AdminExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a pumpkin::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let tps = server.get_tps();
            let mspt = server.get_mspt();
            let player_count = server.get_all_players().len();

            let avg_db_ms = {
                let latencies = self.0.db_latency_ms.read().unwrap();
                if latencies.is_empty() {
                    0.0
                } else {
                    latencies.iter().sum::<f64>() / latencies.len() as f64
                }
            };
            let last_err = self.0.last_error.read().unwrap().clone();

            let msg = format!(
                "--- AlbionMC Observability ---\n\
                 TPS: {:.2} | MSPT: {:.2}ms\n\
                 Players: {}\n\
                 DB latency (avg): {:.2}ms\n\
                 Last error: {}",
                tps,
                mspt,
                player_count,
                avg_db_ms,
                last_err.as_deref().unwrap_or("none")
            );
            sender
                .send_message(TextComponent::text(msg).color_named(NamedColor::Aqua))
                .await;
            Ok(1)
        })
    }
}
