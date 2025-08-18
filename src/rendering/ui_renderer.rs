// UI rendering functionality

use macroquad::prelude::*;

pub struct UIRenderer;

impl UIRenderer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn draw(
        &self, 
        zoom_level: f32, 
        overlay_visible: bool, 
        overlay_alpha: f32, 
        overlay_offset: (f32, f32), 
        has_overlay: bool
    ) {
        let control_color = Color::new(0.2, 0.2, 0.2, 1.0); // Dark gray
        
        // Draw title
        let title = "PANGAEA";
        let title_size = 48.0;
        let title_color = Color::new(0.2, 0.2, 0.2, 1.0);
        
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
        let control_size = 20.0;
        
        // White background for controls
        draw_rectangle(
            10.0,
            screen_height() - 220.0,
            260.0,
            210.0,
            Color::new(1.0, 1.0, 1.0, 0.8)
        );
        
        draw_text("CLICK+DRAG: PAN", 20.0, screen_height() - 190.0, control_size, control_color);
        draw_text("ARROW KEYS: PAN", 20.0, screen_height() - 165.0, control_size, control_color);
        draw_text("+/-: ZOOM IN/OUT", 20.0, screen_height() - 140.0, control_size, control_color);
        draw_text("0: RESET ZOOM", 20.0, screen_height() - 115.0, control_size, control_color);
        draw_text("O: TOGGLE OVERLAY", 20.0, screen_height() - 90.0, control_size, control_color);
        draw_text("[/]: OPACITY", 20.0, screen_height() - 65.0, control_size, control_color);
        draw_text("WASD: SHIFT OVERLAY", 20.0, screen_height() - 40.0, control_size, control_color);
        draw_text("ESC: EXIT", 20.0, screen_height() - 15.0, control_size, control_color);
        
        // Show zoom level
        let zoom_text = format!("Zoom: {:.0}%", zoom_level * 100.0);
        draw_text(&zoom_text, screen_width() - 200.0, 30.0, 20.0, control_color);
        
        // Show overlay status
        if has_overlay {
            let status = if overlay_visible {
                format!("Overlay: ON ({:.0}%)", overlay_alpha * 100.0)
            } else {
                "Overlay: OFF".to_string()
            };
            draw_text(&status, screen_width() - 200.0, 55.0, 20.0, control_color);
            
            // Show offset if overlay is visible
            if overlay_visible {
                let offset_text = format!("Offset: ({:.0}, {:.0})", overlay_offset.0, overlay_offset.1);
                draw_text(&offset_text, screen_width() - 200.0, 80.0, 16.0, control_color);
            }
        }
    }
} 