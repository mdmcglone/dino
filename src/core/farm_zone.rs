use std::collections::{HashSet, VecDeque};

use super::HexCoord;

#[derive(Clone, Debug, Default)]
pub struct OffsetFarmZone {
    pub within: HashSet<HexCoord>,
    pub border: HashSet<HexCoord>,
}

impl OffsetFarmZone {
    pub fn compute(center: &HexCoord, max_ring: i32) -> Self {
        let mut within = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((*center, 0i32));
        within.insert(*center);

        while let Some((current, dist)) = queue.pop_front() {
            if dist >= max_ring {
                continue;
            }
            for neighbor in current.offset_neighbors() {
                if within.insert(neighbor) {
                    queue.push_back((neighbor, dist + 1));
                }
            }
        }

        Self {
            within,
            border: HashSet::new(),
        }
    }

    /// Full ring zone minus tiles already claimed by another nest.
    pub fn compute_claimed(
        center: &HexCoord,
        max_ring: i32,
        already_claimed: &HashSet<HexCoord>,
    ) -> HashSet<HexCoord> {
        Self::compute(center, max_ring)
            .within
            .into_iter()
            .filter(|coord| !already_claimed.contains(coord))
            .collect()
    }
}
