//! Region storage: lookup by dimension and position, list, add, remove, mutate.

use std::collections::HashMap;

use pumpkin_util::math::position::BlockPos;

use crate::region::Region;

/// Key for a region: (dimension_id, region_name).
pub type RegionKey = (u8, String);

/// In-memory store of all regions.
pub struct RegionStore {
    regions: HashMap<RegionKey, Region>,
}

impl RegionStore {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    pub fn add(&mut self, dimension_id: u8, name: String, region: Region) -> bool {
        self.regions.insert((dimension_id, name), region).is_none()
    }

    pub fn remove(&mut self, dimension_id: u8, name: &str) -> Option<Region> {
        self.regions.remove(&(dimension_id, name.to_string()))
    }

    pub fn get_region_at(
        &self,
        dimension_id: u8,
        pos: &BlockPos,
    ) -> Option<((u8, String), &Region)> {
        self.regions
            .iter()
            .find(|((dim, _), r)| *dim == dimension_id && r.contains(pos))
            .map(|(k, v)| ((k.0, k.1.clone()), v))
    }

    pub fn list(&self, dimension_id: u8) -> Vec<(String, BlockPos, BlockPos)> {
        self.regions
            .iter()
            .filter(|((dim, _), _)| *dim == dimension_id)
            .map(|((_, name), r)| (name.clone(), r.min, r.max))
            .collect()
    }

    pub fn get_region_mut(&mut self, dimension_id: u8, name: &str) -> Option<&mut Region> {
        self.regions.get_mut(&(dimension_id, name.to_string()))
    }
}
