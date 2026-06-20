// Mouse input handling

use macroquad::prelude::*;
use crate::core::HexCoord;
use crate::rendering::HexMapRenderer;

pub struct MouseHandler;

impl MouseHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn pixel_to_hex(renderer: &HexMapRenderer, mouse_x: f32, mouse_y: f32) -> HexCoord {
        let adjusted_x = mouse_x - 100.0 + renderer.get_camera_x();
        let adjusted_y = mouse_y - 100.0 + renderer.get_camera_y();

        let hex_size = renderer.get_hex_size();
        let sqrt3 = 3.0_f32.sqrt();

        let q = (adjusted_x * 2.0 / 3.0 / hex_size).round() as i32;
        let y_offset = if q % 2 == 1 { hex_size * sqrt3 / 2.0 } else { 0.0 };
        let r = ((adjusted_y - y_offset) / (hex_size * sqrt3)).round() as i32;

        // Refine among the guess and its neighbors — naive rounding misses near hex edges.
        let guess = HexCoord::new(q, r);
        let mut best = guess;
        let mut best_dist_sq = f32::MAX;

        for candidate in std::iter::once(guess).chain(guess.offset_neighbors()) {
            let (center_x, center_y) = renderer.hex_to_pixel(&candidate);
            let dx = mouse_x - center_x;
            let dy = mouse_y - center_y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best = candidate;
            }
        }

        best
    }
    
    pub fn get_mouse_hex(&self, renderer: &HexMapRenderer) -> Option<HexCoord> {
        let (mouse_x, mouse_y) = mouse_position();
        Some(Self::pixel_to_hex(renderer, mouse_x, mouse_y))
    }
} 