//! BungeeCord/Velocity "Connect" plugin message and /cityselector go command.

use super::PluginState;
use pumpkin::command::args::{Arg, ConsumedArgs};
use pumpkin::command::dispatcher::CommandError::{InvalidConsumption, InvalidRequirement};
use pumpkin::command::tree::builder::{argument, literal};
use pumpkin::command::tree::CommandTree;
use pumpkin::command::{CommandExecutor, CommandResult, CommandSender};
use pumpkin::entity::player::Player;
use pumpkin_protocol::java::client::play::{CPlayPluginMessage, CTransfer};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

const BUNGEE_CHANNELS: [&str; 2] = ["BungeeCord", "bungeecord:main"];
const PROXY_TRANSFER_SUFFIX: &str = "65.109.231.78.nip.io";
const PROXY_TRANSFER_PORT: i32 = 25565;
const ARG_SERVER: &str = "server";

/// BungeeCord plugin message format: for each string, 2-byte big-endian length then UTF-8 bytes.
fn write_utf(s: &str, out: &mut Vec<u8>) {
    let b = s.as_bytes();
    out.extend_from_slice(&(b.len() as u16).to_be_bytes());
    out.extend_from_slice(b);
}

fn bungee_connect_payload(server_name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    write_utf("Connect", &mut out);
    write_utf(server_name, &mut out);
    out
}

fn bungee_connect_other_payload(player_name: &str, server_name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    write_utf("ConnectOther", &mut out);
    write_utf(player_name, &mut out);
    write_utf(server_name, &mut out);
    out
}

pub async fn transfer_player(player: &Arc<Player>, server_id: &str) {
    player
        .send_system_message(&TextComponent::text(format!(
            "Transferring you to {}...",
            server_id
        )))
        .await;

    let payload = bungee_connect_payload(server_id);
    let connect_other_payload = bungee_connect_other_payload(&player.gameprofile.name, server_id);

    // Primary path: native Minecraft transfer packet to a forced-host on Velocity.
    let transfer_host = format!(
        "{}.{}",
        server_id.replace('_', "-"),
        PROXY_TRANSFER_SUFFIX
    );
    let transfer_packet = CTransfer::new(&transfer_host, VarInt(PROXY_TRANSFER_PORT));
    player.client.enqueue_packet(&transfer_packet).await;

    // Fallback path: BungeeCord plugin message channels.
    for channel in BUNGEE_CHANNELS {
        let packet = CPlayPluginMessage::new(channel, &payload);
        player.client.enqueue_packet(&packet).await;
        let packet_other = CPlayPluginMessage::new(channel, &connect_other_payload);
        player.client.enqueue_packet(&packet_other).await;
    }
}

/// Command: /cityselector <server> and /cityselector go <server>
struct ConnectExecutor {
    state: Arc<PluginState>,
}

impl CommandExecutor for ConnectExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a pumpkin::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(server_id)) = args.get(ARG_SERVER) else {
                return Err(InvalidConsumption(Some(
                    "Usage: /cityselector <server> or /cityselector go <server>".into(),
                )));
            };

            let player = match sender {
                CommandSender::Player(p) => p.clone(),
                CommandSender::Console | CommandSender::Rcon(_) | CommandSender::CommandBlock(_, _) => {
                    return Err(InvalidRequirement);
                }
            };

            let resolved_server = self
                .state
                .servers
                .get(*server_id)
                .cloned()
                .or_else(|| {
                    let requested_lower = server_id.to_lowercase();
                    self.state
                        .servers
                        .iter()
                        .find(|(display, _)| display.to_lowercase() == requested_lower)
                        .map(|(_, id)| id.clone())
                })
                .unwrap_or_else(|| (*server_id).to_string());

            transfer_player(&player, &resolved_server).await;

            Ok(1)
        })
    }
}

pub fn build_command_tree(state: Arc<PluginState>) -> CommandTree {
    use pumpkin::command::args::simple::SimpleArgConsumer;
    let direct_executor = ConnectExecutor {
        state: state.clone(),
    };
    let go_executor = ConnectExecutor { state };

    CommandTree::new(["cityselector"], "Connect to a city server via Velocity")
        .then(argument(ARG_SERVER, SimpleArgConsumer).execute(direct_executor))
        .then(
            literal("go").then(argument(ARG_SERVER, SimpleArgConsumer).execute(go_executor)),
        )
}
