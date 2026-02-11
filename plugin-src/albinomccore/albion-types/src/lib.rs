//! Shared types for AlbionMC plugins.
//!
//! No server or database dependencies — safe to use from any crate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Player profile model. Stored in PostgreSQL by albion-db.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub uuid: Uuid,
    pub silver: i64,
    pub fame: i64,
    pub mastery: HashMap<String, i64>,
    pub flags: HashMap<String, bool>,
}

impl Default for PlayerProfile {
    fn default() -> Self {
        Self {
            uuid: Uuid::nil(),
            silver: 0,
            fame: 0,
            mastery: HashMap::new(),
            flags: HashMap::new(),
        }
    }
}

impl PlayerProfile {
    #[must_use]
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            ..Default::default()
        }
    }
}
