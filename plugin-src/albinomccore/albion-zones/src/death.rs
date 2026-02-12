//! Death rules — controls what happens to a player's items on death per zone.
//!
//! Since Pumpkin doesn't currently expose a `PlayerDeathEvent`, this module provides
//! utility functions that can be called when such an event becomes available, or
//! integrated via a health-monitoring tick.

use crate::config::{DeathRule, RiskLevel};
use crate::zone_engine::ZoneEngine;

/// Result of computing death loot rules.
#[derive(Debug, Clone)]
pub struct DeathLootResult {
    /// Zone the player died in.
    pub zone_name: String,
    /// Risk level of the zone.
    pub risk: RiskLevel,
    /// The death rule that applies.
    pub rule: DeathRule,
    /// How many inventory slots (as %) should drop.
    pub drop_percent: u8,
    /// Percentage of dropped items that are destroyed (trashed).
    pub trash_percent: u8,
}

impl DeathLootResult {
    /// Human-readable summary for log or player message.
    #[must_use]
    pub fn summary(&self) -> String {
        match self.rule {
            DeathRule::Safe => format!(
                "You died in {} ({}). Your items are safe.",
                self.zone_name, self.risk
            ),
            DeathRule::Partial => format!(
                "You died in {} ({}). {}% of your items dropped.",
                self.zone_name, self.risk, self.drop_percent
            ),
            DeathRule::FullLoot => format!(
                "You died in {} ({}). ALL items dropped! {}% were destroyed.",
                self.zone_name, self.risk, self.trash_percent
            ),
        }
    }
}

/// Compute what should happen to a player's items when they die at a given position.
#[must_use]
pub fn compute_death_loot(engine: &ZoneEngine, x: f64, y: f64, z: f64) -> DeathLootResult {
    let lookup = engine.zone_at(x, y, z);

    let (drop_percent, trash_percent) = match lookup.death_rule {
        DeathRule::Safe => (0, 0),
        DeathRule::Partial => (lookup.partial_drop_percent, 0),
        DeathRule::FullLoot => (100, engine.trash_chance_percent),
    };

    DeathLootResult {
        zone_name: lookup.name,
        risk: lookup.risk,
        rule: lookup.death_rule,
        drop_percent,
        trash_percent,
    }
}
