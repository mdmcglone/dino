use std::collections::HashSet;

use crate::core::{HexCoord, OffsetFarmZone};

pub const NEST_FARM_RADIUS: i32 = 2;

#[derive(Debug, Clone)]
pub struct Nest {
    pub team: usize,
    pub position: HexCoord,
    pub food: f32,
    farm_within: HashSet<HexCoord>,
}

impl Nest {
    pub fn new(team: usize, position: HexCoord, already_claimed: &HashSet<HexCoord>) -> Self {
        let farm_within = OffsetFarmZone::compute_claimed(
            &position,
            NEST_FARM_RADIUS,
            already_claimed,
        );
        Self {
            team,
            position,
            food: 0.0,
            farm_within,
        }
    }

    pub fn is_in_farm_range(&self, coord: &HexCoord) -> bool {
        self.farm_within.contains(coord)
    }

    pub fn farm_within(&self) -> &HashSet<HexCoord> {
        &self.farm_within
    }

    /// Spawn positions for the three team members: above, bottom-left, bottom-right of the nest.
    pub fn member_spawn_positions(&self) -> [HexCoord; 3] {
        let above = HexCoord::new(self.position.q, self.position.r - 1);
        let (bottom_left, bottom_right) = if self.position.q % 2 == 0 {
            (
                HexCoord::new(self.position.q - 1, self.position.r),
                HexCoord::new(self.position.q + 1, self.position.r),
            )
        } else {
            (
                HexCoord::new(self.position.q - 1, self.position.r + 1),
                HexCoord::new(self.position.q + 1, self.position.r + 1),
            )
        };
        [above, bottom_left, bottom_right]
    }

    pub fn occupied_tiles(&self) -> Vec<HexCoord> {
        let mut tiles = vec![self.position];
        tiles.extend_from_slice(&self.member_spawn_positions());
        tiles
    }
}
