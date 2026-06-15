// Keyboard input handling

use macroquad::prelude::*;
use crate::rendering::HexMapRenderer;

pub struct KeyboardHandler {
    /// Screen pixels per second when panning with arrow keys / WASD.
    pan_speed: f32,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            pan_speed: 600.0,
        }
    }
    
    pub fn handle_input(&self, renderer: &mut HexMapRenderer) -> bool {
        let dt = get_frame_time();
        let mut dx = 0.0;
        let mut dy = 0.0;

        // Camera panning (inverted to feel natural - arrow keys / WASD move the view)
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            dx += self.pan_speed * dt;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            dx -= self.pan_speed * dt;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            dy += self.pan_speed * dt;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            dy -= self.pan_speed * dt;
        }
        if dx != 0.0 || dy != 0.0 {
            renderer.pan_camera(dx, dy);
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