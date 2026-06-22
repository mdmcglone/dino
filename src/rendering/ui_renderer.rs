// UI rendering functionality

use macroquad::prelude::*;

pub struct UIRenderer;

impl UIRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_ui(
        &self,
        zoom_level: f32,
        show_controls: bool,
        team_label: &str,
        population: usize,
        population_cap: usize,
        nestless_seconds_left: Option<f64>,
    ) {
        let control_color = Color::new(0.2, 0.2, 0.2, 1.0);
        let section_color = Color::new(0.45, 0.45, 0.45, 1.0);
        let control_size = 16.0;
        let section_size = 14.0;
        let line_height = 20.0;

        if show_controls {
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
                ("P: PLACE NEST (1, 5, 10...)", false),
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
                ("Z: TOGGLE CONTROLS", false),
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
        }

        let zoom_text = format!("Zoom: {:.0}%", zoom_level * 100.0);
        draw_text(&zoom_text, screen_width() - 200.0, 30.0, 20.0, control_color);

        let pop_text = format!("{} Population: {}/{}", team_label, population, population_cap);
        draw_text(&pop_text, screen_width() - 280.0, 55.0, 20.0, control_color);

        if let Some(seconds_left) = nestless_seconds_left {
            let warning = format!("NO NEST! Eliminated in {:.0}s", seconds_left.ceil());
            draw_text(
                &warning,
                screen_width() - 320.0,
                80.0,
                20.0,
                Color::new(0.85, 0.15, 0.15, 1.0),
            );
        }
    }

    pub fn draw_game_over(&self, winner_label: Option<&str>, draw: bool) {
        if winner_label.is_none() && !draw {
            return;
        }

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.55),
        );

        let title = if draw {
            "Draw - no teams remain".to_string()
        } else {
            format!("{} wins!", winner_label.unwrap())
        };

        let font_size = 48.0;
        let text_width = measure_text(&title, None, font_size as u16, 1.0).width;
        draw_text(
            &title,
            screen_width() / 2.0 - text_width / 2.0,
            screen_height() / 2.0 - 10.0,
            font_size,
            Color::new(1.0, 0.95, 0.85, 1.0),
        );

        let subtitle = "Click anywhere or press Q to restart";
        let sub_size = 22.0;
        let sub_width = measure_text(subtitle, None, sub_size as u16, 1.0).width;
        draw_text(
            subtitle,
            screen_width() / 2.0 - sub_width / 2.0,
            screen_height() / 2.0 + 40.0,
            sub_size,
            Color::new(0.9, 0.9, 0.9, 1.0),
        );
    }
}
