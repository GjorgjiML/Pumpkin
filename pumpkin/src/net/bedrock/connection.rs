use std::{
    io::{Cursor, Error},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::UNIX_EPOCH,
};

use pumpkin_protocol::bedrock::{
    RakReliability,
    client::raknet::connection::{CConnectedPong, CConnectionRequestAccepted},
    server::raknet::connection::{SConnectedPing, SConnectionRequest, SNewIncomingConnection},
};
use pumpkin_protocol::{codec::u24, serial::PacketRead};

use crate::net::bedrock::BedrockClient;
use crate::server::Server;

/// Resolve the address to advertise to Bedrock clients. 0.0.0.0 is not routable; use 127.0.0.1 when bound to 0.0.0.0 so local play works.
pub(crate) fn bedrock_advertise_addr(server: &Server) -> SocketAddr {
    server
        .basic_config
        .bedrock_advertise_address
        .unwrap_or_else(|| {
            let bind = server.basic_config.bedrock_edition_address;
            if bind.ip().is_unspecified() {
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), bind.port())
            } else {
                bind
            }
        })
}

impl BedrockClient {
    pub fn is_connection_request(reader: &mut Cursor<&[u8]>) -> Result<SConnectionRequest, Error> {
        //Must be reliable and non split
        if u8::read(reader)? == 0x40 {
            u16::read_be(reader)?;
            //skip reliable seq
            u24::read(reader)?;
            SConnectionRequest::read(reader)
        } else {
            Err(Error::other(""))
        }
    }

    pub async fn handle_connection_request(&self, packet: SConnectionRequest, server: &Server) {
        let addr = bedrock_advertise_addr(server);
        let addrs = [addr; 10];
        self.send_framed_packet(
            &CConnectionRequestAccepted::new(
                self.address,
                0,
                addrs,
                packet.time,
                UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
            ),
            RakReliability::Unreliable,
        )
        .await;
    }

    pub const fn handle_new_incoming_connection(&self, _packet: &SNewIncomingConnection) {
        // self.connection_state.store(ConnectionState::Login);
    }

    pub async fn handle_connected_ping(&self, packet: SConnectedPing) {
        self.send_framed_packet(
            &CConnectedPong::new(
                packet.time,
                UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
            ),
            RakReliability::Unreliable,
        )
        .await;
        // TODO Make this cleaner and handle it only with the ClientPlatform
        // This would also help with potential deadlocks by preventing to lock the player
        //self.player.lock().await.clone().map(async |player| {
        //    player.wait_for_keep_alive.store(false, Ordering::Relaxed);
        //    println!("ping procedet");
        //});
    }
}
