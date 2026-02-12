//! Zone engine — determines which zone a position belongs to using AABB regions.
//! Zones are rectangular boxes defined by two corners (pos1, pos2).

use crate::config::{DeathRule, RiskLevel, WildernessConfig};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// A single zone region defined by two opposite corners (axis-aligned bounding box).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneRegion {
    pub name: String,
    pub risk: RiskLevel,
    pub pvp_enabled: bool,
    pub death_rule: DeathRule,
    pub partial_drop_percent: u8,
    /// Minimum corner (lower x, y, z).
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    /// Maximum corner (upper x, y, z).
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl ZoneRegion {
    /// Create a new region from two arbitrary corners — auto-sorts min/max.
    #[must_use]
    pub fn new(
        name: String,
        risk: RiskLevel,
        pvp_enabled: bool,
        death_rule: DeathRule,
        partial_drop_percent: u8,
        x1: f64,
        y1: f64,
        z1: f64,
        x2: f64,
        y2: f64,
        z2: f64,
    ) -> Self {
        Self {
            name,
            risk,
            pvp_enabled,
            death_rule,
            partial_drop_percent,
            min_x: x1.min(x2),
            min_y: y1.min(y2),
            min_z: z1.min(z2),
            max_x: x1.max(x2),
            max_y: y1.max(y2),
            max_z: z1.max(z2),
        }
    }

    /// Check if a position is inside this region.
    #[must_use]
    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        x >= self.min_x
            && x <= self.max_x
            && y >= self.min_y
            && y <= self.max_y
            && z >= self.min_z
            && z <= self.max_z
    }
}

/// Zone lookup result — either a defined zone or wilderness.
#[derive(Debug, Clone)]
pub struct ZoneLookup {
    pub name: String,
    pub risk: RiskLevel,
    pub pvp_enabled: bool,
    pub death_rule: DeathRule,
    pub partial_drop_percent: u8,
}

/// Mutable zone engine that supports adding/removing regions at runtime.
#[derive(Debug)]
pub struct ZoneEngine {
    /// All defined zone regions. Protected by RwLock for live edits.
    pub regions: RwLock<Vec<ZoneRegion>>,
    /// Fallback for positions not inside any zone.
    pub wilderness: WildernessConfig,
    pub trash_chance_percent: u8,
    pub newbie_required_hours: u64,
    pub default_partial_drop: u8,
}

impl ZoneEngine {
    /// Build the engine from config + pre-loaded regions.
    #[must_use]
    pub fn new(
        wilderness: WildernessConfig,
        trash_chance_percent: u8,
        newbie_required_hours: u64,
        default_partial_drop: u8,
        regions: Vec<ZoneRegion>,
    ) -> Self {
        Self {
            regions: RwLock::new(regions),
            wilderness,
            trash_chance_percent,
            newbie_required_hours,
            default_partial_drop,
        }
    }

    /// Find which zone a world position (x, y, z) falls in.
    /// Returns the first matching region, or wilderness defaults.
    #[must_use]
    pub fn zone_at(&self, x: f64, y: f64, z: f64) -> ZoneLookup {
        if let Ok(regions) = self.regions.read() {
            for region in regions.iter() {
                if region.contains(x, y, z) {
                    return ZoneLookup {
                        name: region.name.clone(),
                        risk: region.risk,
                        pvp_enabled: region.pvp_enabled,
                        death_rule: region.death_rule,
                        partial_drop_percent: region.partial_drop_percent,
                    };
                }
            }
        }
        // Wilderness fallback
        ZoneLookup {
            name: "Wilderness".to_owned(),
            risk: self.wilderness.risk,
            pvp_enabled: self.wilderness.pvp_enabled,
            death_rule: self.wilderness.death_rule,
            partial_drop_percent: self.default_partial_drop,
        }
    }

    /// Convenience: get zone name at position.
    #[must_use]
    pub fn zone_name_at(&self, x: f64, y: f64, z: f64) -> String {
        self.zone_at(x, y, z).name
    }

    /// Convenience: get risk level at position.
    #[must_use]
    pub fn risk_at(&self, x: f64, y: f64, z: f64) -> RiskLevel {
        self.zone_at(x, y, z).risk
    }

    /// Whether PvP is allowed at position.
    #[must_use]
    pub fn pvp_at(&self, x: f64, y: f64, z: f64) -> bool {
        self.zone_at(x, y, z).pvp_enabled
    }

    /// Add a new zone region at runtime.
    pub fn add_region(&self, region: ZoneRegion) {
        if let Ok(mut regions) = self.regions.write() {
            regions.push(region);
        }
    }

    /// Remove a zone by name. Returns true if found and removed.
    pub fn remove_region(&self, name: &str) -> bool {
        if let Ok(mut regions) = self.regions.write() {
            let before = regions.len();
            regions.retain(|r| r.name != name);
            return regions.len() < before;
        }
        false
    }

    /// Check if a zone name already exists.
    #[must_use]
    pub fn zone_exists(&self, name: &str) -> bool {
        self.regions
            .read()
            .map_or(false, |r| r.iter().any(|z| z.name == name))
    }

    /// Get count of defined zones.
    #[must_use]
    pub fn zone_count(&self) -> usize {
        self.regions.read().map_or(0, |r| r.len())
    }

    /// Get a snapshot of all zone regions (for listing).
    #[must_use]
    pub fn all_regions(&self) -> Vec<ZoneRegion> {
        self.regions.read().map_or_else(|_| Vec::new(), |r| r.clone())
    }
}
