//! Zone service — exposed to other plugins via Pumpkin's service registry.
//!
//! Other plugins can query the current zone of a player, check PvP status, or
//! get death rules without directly depending on albion_zones internals.

use crate::config::RiskLevel;
use crate::death::{self, DeathLootResult};
use crate::state::PluginState;
use crate::zone_engine::{ZoneEngine, ZoneLookup};
use std::sync::Arc;

/// Public zone service for cross-plugin queries.
pub struct ZoneService {
    pub engine: Arc<ZoneEngine>,
    pub state: PluginState,
}

impl ZoneService {
    #[must_use]
    pub fn new(state: &PluginState) -> Self {
        Self {
            engine: Arc::clone(&state.zone_engine),
            state: state.clone(),
        }
    }

    /// Get full zone lookup at a world position.
    #[must_use]
    pub fn zone_at(&self, x: f64, y: f64, z: f64) -> ZoneLookup {
        self.engine.zone_at(x, y, z)
    }

    /// Get the zone name at a world position.
    #[must_use]
    pub fn zone_name_at(&self, x: f64, y: f64, z: f64) -> String {
        self.engine.zone_name_at(x, y, z)
    }

    /// Get the risk level at a world position.
    #[must_use]
    pub fn risk_at(&self, x: f64, y: f64, z: f64) -> RiskLevel {
        self.engine.risk_at(x, y, z)
    }

    /// Whether PvP is allowed at a world position.
    #[must_use]
    pub fn pvp_at(&self, x: f64, y: f64, z: f64) -> bool {
        self.engine.pvp_at(x, y, z)
    }

    /// Compute full death loot result at a position.
    #[must_use]
    pub fn compute_death_loot(&self, x: f64, y: f64, z: f64) -> DeathLootResult {
        death::compute_death_loot(&self.engine, x, y, z)
    }
}

impl pumpkin::plugin::api::Payload for ZoneService {
    fn get_name_static() -> &'static str {
        "albion_zones::ZoneService"
    }
    fn get_name(&self) -> &'static str {
        Self::get_name_static()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
