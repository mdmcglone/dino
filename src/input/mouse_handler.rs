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
        // This is the inverse of hex_to_pixel
        // For pointy-top hexagons with offset coordinates
        let adjusted_x = mouse_x - 100.0 + renderer.get_camera_x();
        let adjusted_y = mouse_y - 100.0 + renderer.get_camera_y();
        
        let hex_size = renderer.get_hex_size();
        let sqrt3 = 3.0_f32.sqrt();
        
        // Approximate q coordinate
        let q = (adjusted_x * 2.0 / 3.0 / hex_size).round() as i32;
        
        // Calculate y offset for this column
        let y_offset = if q % 2 == 1 { hex_size * sqrt3 / 2.0 } else { 0.0 };
        
        // Calculate r coordinate
        let r = ((adjusted_y - y_offset) / (hex_size * sqrt3)).round() as i32;
        
        HexCoord::new(q, r)
    }
    
    pub fn get_mouse_hex(&self, renderer: &HexMapRenderer) -> Option<HexCoord> {
        let (mouse_x, mouse_y) = mouse_position();
        Some(Self::pixel_to_hex(renderer, mouse_x, mouse_y))
    }
} 