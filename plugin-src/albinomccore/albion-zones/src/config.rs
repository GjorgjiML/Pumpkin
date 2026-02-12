//! Zone configuration types — loaded from `config.toml`.
//! Zones themselves are stored in the database; config only has global settings.

use serde::{Deserialize, Serialize};

/// Top-level config file layout.
#[derive(Debug, Clone, Deserialize)]
pub struct ZonesConfig {
    pub database: DatabaseConfig,
    pub newbie_protection: NewbieProtectionConfig,
    pub death: DeathConfig,
    pub wilderness: WildernessConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewbieProtectionConfig {
    /// Hours of playtime required before entering Red/Black.
    pub required_hours: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeathConfig {
    /// Percentage of dropped items that get destroyed (trash).
    pub trash_chance_percent: u8,
    /// Default partial drop percent for Yellow zones.
    pub partial_drop_percent: u8,
}

/// What applies when a player is outside any defined zone.
#[derive(Debug, Clone, Deserialize)]
pub struct WildernessConfig {
    pub risk: RiskLevel,
    pub pvp_enabled: bool,
    pub death_rule: DeathRule,
}

/// Risk level determines bossbar color and rule set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Green,
    Yellow,
    Red,
    Black,
}

impl RiskLevel {
    /// Whether this risk level requires newbie protection checks.
    #[must_use]
    pub const fn is_dangerous(self) -> bool {
        matches!(self, Self::Red | Self::Black)
    }

    /// Human-readable label used in bossbars and messages.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Green => "SAFE",
            Self::Yellow => "CAUTION",
            Self::Red => "DANGER",
            Self::Black => "LETHAL",
        }
    }

    /// Parse from a string (for commands).
    #[must_use]
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "green" => Some(Self::Green),
            "yellow" => Some(Self::Yellow),
            "red" => Some(Self::Red),
            "black" => Some(Self::Black),
            _ => None,
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Green => write!(f, "Green"),
            Self::Yellow => write!(f, "Yellow"),
            Self::Red => write!(f, "Red"),
            Self::Black => write!(f, "Black"),
        }
    }
}

/// What happens to a player's items on death.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathRule {
    /// No item loss.
    Safe,
    /// Drop a percentage of inventory.
    Partial,
    /// Drop everything; trash chance applies.
    FullLoot,
}

impl ZonesConfig {
    /// Load config from a TOML file, falling back to built-in defaults.
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        if path.exists() {
            let text =
                std::fs::read_to_string(path).map_err(|e| format!("read config: {e}"))?;
            toml::from_str(&text).map_err(|e| format!("parse config: {e}"))
        } else {
            let default_toml = include_str!("../config.toml");
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create config dir: {e}"))?;
            }
            std::fs::write(path, default_toml)
                .map_err(|e| format!("write default config: {e}"))?;
            log::info!("albion_zones: Created default config at {path:?}");
            toml::from_str(default_toml).map_err(|e| format!("parse default config: {e}"))
        }
    }
}
