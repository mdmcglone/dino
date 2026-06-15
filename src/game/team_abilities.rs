// Per-team dino bonuses (civ-style abilities)

pub const TREX_TEAM: usize = 0;
pub const BRONTO_TEAM: usize = 1;
pub const PTERO_TEAM: usize = 2;
pub const TRICERA_TEAM: usize = 3;
pub const KRONO_TEAM: usize = 4;
pub const PLAYER_TEAMS: usize = 4;

const TREX_ATTACK_CYCLE_MULTIPLIER: f64 = 0.9;
const BRONTO_SIEGE_RATE_MULTIPLIER: f64 = 1.5;
const DEFAULT_POPULATION_PER_NEST: usize = 20;
const TRICERA_POPULATION_PER_NEST: usize = 30;

pub fn team_name(team: usize) -> &'static str {
    match team {
        TREX_TEAM => "T-Rex",
        BRONTO_TEAM => "Bronto",
        PTERO_TEAM => "Ptero",
        TRICERA_TEAM => "Tricera",
        KRONO_TEAM => "Krono",
        _ => "Unknown",
    }
}

/// Max dinos supported per nest (population cap scales with nest count).
pub fn population_per_nest(team: usize) -> usize {
    match team {
        TRICERA_TEAM => TRICERA_POPULATION_PER_NEST,
        _ => DEFAULT_POPULATION_PER_NEST,
    }
}

/// Pteros fly over mountains and deep water; nest placement still uses walkable land only.
pub fn can_fly_over_terrain(team: usize) -> bool {
    team == PTERO_TEAM
}

/// Pteros ignore the rough-terrain movement speed penalty.
pub fn ignores_rough_terrain_movement_penalty(team: usize) -> bool {
    team == PTERO_TEAM
}

/// Multiplier applied to combat cycle duration (< 1.0 = faster attacks).
pub fn combat_cycle_multiplier(team: usize) -> f64 {
    match team {
        TREX_TEAM => TREX_ATTACK_CYCLE_MULTIPLIER,
        _ => 1.0,
    }
}

/// Multiplier for siege capture rate when this team is attacking a nest.
pub fn siege_attack_rate_multiplier(team: usize) -> f64 {
    match team {
        BRONTO_TEAM => BRONTO_SIEGE_RATE_MULTIPLIER,
        _ => 1.0,
    }
}

/// Multiplier for siege repair rate when this team is defending its nest.
pub fn siege_repair_rate_multiplier(team: usize) -> f64 {
    match team {
        BRONTO_TEAM => BRONTO_SIEGE_RATE_MULTIPLIER,
        _ => 1.0,
    }
}
