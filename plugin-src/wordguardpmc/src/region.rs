//! Region and flag types for WordGuardPMC.
//!
//! Decoupled from plugin/command logic so region rules can be reused or tested independently.

use std::collections::HashSet;

use pumpkin_util::math::position::BlockPos;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Per-region flags. When a flag is true, only owners/members can do that action.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RegionFlags {
    /// If true, only owners/members can break blocks (deny for others).
    pub block_break: bool,
    /// If true, only owners/members can place blocks (deny for others).
    pub block_place: bool,
}

impl RegionFlags {
    pub const fn all_protected() -> Self {
        Self {
            block_break: true,
            block_place: true,
        }
    }
}

/// A protected cuboid region with owners, members, and flags.
#[derive(Clone, Serialize, Deserialize)]
pub struct Region {
    pub min: BlockPos,
    pub max: BlockPos,
    pub owners: HashSet<Uuid>,
    pub members: HashSet<Uuid>,
    pub flags: RegionFlags,
}

impl Region {
    /// Returns true if the block position is inside this region (inclusive min/max).
    pub fn contains(&self, pos: &BlockPos) -> bool {
        let min_x = self.min.0.x.min(self.max.0.x);
        let max_x = self.min.0.x.max(self.max.0.x);
        let min_y = self.min.0.y.min(self.max.0.y);
        let max_y = self.min.0.y.max(self.max.0.y);
        let min_z = self.min.0.z.min(self.max.0.z);
        let max_z = self.min.0.z.max(self.max.0.z);
        pos.0.x >= min_x && pos.0.x <= max_x
            && pos.0.y >= min_y && pos.0.y <= max_y
            && pos.0.z >= min_z && pos.0.z <= max_z
    }

    /// Returns true if the player is allowed to build (break/place) in this region.
    pub fn can_build(&self, uuid: &Uuid) -> bool {
        self.owners.contains(uuid) || self.members.contains(uuid)
    }
}
