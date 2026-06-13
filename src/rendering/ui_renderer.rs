// UI rendering functionality

use macroquad::prelude::*;

pub struct UIRenderer;

impl UIRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, zoom_level: f32) {
        let control_color = Color::new(0.2, 0.2, 0.2, 1.0);
        let section_color = Color::new(0.45, 0.45, 0.45, 1.0);
        let control_size = 16.0;
        let section_size = 14.0;
        let line_height = 20.0;

        let title = "PANGAEA";
        let title_size = 48.0;
        let title_color = Color::new(0.2, 0.2, 0.2, 1.0);
        let title_width = title.len() as f32 * title_size * 0.6;
        let title_x = (screen_width() - title_width) / 2.0;
        let title_y = 40.0;

        draw_rectangle(
            title_x - 20.0,
            title_y - title_size + 10.0,
            title_width + 40.0,
            title_size + 10.0,
            Color::new(1.0, 1.0, 1.0, 0.8),
        );
        draw_text(title, title_x, title_y, title_size, title_color);

        let panel_width = 340.0;
        let panel_height = 360.0;
        let panel_x = 10.0;
        let panel_y = screen_height() - panel_height - 10.0;

        draw_rectangle(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Color::new(1.0, 1.0, 1.0, 0.8),
        );

        let text_x = panel_x + 10.0;
        let mut y = panel_y + 24.0;

        let lines: &[(&str, bool)] = &[
            ("MOUSE", true),
            ("LEFT CLICK: SELECT / MOVE", false),
            ("RIGHT DRAG: BOX SELECT", false),
            ("SHIFT+CLICK: QUEUE WAYPOINTS", false),
            ("TEAMS", true),
            ("SPACE: CYCLE TEAM", false),
            ("P: PLACE NEST (10+ STACK)", false),
            ("STACK (WHEN SELECTED)", true),
            ("E: SELECT HALF", false),
            ("R: SELECT ONE", false),
            ("1-9: SELECT COUNT", false),
            ("DEBUG", true),
            ("Q: RESET UNITS", false),
            ("CAMERA", true),
            ("ARROWS / WASD: PAN", false),
            ("+/-: ZOOM", false),
            ("0: RESET ZOOM", false),
            ("OTHER", true),
            ("ESC: EXIT", false),
        ];

        for (label, is_section) in lines {
            let (size, color) = if *is_section {
                (section_size, section_color)
            } else {
                (control_size, control_color)
            };
            draw_text(label, text_x, y, size, color);
            y += line_height;
        }

        let zoom_text = format!("Zoom: {:.0}%", zoom_level * 100.0);
        draw_text(&zoom_text, screen_width() - 200.0, 30.0, 20.0, control_color);
    }
}
