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
    team_sprites: Vec<Option<Texture2D>>,
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
            team_sprites: vec![None, None],
        }
    }

    pub async fn load_team_sprite(&mut self, team: usize, path: &str) {
        match load_texture(path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest); // Crisp pixel art
                if team < self.team_sprites.len() {
                    self.team_sprites[team] = Some(texture);
                    println!("Team {} sprite loaded successfully", team);
                }
            }
            Err(e) => {
                println!("Failed to load team {} sprite: {:?}", team, e);
            }
        }
    }
    
    // Camera controls
    pub fn pan_camera(&mut self, dx: f32, dy: f32) {
        self.camera_x -= dx;
        self.camera_y -= dy;
    }
    
    pub fn get_camera_x(&self) -> f32 {
        self.camera_x
    }
    
    pub fn get_camera_y(&self) -> f32 {
        self.camera_y
    }
    
    pub fn get_hex_size(&self) -> f32 {
        self.hex_size
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
        
        // // Draw coordinates on the hex
        // let coord_text = format!("{},{}", coord.q, coord.r);
        // let text_size = 12.0 * self.zoom_level;
        // let text_color = Color::new(0.0, 0.0, 0.0, 0.8); // Black with slight transparency
        
        // // Center the text
        // let text_width = coord_text.len() as f32 * text_size * 0.5;
        // let text_x = center_x - text_width / 2.0;
        // let text_y = center_y + text_size / 3.0;
        
        // // Draw white background for better readability
        // draw_rectangle(
        //     text_x - 2.0,
        //     text_y - text_size + 2.0,
        //     text_width + 4.0,
        //     text_size,
        //     Color::new(1.0, 1.0, 1.0, 0.7)
        // );
        
        // // Draw the coordinate text
        // draw_text(&coord_text, text_x, text_y, text_size, text_color);
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
    
    pub fn draw_player(&self, coord: &HexCoord, team: usize) {
        self.draw_player_with_offset(coord, team, 0.0);
    }
    
    pub fn draw_player_with_offset(&self, coord: &HexCoord, team: usize, offset_factor: f32) {
        let (center_x, center_y) = self.hex_to_pixel(coord);
        
        // Apply horizontal offset for side-by-side battles (half sprite width from center)
        let sprite_size = 40.0 * self.zoom_level;
        let offset_x = offset_factor * sprite_size;
        let draw_x = center_x + offset_x;
        
        // Get sprite for this team
        let sprite = if team < self.team_sprites.len() {
            self.team_sprites[team].as_ref()
        } else {
            None
        };
        
        if let Some(sprite) = sprite {
            // Draw the sprite centered on the hex
            let sprite_size = 40.0 * self.zoom_level;
            let x = draw_x - sprite_size / 2.0;
            let y = center_y - sprite_size / 2.0;
            
            draw_texture_ex(
                *sprite,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(sprite_size, sprite_size)),
                    ..Default::default()
                }
            );
        } else {
            // Fallback to stick figure if sprite not loaded
            let scale = self.zoom_level;
            
            // Head
            draw_circle(draw_x, center_y - 10.0 * scale, 5.0 * scale, Color::new(0.1, 0.1, 0.1, 1.0));
            
            // Body
            draw_line(
                draw_x, center_y - 5.0 * scale,
                draw_x, center_y + 10.0 * scale,
                2.0 * scale,
                Color::new(0.1, 0.1, 0.1, 1.0)
            );
            
            // Arms
            draw_line(
                draw_x - 8.0 * scale, center_y,
                draw_x + 8.0 * scale, center_y,
                2.0 * scale,
                Color::new(0.1, 0.1, 0.1, 1.0)
            );
            
            // Left leg
            draw_line(
                draw_x, center_y + 10.0 * scale,
                draw_x - 5.0 * scale, center_y + 20.0 * scale,
                2.0 * scale,
                Color::new(0.1, 0.1, 0.1, 1.0)
            );
            
            // Right leg
            draw_line(
                draw_x, center_y + 10.0 * scale,
                draw_x + 5.0 * scale, center_y + 20.0 * scale,
                2.0 * scale,
                Color::new(0.1, 0.1, 0.1, 1.0)
            );
        }
    }    
    pub fn draw_selection_highlight(&self, coord: &HexCoord) {
        let (center_x, center_y) = self.hex_to_pixel(coord);
        
        // Calculate hexagon vertices
        let mut vertices = Vec::new();
        for i in 0..6 {
            let angle = std::f32::consts::PI / 3.0 * i as f32;
            vertices.push(Vec2::new(
                center_x + self.hex_size * angle.cos(),
                center_y + self.hex_size * angle.sin()
            ));
        }
        
        // Draw thick yellow border for selection
        let highlight_color = Color::new(1.0, 1.0, 0.0, 0.8); // Yellow
        for i in 0..vertices.len() {
            let next = (i + 1) % vertices.len();
            draw_line(
                vertices[i].x, vertices[i].y,
                vertices[next].x, vertices[next].y,
                3.0,
                highlight_color
            );
        }
    }
    
    pub fn draw_movement_arrow(&self, from: &HexCoord, to: &HexCoord, progress: f32) {
        let (from_x, from_y) = self.hex_to_pixel(from);
        let (to_x, to_y) = self.hex_to_pixel(to);
        
        // Calculate arrow direction and length
        let dx = to_x - from_x;
        let dy = to_y - from_y;
        let total_length = (dx * dx + dy * dy).sqrt();
        
        // Normalize direction
        let dir_x = dx / total_length;
        let dir_y = dy / total_length;
        
        // Calculate the current end point based on progress
        let current_length = total_length * progress;
        let end_x = from_x + dir_x * current_length;
        let end_y = from_y + dir_y * current_length;
        
        // Draw the main arrow shaft
        let arrow_color = Color::new(0.2, 0.6, 1.0, 0.9); // Bright blue
        let shaft_thickness = 4.0 * self.zoom_level;
        draw_line(from_x, from_y, end_x, end_y, shaft_thickness, arrow_color);
        
        // Draw arrowhead if we're past 20% progress
        if progress > 0.2 {
            let arrowhead_size = 12.0 * self.zoom_level;
            
            // Calculate perpendicular vector
            let perp_x = -dir_y;
            let perp_y = dir_x;
            
            // Arrowhead points
            let head_back = arrowhead_size * 0.8;
            let head_base_x = end_x - dir_x * head_back;
            let head_base_y = end_y - dir_y * head_back;
            
            let wing_offset = arrowhead_size * 0.5;
            let left_x = head_base_x + perp_x * wing_offset;
            let left_y = head_base_y + perp_y * wing_offset;
            let right_x = head_base_x - perp_x * wing_offset;
            let right_y = head_base_y - perp_y * wing_offset;
            
            // Draw filled arrowhead triangle
            draw_triangle(
                Vec2::new(end_x, end_y),
                Vec2::new(left_x, left_y),
                Vec2::new(right_x, right_y),
                arrow_color
            );
        }
        
        // Draw a glowing circle at the current position
        let glow_radius = 6.0 * self.zoom_level;
        draw_circle(end_x, end_y, glow_radius, Color::new(1.0, 1.0, 1.0, 0.8));
        draw_circle(end_x, end_y, glow_radius * 0.6, arrow_color);
    }
    
    pub fn draw_stack_count(&self, coord: &HexCoord, count: usize) {
        self.draw_team_stack_count(coord, 0, count, 0.0);
    }
    
    pub fn draw_team_stack_count(&self, coord: &HexCoord, _team: usize, count: usize, offset_factor: f32) {
        let (center_x, center_y) = self.hex_to_pixel(coord);
        
        // Apply horizontal offset for side-by-side battles (half sprite width from center)
        let sprite_size = 40.0 * self.zoom_level;
        let offset_x = offset_factor * sprite_size;
        let draw_x = center_x + offset_x;
        
        // Position the count indicator in the bottom-right corner of the sprite
        let count_x = draw_x + sprite_size * 0.3;
        let count_y = center_y + sprite_size * 0.3;
        
        let text = count.to_string();
        let font_size = 20.0 * self.zoom_level;
        
        // Draw background circle
        let bg_radius = 10.0 * self.zoom_level;
        draw_circle(count_x, count_y, bg_radius, Color::new(0.0, 0.0, 0.0, 0.8));
        
        // Draw white border
        draw_circle_lines(count_x, count_y, bg_radius, 1.5, Color::new(1.0, 1.0, 1.0, 1.0));
        
        // Draw count text
        let text_width = measure_text(&text, None, font_size as u16, 1.0).width;
        draw_text(
            &text,
            count_x - text_width / 2.0,
            count_y + font_size * 0.35,
            font_size,
            Color::new(1.0, 1.0, 1.0, 1.0)
        );
    }
    
    pub fn draw_battle_indicator(&self, coord: &HexCoord) {
        let (center_x, center_y) = self.hex_to_pixel(coord);
        
        // Draw crossed swords icon above the tile
        let indicator_y = center_y - self.hex_size * 0.8;
        let sword_size = 8.0 * self.zoom_level;
        
        // Red background circle
        draw_circle(center_x, indicator_y, 12.0 * self.zoom_level, Color::new(0.8, 0.0, 0.0, 0.9));
        
        // Draw simple "X" for battle indicator
        let x_size = sword_size;
        draw_line(
            center_x - x_size, indicator_y - x_size,
            center_x + x_size, indicator_y + x_size,
            3.0 * self.zoom_level,
            Color::new(1.0, 1.0, 1.0, 1.0)
        );
        draw_line(
            center_x - x_size, indicator_y + x_size,
            center_x + x_size, indicator_y - x_size,
            3.0 * self.zoom_level,
            Color::new(1.0, 1.0, 1.0, 1.0)
        );
    }
    
    pub fn draw_selection_box(&self, start: (f32, f32), current: (f32, f32)) {
        let (start_x, start_y) = start;
        let (current_x, current_y) = current;
        
        // Calculate rectangle bounds
        let min_x = start_x.min(current_x);
        let max_x = start_x.max(current_x);
        let min_y = start_y.min(current_y);
        let max_y = start_y.max(current_y);
        
        let width = max_x - min_x;
        let height = max_y - min_y;
        
        // Draw filled rectangle with transparency
        draw_rectangle(
            min_x, min_y, width, height,
            Color::new(0.0, 1.0, 0.0, 0.2) // Light green fill
        );
        
        // Draw border
        draw_rectangle_lines(
            min_x, min_y, width, height,
            3.0,
            Color::new(0.0, 1.0, 0.0, 0.9) // Bright green border
        );
    }
} 