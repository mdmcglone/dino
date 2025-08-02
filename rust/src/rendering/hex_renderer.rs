// Core hex map rendering functionality

use macroquad::prelude::*;
use crate::core::HexCoord;
use crate::maps::{Map, TerrainType};
use super::overlay_renderer::OverlayRenderer;
use super::ui_renderer::UIRenderer;

pub struct HexMapRenderer {
    hex_size: f32,
    camera_x: f32,
    camera_y: f32,
    zoom_level: f32,
    base_hex_size: f32,
    overlay_renderer: OverlayRenderer,
    ui_renderer: UIRenderer,
}

impl HexMapRenderer {
    pub fn new() -> Self {
        let base_size = 30.0;
        Self {
            hex_size: base_size,
            camera_x: 0.0,
            camera_y: 0.0,
            zoom_level: 1.0,
            base_hex_size: base_size,
            overlay_renderer: OverlayRenderer::new(),
            ui_renderer: UIRenderer::new(),
        }
    }
    
    // Camera controls
    pub fn pan_camera(&mut self, dx: f32, dy: f32) {
        self.camera_x -= dx;
        self.camera_y -= dy;
    }
    
    // Zoom controls
    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.1).min(3.0);
        self.hex_size = self.base_hex_size * self.zoom_level;
    }
    
    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.1).max(0.3);
        self.hex_size = self.base_hex_size * self.zoom_level;
    }
    
    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
        self.hex_size = self.base_hex_size;
    }
    
    // Overlay controls delegation
    pub async fn load_overlay(&mut self, path: &str) {
        self.overlay_renderer.load_texture(path).await;
    }
    
    pub fn toggle_overlay(&mut self) {
        self.overlay_renderer.toggle();
    }
    
    pub fn adjust_overlay_alpha(&mut self, delta: f32) {
        self.overlay_renderer.adjust_alpha(delta);
    }
    
    pub fn shift_overlay(&mut self, dx: f32, dy: f32) {
        self.overlay_renderer.shift(dx, dy);
    }
    
    pub fn reset_overlay_position(&mut self) {
        self.overlay_renderer.reset_position();
    }
    
    // Core rendering methods
    pub fn hex_to_pixel(&self, coord: &HexCoord) -> (f32, f32) {
        // For pointy-top hexagons in offset coordinates:
        // - Horizontal spacing between columns = 3/2 * size
        // - Vertical spacing between rows = sqrt(3) * size
        // - Odd columns are offset down by half the vertical spacing
        let sqrt3 = 3.0_f32.sqrt();
        
        let x = self.hex_size * 3.0 / 2.0 * coord.q as f32;
        let y_offset = if coord.q % 2 == 1 { self.hex_size * sqrt3 / 2.0 } else { 0.0 };
        let y = self.hex_size * sqrt3 * coord.r as f32 + y_offset;
        
        (x + 100.0 - self.camera_x, y + 100.0 - self.camera_y)
    }
    
    pub fn draw_hex(&self, coord: &HexCoord, terrain_type: TerrainType) {
        let (center_x, center_y) = self.hex_to_pixel(coord);
        
        // Skip if off screen with larger margin
        if center_x < -self.hex_size * 2.0 || center_x > screen_width() + self.hex_size * 2.0 ||
           center_y < -self.hex_size * 2.0 || center_y > screen_height() + self.hex_size * 2.0 {
            return;
        }
        
        // Calculate hexagon vertices for flat-top orientation
        let mut vertices = Vec::new();
        for i in 0..6 {
            let angle = std::f32::consts::PI / 3.0 * i as f32; // + std::f32::consts::PI / 6.0; // Rotate 30 degrees for flat-top
            vertices.push(Vec2::new(
                center_x + self.hex_size * angle.cos(),
                center_y + self.hex_size * angle.sin()
            ));
        }
        
        // Draw filled hexagon using triangle fan
        let color = terrain_type.color();
        for i in 1..vertices.len() - 1 {
            draw_triangle(
                Vec2::new(center_x, center_y),
                vertices[i],
                vertices[i + 1],
                color
            );
        }
        // Close the hexagon
        draw_triangle(
            Vec2::new(center_x, center_y),
            vertices[vertices.len() - 1],
            vertices[0],
            color
        );
        draw_triangle(
            Vec2::new(center_x, center_y),
            vertices[0],
            vertices[1],
            color
        );
        
        // Draw border for better definition
        let border_color = terrain_type.border_color();
        for i in 0..vertices.len() {
            let next = (i + 1) % vertices.len();
            draw_line(
                vertices[i].x, vertices[i].y,
                vertices[next].x, vertices[next].y,
                1.0,
                border_color
            );
        }
        
        // Draw coordinates on the hex
        let coord_text = format!("{},{}", coord.q, coord.r);
        let text_size = 12.0 * self.zoom_level;
        let text_color = Color::new(0.0, 0.0, 0.0, 0.8); // Black with slight transparency
        
        // Center the text
        let text_width = coord_text.len() as f32 * text_size * 0.5;
        let text_x = center_x - text_width / 2.0;
        let text_y = center_y + text_size / 3.0;
        
        // Draw white background for better readability
        draw_rectangle(
            text_x - 2.0,
            text_y - text_size + 2.0,
            text_width + 4.0,
            text_size,
            Color::new(1.0, 1.0, 1.0, 0.7)
        );
        
        // Draw the coordinate text
        draw_text(&coord_text, text_x, text_y, text_size, text_color);
    }
    
    pub fn draw_map(&self, map: &dyn Map) {
        for (coord, terrain) in map.get_tiles() {
            self.draw_hex(coord, *terrain);
        }
    }
    
    pub fn draw_grid_effect(&self) {
        let grid_color = Color::new(0.9, 0.9, 0.9, 0.1); // Very light gray with low opacity
        
        // Vertical lines
        let mut x = 0.0;
        while x < screen_width() {
            draw_line(x, 0.0, x, screen_height(), 1.0, grid_color);
            x += 100.0;
        }
        
        // Horizontal lines
        let mut y = 0.0;
        while y < screen_height() {
            draw_line(0.0, y, screen_width(), y, 1.0, grid_color);
            y += 100.0;
        }
    }
    
    pub fn draw_overlay(&self) {
        self.overlay_renderer.draw(self.hex_size, self.camera_x, self.camera_y);
    }
    
    pub fn draw_ui(&self) {
        self.ui_renderer.draw(
            self.zoom_level,
            self.overlay_renderer.is_visible(),
            self.overlay_renderer.get_alpha(),
            self.overlay_renderer.get_offset(),
            self.overlay_renderer.has_texture(),
        );
    }
} 