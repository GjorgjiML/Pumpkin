use pumpkin_macros::Event;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// Fired when a player has died, before inventory is dropped.
///
/// Plugins can set `drop_percent` and `trash_percent` to control zone-based
/// death loot (e.g. partial loot in yellow zones, full loot + trash in red/black).
#[derive(Event, Clone)]
pub struct PlayerDeathEvent {
    /// The player who died.
    pub player: Arc<Player>,

    /// Death position (used to look up zone rules).
    pub position: Vector3<f64>,

    /// Percentage of main inventory slots to drop (0 = keep all, 100 = drop all).
    /// Default from server: 0 if keepInventory gamerule, else 100.
    pub drop_percent: u8,

    /// Of the dropped items, percentage to destroy instead of dropping (0–100).
    /// Used for full-loot zones with "trash chance".
    pub trash_percent: u8,
}

impl PlayerEvent for PlayerDeathEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
