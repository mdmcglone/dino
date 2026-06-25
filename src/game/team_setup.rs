use macroquad::prelude::*;
use super::team_abilities;
use crate::maps::{MapKind, MapSize};

pub const MIN_TEAMS: usize = 2;
pub const MAX_TEAMS: usize = 4;
pub const PLAYER_DINO_COUNT: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct TeamSetup {
    pub dino_type: usize,
    pub color: Color,
}

impl TeamSetup {
    pub fn default_for_slot(slot: usize) -> Self {
        Self {
            dino_type: slot % PLAYER_DINO_COUNT,
            color: default_team_color(slot),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MatchConfig {
    pub teams: Vec<TeamSetup>,
    pub map_kind: MapKind,
    pub map_size: MapSize,
}

impl MatchConfig {
    pub fn debug_default() -> Self {
        Self {
            teams: (0..team_abilities::PLAYER_TEAMS)
                .map(TeamSetup::default_for_slot)
                .collect(),
            map_kind: MapKind::Pangaea,
            map_size: MapSize::Medium,
        }
    }

    pub fn num_teams(&self) -> usize {
        self.teams.len()
    }
}

pub fn default_team_color(slot: usize) -> Color {
    match slot {
        0 => Color::new(0.85, 0.55, 0.1, 1.0),
        1 => Color::new(0.15, 0.65, 0.55, 1.0),
        2 => Color::new(0.45, 0.55, 0.95, 1.0),
        3 => Color::new(0.75, 0.25, 0.55, 1.0),
        _ => Color::new(0.6, 0.6, 0.6, 1.0),
    }
}

pub fn krono_team_color() -> Color {
    Color::new(0.35, 0.15, 0.45, 1.0)
}

pub fn color_palette() -> Vec<Color> {
    const RGB: &[[u8; 3]] = &[
        [220, 60, 60],
        [230, 100, 40],
        [240, 180, 30],
        [120, 200, 60],
        [40, 170, 110],
        [50, 150, 210],
        [70, 100, 220],
        [130, 70, 200],
        [190, 60, 170],
        [240, 120, 170],
        [180, 90, 50],
        [140, 110, 70],
        [90, 90, 90],
        [200, 200, 200],
        [255, 140, 140],
        [255, 200, 120],
        [255, 255, 140],
        [180, 240, 160],
        [140, 220, 220],
        [160, 180, 255],
        [200, 160, 255],
        [255, 160, 220],
        [100, 40, 40],
        [100, 70, 30],
        [60, 90, 40],
        [30, 80, 80],
        [30, 50, 100],
        [70, 40, 90],
        [120, 50, 90],
        [160, 160, 160],
        [40, 40, 40],
        [250, 250, 250],
    ];

    RGB.iter()
        .map(|&[r, g, b]| Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0))
        .collect()
}

pub fn player_dino_name(dino_type: usize) -> &'static str {
    team_abilities::team_name(dino_type)
}

pub fn cycle_dino(dino_type: usize, delta: i32) -> usize {
    let next = (dino_type as i32 + delta).rem_euclid(PLAYER_DINO_COUNT as i32);
    next as usize
}
