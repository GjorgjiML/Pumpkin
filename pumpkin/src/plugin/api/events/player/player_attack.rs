use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// Fired when one player attempts to attack another player.
///
/// If cancelled, the attack is ignored and no damage is applied.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerAttackEvent {
    /// The attacking player.
    pub attacker: Arc<Player>,
    /// The player being attacked.
    pub victim: Arc<Player>,
}

impl PlayerAttackEvent {
    #[must_use]
    pub const fn new(attacker: Arc<Player>, victim: Arc<Player>) -> Self {
        Self {
            attacker,
            victim,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerAttackEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.attacker
    }
}
