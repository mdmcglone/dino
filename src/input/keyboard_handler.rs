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
        // Camera panning (inverted to feel natural - arrow keys move the view)
        if is_key_down(KeyCode::Left) {
            renderer.pan_camera(self.pan_speed, 0.0);
        }
        if is_key_down(KeyCode::Right) {
            renderer.pan_camera(-self.pan_speed, 0.0);
        }
        if is_key_down(KeyCode::Up) {
            renderer.pan_camera(0.0, self.pan_speed);
        }
        if is_key_down(KeyCode::Down) {
            renderer.pan_camera(0.0, -self.pan_speed);
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
        
        // Exit
        if is_key_pressed(KeyCode::Escape) {
            return true;
        }
        
        false
    }
} 