// Mouse input handling for panning

use macroquad::prelude::*;
use crate::rendering::HexMapRenderer;

pub struct MouseHandler {
    is_dragging: bool,
    last_mouse_pos: (f32, f32),
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            is_dragging: false,
            last_mouse_pos: (0.0, 0.0),
        }
    }
    
    pub fn handle_input(&mut self, renderer: &mut HexMapRenderer) {
        let mouse_pos = mouse_position();
        
        // Start dragging on left mouse button press
        if is_mouse_button_pressed(MouseButton::Left) {
            self.is_dragging = true;
            self.last_mouse_pos = mouse_pos;
        }
        
        // Stop dragging on mouse button release
        if is_mouse_button_released(MouseButton::Left) {
            self.is_dragging = false;
        }
        
        // Pan camera while dragging
        if self.is_dragging {
            let delta_x = mouse_pos.0 - self.last_mouse_pos.0;
            let delta_y = mouse_pos.1 - self.last_mouse_pos.1;
            
            // Pan in the opposite direction of mouse movement
            // (moving mouse right should move the map left, etc.)
            renderer.pan_camera(delta_x, delta_y);
            
            self.last_mouse_pos = mouse_pos;
        }
    }
} 