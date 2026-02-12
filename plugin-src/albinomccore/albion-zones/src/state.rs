//! Shared plugin state for albion_zones.

use crate::config::RiskLevel;
use crate::zone_engine::ZoneEngine;
use pumpkin_util::math::vector3::Vector3;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Per-player tracking data (zone they are currently in, bossbar UUID, etc.).
#[derive(Debug, Clone)]
pub struct PlayerZoneState {
    /// Current zone name the player is in.
    pub current_zone_name: String,
    /// Current risk level the player is in.
    pub current_risk: RiskLevel,
    /// UUID of the bossbar sent to this player (so we can remove/update it).
    pub bossbar_uuid: Uuid,
    /// Whether the player has confirmed dangerous zone entry this session.
    pub danger_confirmed: bool,
}

/// Admin-only pos1/pos2 selection for defining zones.
/// Only one selection exists at a time (shared by whoever is setting it up).
#[derive(Debug, Clone, Default)]
pub struct AdminSelection {
    pub pos1: Option<Vector3<f64>>,
    pub pos2: Option<Vector3<f64>>,
}

impl AdminSelection {
    /// Returns true when both positions are set.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.pos1.is_some() && self.pos2.is_some()
    }
}

/// Shared state passed to event handlers and command executors.
#[derive(Clone)]
pub struct PluginState {
    pub runtime: Arc<tokio::runtime::Runtime>,
    pub db_pool: Arc<RwLock<Option<PgPool>>>,
    pub zone_engine: Arc<ZoneEngine>,
    /// Per-player zone tracking. Key = player UUID.
    pub player_zones: Arc<RwLock<HashMap<Uuid, PlayerZoneState>>>,
    /// Single admin selection for pos1/pos2 zone creation.
    pub admin_selection: Arc<RwLock<AdminSelection>>,
}

impl PluginState {
    /// Run async work on the plugin's tokio runtime.
    #[inline]
    pub fn block_on<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.runtime.block_on(f)
    }

    /// Get a clone of a player's zone state.
    #[must_use]
    pub fn get_player_zone(&self, player_uuid: &Uuid) -> Option<PlayerZoneState> {
        self.player_zones.read().ok()?.get(player_uuid).cloned()
    }

    /// Update a player's zone state.
    pub fn set_player_zone(&self, player_uuid: Uuid, state: PlayerZoneState) {
        if let Ok(mut map) = self.player_zones.write() {
            map.insert(player_uuid, state);
        }
    }

    /// Remove a player's tracking data (on leave).
    pub fn remove_player(&self, player_uuid: &Uuid) -> Option<PlayerZoneState> {
        self.player_zones.write().ok()?.remove(player_uuid)
    }

    /// Mark a player as having confirmed dangerous zone entry.
    pub fn set_danger_confirmed(&self, player_uuid: &Uuid) {
        if let Ok(mut map) = self.player_zones.write() {
            if let Some(ps) = map.get_mut(player_uuid) {
                ps.danger_confirmed = true;
            }
        }
    }

    // ── Admin selection helpers ──

    /// Set pos1 for the admin selection.
    pub fn set_pos1(&self, pos: Vector3<f64>) {
        if let Ok(mut sel) = self.admin_selection.write() {
            sel.pos1 = Some(pos);
        }
    }

    /// Set pos2 for the admin selection.
    pub fn set_pos2(&self, pos: Vector3<f64>) {
        if let Ok(mut sel) = self.admin_selection.write() {
            sel.pos2 = Some(pos);
        }
    }

    /// Get the current admin selection.
    #[must_use]
    pub fn get_selection(&self) -> AdminSelection {
        self.admin_selection
            .read()
            .map_or_else(|_| AdminSelection::default(), |s| s.clone())
    }

    /// Clear the admin selection.
    pub fn clear_selection(&self) {
        if let Ok(mut sel) = self.admin_selection.write() {
            *sel = AdminSelection::default();
        }
    }
}
