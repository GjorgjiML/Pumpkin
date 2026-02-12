use pumpkin_data::packet::clientbound::PLAY_CUSTOM_PAYLOAD;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::ser::network_serialize_no_prefix;

/// Client-bound plugin message in the play state (e.g. for BungeeCord/Velocity transfer).
#[derive(Serialize)]
#[java_packet(PLAY_CUSTOM_PAYLOAD)]
pub struct CPlayPluginMessage<'a> {
    pub channel: &'a str,
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub data: Box<[u8]>,
}

impl<'a> CPlayPluginMessage<'a> {
    #[must_use]
    pub fn new(channel: &'a str, data: &[u8]) -> Self {
        Self {
            channel,
            data: data.to_vec().into_boxed_slice(),
        }
    }
}
