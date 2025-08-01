use macroquad::prelude::*;
use crate::map::{HexCoord, Map};

pub struct HexMapRenderer {
    hex_size: f32,
    camera_x: f32,
    camera_y: f32,
}

impl HexMapRenderer {
    pub fn new() -> Self {
        Self {
            hex_size: 30.0,  // Increased from 25.0 for better resolution
            camera_x: 0.0,
            camera_y: 0.0,
        }
    }
    
    pub fn hex_to_pixel(&self, coord: &HexCoord) -> (f32, f32) {
        let x = self.hex_size * 3.0 / 2.0 * coord.q as f32;
        let y = self.hex_size * 3.0_f32.sqrt() * (coord.r as f32 + coord.q as f32 / 2.0);
        (x + 100.0 - self.camera_x, y + 100.0 - self.camera_y)
    }
    
    pub fn draw_hex(&self, coord: &HexCoord, terrain_type: crate::terrain::TerrainType) {
        let (center_x, center_y) = self.hex_to_pixel(coord);
        
        // Skip if off screen with larger margin
        if center_x < -self.hex_size * 2.0 || center_x > screen_width() + self.hex_size * 2.0 ||
           center_y < -self.hex_size * 2.0 || center_y > screen_height() + self.hex_size * 2.0 {
            return;
        }
        
        // Calculate hexagon vertices for flat-top orientation
        let mut vertices = Vec::new();
        for i in 0..6 {
            let angle = std::f32::consts::PI / 3.0 * i as f32; // No rotation - flat-top hexagons
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
    
    pub fn draw_ui(&self) {
        // Draw title
        let title = "PANGAEA";
        let title_size = 48.0;
        let title_color = Color::new(0.2, 0.2, 0.2, 1.0); // Dark gray
        
        // Measure text to center it
        let title_width = title.len() as f32 * title_size * 0.6;
        let title_x = (screen_width() - title_width) / 2.0;
        let title_y = 40.0;
        
        // Draw white background for title
        draw_rectangle(
            title_x - 20.0,
            title_y - title_size + 10.0,
            title_width + 40.0,
            title_size + 10.0,
            Color::new(1.0, 1.0, 1.0, 0.8)
        );
        
        // Draw title
        draw_text(title, title_x, title_y, title_size, title_color);
        
        // Draw controls
        let control_color = Color::new(0.2, 0.2, 0.2, 1.0); // Dark gray
        let control_size = 20.0;
        
        // White background for controls
        draw_rectangle(
            10.0,
            screen_height() - 90.0,
            180.0,
            80.0,
            Color::new(1.0, 1.0, 1.0, 0.8)
        );
        
        draw_text("ARROW KEYS: PAN", 20.0, screen_height() - 60.0, control_size, control_color);
        draw_text("ESC: EXIT", 20.0, screen_height() - 35.0, control_size, control_color);
    }
    
    pub fn pan_camera(&mut self, dx: f32, dy: f32) {
        self.camera_x -= dx;
        self.camera_y -= dy;
    }
} 