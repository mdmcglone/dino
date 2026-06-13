use crate::core::HexCoord;

#[derive(Debug, Clone, Copy)]
pub struct Nest {
    pub team: usize,
    pub position: HexCoord,
    last_spawn_time: f64,
}

impl Nest {
    pub fn new(team: usize, position: HexCoord, spawn_time: f64) -> Self {
        Self {
            team,
            position,
            last_spawn_time: spawn_time,
        }
    }

    pub fn should_spawn(&self, current_time: f64, interval: f64) -> bool {
        current_time - self.last_spawn_time >= interval
    }

    pub fn mark_spawned(&mut self, current_time: f64) {
        self.last_spawn_time = current_time;
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
