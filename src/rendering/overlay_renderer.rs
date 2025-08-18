// Overlay rendering functionality

use macroquad::prelude::*;

pub struct OverlayRenderer {
    texture: Option<Texture2D>,
    show_overlay: bool,
    alpha: f32,
    offset_x: f32,
    offset_y: f32,
}

impl OverlayRenderer {
    pub fn new() -> Self {
        Self {
            texture: None,
            show_overlay: true,
            alpha: 0.3,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
    
    pub async fn load_texture(&mut self, path: &str) {
        match load_texture(path).await {
            Ok(texture) => {
                self.texture = Some(texture);
                println!("Overlay texture loaded successfully");
            }
            Err(e) => {
                println!("Failed to load overlay texture: {:?}", e);
            }
        }
    }
    
    pub fn toggle(&mut self) {
        self.show_overlay = !self.show_overlay;
    }
    
    pub fn adjust_alpha(&mut self, delta: f32) {
        self.alpha = (self.alpha + delta).clamp(0.0, 1.0);
    }
    
    pub fn shift(&mut self, dx: f32, dy: f32) {
        self.offset_x += dx;
        self.offset_y += dy;
    }
    
    pub fn reset_position(&mut self) {
        self.offset_x = 0.0;
        self.offset_y = 0.0;
    }
    
    pub fn is_visible(&self) -> bool {
        self.show_overlay
    }
    
    pub fn get_alpha(&self) -> f32 {
        self.alpha
    }
    
    pub fn get_offset(&self) -> (f32, f32) {
        (self.offset_x, self.offset_y)
    }
    
    pub fn has_texture(&self) -> bool {
        self.texture.is_some()
    }
    
    pub fn draw(&self, hex_size: f32, camera_x: f32, camera_y: f32) {
        if self.show_overlay {
            if let Some(texture) = &self.texture {
                // Calculate scale to fit the map area
                // The hex map is roughly 35x35 tiles, each hex_size pixels with 3/2 spacing
                let map_width = 35.0 * hex_size * 3.0 / 2.0;
                let map_height = 35.0 * hex_size * 3.0_f32.sqrt();
                
                // Scale the overlay to match the map size
                let scale_x = map_width / texture.width();
                let scale_y = map_height / texture.height();
                let scale = scale_x.min(scale_y) * 0.8; // 80% to leave some margin
                
                // Center the overlay on the map with offset
                let overlay_width = texture.width() * scale;
                let overlay_height = texture.height() * scale;
                let overlay_x = 100.0 - camera_x + (map_width - overlay_width) / 2.0 + self.offset_x;
                let overlay_y = 100.0 - camera_y + (map_height - overlay_height) / 2.0 + self.offset_y;
                
                // Draw with transparency
                draw_texture_ex(
                    *texture,
                    overlay_x,
                    overlay_y,
                    Color::new(1.0, 1.0, 1.0, self.alpha),
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(overlay_width, overlay_height)),
                        ..Default::default()
                    }
                );
            }
        }
    }
} 