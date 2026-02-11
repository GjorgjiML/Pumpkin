//! Selection wand state: per-player pos1/pos2 for defining regions.

use std::collections::HashMap;

use pumpkin_util::math::position::BlockPos;
use uuid::Uuid;

/// Per-player selection (pos1, pos2) per dimension.
#[derive(Default)]
pub struct SelectionStore {
    /// (dimension_id, (pos1, pos2))
    selections: HashMap<Uuid, (u8, Option<BlockPos>, Option<BlockPos>)>,
}

impl SelectionStore {
    pub fn set_pos(&mut self, uuid: Uuid, dimension_id: u8, pos: BlockPos, is_first: bool) {
        let entry = self.selections.entry(uuid).or_insert((dimension_id, None, None));
        if entry.0 != dimension_id {
            *entry = (dimension_id, None, None);
        }
        if is_first {
            entry.1 = Some(pos);
        } else {
            entry.2 = Some(pos);
        }
    }

    pub fn get(&self, uuid: &Uuid) -> Option<(BlockPos, BlockPos)> {
        let (_, p1, p2) = self.selections.get(uuid)?;
        let pos1 = *p1.as_ref()?;
        let pos2 = *p2.as_ref()?;
        Some((pos1, pos2))
    }
}
