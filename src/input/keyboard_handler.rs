// Keyboard input handling

use macroquad::prelude::*;
use crate::rendering::HexMapRenderer;

pub struct KeyboardHandler {
    pan_speed: f32,
    shift_speed: f32,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            pan_speed: 10.0,
            shift_speed: 5.0,
        }
    }
    
    pub fn handle_input(&self, renderer: &mut HexMapRenderer) -> bool {
        // Camera panning
        if is_key_down(KeyCode::Left) {
            renderer.pan_camera(-self.pan_speed, 0.0);
        }
        if is_key_down(KeyCode::Right) {
            renderer.pan_camera(self.pan_speed, 0.0);
        }
        if is_key_down(KeyCode::Up) {
            renderer.pan_camera(0.0, -self.pan_speed);
        }
        if is_key_down(KeyCode::Down) {
            renderer.pan_camera(0.0, self.pan_speed);
        }
        
        // Zoom controls
        if is_key_pressed(KeyCode::KpAdd) || is_key_pressed(KeyCode::Equal) {
            renderer.zoom_in();
        }
        if is_key_pressed(KeyCode::KpSubtract) || is_key_pressed(KeyCode::Minus) {
            renderer.zoom_out();
        }
        if is_key_pressed(KeyCode::Key0) {
            renderer.reset_zoom();
        }
        
        // Overlay controls
        if is_key_pressed(KeyCode::O) {
            renderer.toggle_overlay();
        }
        
        // Overlay opacity controls
        if is_key_pressed(KeyCode::LeftBracket) {
            renderer.adjust_overlay_alpha(-0.1);
        }
        if is_key_pressed(KeyCode::RightBracket) {
            renderer.adjust_overlay_alpha(0.1);
        }
        
        // Overlay position controls
        if is_key_down(KeyCode::W) {
            renderer.shift_overlay(0.0, -self.shift_speed);
        }
        if is_key_down(KeyCode::S) {
            renderer.shift_overlay(0.0, self.shift_speed);
        }
        if is_key_down(KeyCode::A) {
            renderer.shift_overlay(-self.shift_speed, 0.0);
        }
        if is_key_down(KeyCode::D) {
            renderer.shift_overlay(self.shift_speed, 0.0);
        }
        
        // Reset overlay position
        if is_key_pressed(KeyCode::R) {
            renderer.reset_overlay_position();
        }
        
        // Exit
        if is_key_pressed(KeyCode::Escape) {
            return true;
        }
        
        false
    }
} 