use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::java::server::play::SlotActionType;

use super::PlayerEvent;

/// Event fired when a player clicks inside an open inventory container.
///
/// This can be cancelled to prevent the default screen handler click processing.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerContainerClickEvent {
    /// The player who clicked.
    pub player: Arc<Player>,
    /// The current container sync id sent by the client.
    pub sync_id: i32,
    /// The raw clicked slot index.
    pub slot: i16,
    /// The click button value from the client packet.
    pub button: i8,
    /// The click action mode.
    pub mode: SlotActionType,
}

impl PlayerContainerClickEvent {
    #[must_use]
    pub fn new(
        player: Arc<Player>,
        sync_id: i32,
        slot: i16,
        button: i8,
        mode: SlotActionType,
    ) -> Self {
        Self {
            player,
            sync_id,
            slot,
            button,
            mode,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerContainerClickEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
