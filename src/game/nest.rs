use std::collections::HashSet;

use crate::core::{HexCoord, OffsetFarmZone};

pub const NEST_FARM_RADIUS: i32 = 2;
pub const SIEGE_DINO_SECONDS_TARGET: f32 = 300.0;

#[derive(Debug, Clone)]
pub struct Nest {
    pub team: usize,
    pub position: HexCoord,
    pub food: f32,
    pub siege_progress: f32,
    pub siege_team: Option<usize>,
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
            siege_progress: 0.0,
            siege_team: None,
            farm_within,
        }
    }

    pub fn set_farm_within(&mut self, farm_within: HashSet<HexCoord>) {
        self.farm_within = farm_within;
    }

    pub fn has_siege_damage(&self) -> bool {
        self.siege_progress > 0.0
    }

    pub fn reset_siege(&mut self) {
        self.siege_progress = 0.0;
        self.siege_team = None;
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
