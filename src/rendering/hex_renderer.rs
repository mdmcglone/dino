// Core hex map rendering functionality

use macroquad::prelude::*;
use std::collections::HashSet;
use crate::core::HexCoord;
use crate::game::Nest;
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
            team_sprites: vec![None; 5],
        }
    }

    pub async fn load_team_sprite(&mut self, team: usize, path: &str) {
        match load_texture(path).await {
            Ok(texture) => {
                texture.set_filter(FilterMode::Nearest); // Crisp pixel art
                if team >= self.team_sprites.len() {
                    self.team_sprites.resize(team + 1, None);
                }
                self.team_sprites[team] = Some(texture);
                println!("Team {} sprite loaded successfully", team);
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

    pub fn center_camera_on(&mut self, coord: &HexCoord) {
        let sqrt3 = 3.0_f32.sqrt();
        let x = self.hex_size * 3.0 / 2.0 * coord.q as f32;
        let y_offset = if coord.q % 2 == 1 {
            self.hex_size * sqrt3 / 2.0
        } else {
            0.0
        };
        let y = self.hex_size * sqrt3 * coord.r as f32 + y_offset;
        self.camera_x = x + 100.0 - screen_width() / 2.0;
        self.camera_y = y + 100.0 - screen_height() / 2.0;
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

    fn hex_on_screen(&self, center_x: f32, center_y: f32) -> bool {
        let margin = self.hex_size * 2.0;
        center_x >= -margin
            && center_x <= screen_width() + margin
            && center_y >= -margin
            && center_y <= screen_height() + margin
    }

    fn hex_vertices(center_x: f32, center_y: f32, hex_size: f32) -> [Vec2; 6] {
        let mut vertices = [Vec2::ZERO; 6];
        for i in 0..6 {
            let angle = std::f32::consts::PI / 3.0 * i as f32;
            vertices[i] = Vec2::new(
                center_x + hex_size * angle.cos(),
                center_y + hex_size * angle.sin(),
            );
        }
        vertices
    }
    
    pub fn draw_hex(&self, coord: &HexCoord, terrain_type: TerrainType) {
        let (center_x, center_y) = self.hex_to_pixel(coord);
        if !self.hex_on_screen(center_x, center_y) {
            return;
        }
        self.draw_hex_at(center_x, center_y, terrain_type);
    }
    
    pub fn draw_map_with_fog(&self, map: &dyn Map, visible: &HashSet<HexCoord>) {
        for (coord, terrain) in map.get_tiles() {
            let (center_x, center_y) = self.hex_to_pixel(coord);
            if !self.hex_on_screen(center_x, center_y) {
                continue;
            }
            self.draw_hex_at(center_x, center_y, *terrain);
            if !visible.contains(coord) {
                self.draw_hex_fog_overlay_at(center_x, center_y);
            }
        }
    }

    fn draw_hex_at(&self, center_x: f32, center_y: f32, terrain_type: TerrainType) {
        let vertices = Self::hex_vertices(center_x, center_y, self.hex_size);
        let center = Vec2::new(center_x, center_y);
        let color = terrain_type.color();
        for i in 1..5 {
            draw_triangle(center, vertices[i], vertices[i + 1], color);
        }
        draw_triangle(center, vertices[5], vertices[0], color);
        draw_triangle(center, vertices[0], vertices[1], color);

        let border_color = terrain_type.border_color();
        for i in 0..6 {
            let next = (i + 1) % 6;
            draw_line(
                vertices[i].x, vertices[i].y,
                vertices[next].x, vertices[next].y,
                1.0,
                border_color,
            );
        }
    }

    fn draw_hex_fog_overlay_at(&self, center_x: f32, center_y: f32) {
        let fog_color = Color::new(0.45, 0.45, 0.48, 0.55);
        let vertices = Self::hex_vertices(center_x, center_y, self.hex_size);
        let center = Vec2::new(center_x, center_y);
        for i in 1..5 {
            draw_triangle(center, vertices[i], vertices[i + 1], fog_color);
        }
        draw_triangle(center, vertices[5], vertices[0], fog_color);
        draw_triangle(center, vertices[0], vertices[1], fog_color);
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
    
    pub fn draw_ui(
        &self,
        show_controls: bool,
        current_team: usize,
        population: usize,
        population_cap: usize,
        nestless_seconds_left: Option<f64>,
    ) {
        self.ui_renderer.draw_ui(
            self.zoom_level,
            show_controls,
            current_team,
            population,
            population_cap,
            nestless_seconds_left,
        );
    }

    pub fn draw_game_over(&self, winner: Option<usize>, draw: bool) {
        self.ui_renderer.draw_game_over(winner, draw);
    }

    fn nest_team_color(team: usize) -> Color {
        match team {
            0 => Color::new(0.85, 0.55, 0.1, 0.95),   // T-Rex — gold
            1 => Color::new(0.15, 0.65, 0.55, 0.95),  // Bronto — teal
            2 => Color::new(0.45, 0.55, 0.95, 0.95),  // Ptero — sky blue
            3 => Color::new(0.75, 0.25, 0.55, 0.95),  // Tricera — rose
            4 => Color::new(0.35, 0.15, 0.45, 0.95),  // Krono — deep purple
            _ => Color::new(0.6, 0.6, 0.6, 0.95),
        }
    }

    pub fn draw_nest_farm_zone(&self, nest: &Nest, visible: &HashSet<HexCoord>) {
        let team_color = Self::nest_team_color(nest.team);
        let border_color = Color::new(team_color.r, team_color.g, team_color.b, 1.0);
        let thickness = 5.0 * self.zoom_level;
        let farm_within = nest.farm_within();

        for coord in nest.farm_within() {
            if !visible.contains(coord) {
                continue;
            }
            let (center_x, center_y) = self.hex_to_pixel(coord);
            if center_x < -self.hex_size * 2.0 || center_x > screen_width() + self.hex_size * 2.0
                || center_y < -self.hex_size * 2.0 || center_y > screen_height() + self.hex_size * 2.0
            {
                continue;
            }

            let mut vertices = [Vec2::ZERO; 6];
            for i in 0..6 {
                let angle = std::f32::consts::PI / 3.0 * i as f32;
                vertices[i] = Vec2::new(
                    center_x + self.hex_size * angle.cos(),
                    center_y + self.hex_size * angle.sin(),
                );
            }

            for neighbor in coord.offset_neighbors() {
                if farm_within.contains(&neighbor) {
                    continue;
                }

                let (neighbor_x, neighbor_y) = self.hex_to_pixel(&neighbor);
                let edge_mid_x = (center_x + neighbor_x) / 2.0;
                let edge_mid_y = (center_y + neighbor_y) / 2.0;

                let mut best_edge = 0;
                let mut best_dist_sq = f32::MAX;
                for i in 0..6 {
                    let next = (i + 1) % 6;
                    let mid_x = (vertices[i].x + vertices[next].x) / 2.0;
                    let mid_y = (vertices[i].y + vertices[next].y) / 2.0;
                    let dx = mid_x - edge_mid_x;
                    let dy = mid_y - edge_mid_y;
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq < best_dist_sq {
                        best_dist_sq = dist_sq;
                        best_edge = i;
                    }
                }

                let next = (best_edge + 1) % 6;
                draw_line(
                    vertices[best_edge].x, vertices[best_edge].y,
                    vertices[next].x, vertices[next].y,
                    thickness,
                    border_color,
                );
            }
        }
    }

    pub fn draw_nest_food_bar(&self, nest: &Nest, food_cap: f32) {
        let (center_x, center_y) = self.hex_to_pixel(&nest.position);
        if center_x < -self.hex_size * 2.0 || center_x > screen_width() + self.hex_size * 2.0
            || center_y < -self.hex_size * 2.0 || center_y > screen_height() + self.hex_size * 2.0
        {
            return;
        }

        let bar_width = self.hex_size * 1.4;
        let bar_height = 7.0 * self.zoom_level;
        let bar_x = center_x - bar_width / 2.0;
        let bar_y = center_y + self.hex_size * 0.55;
        let progress = (nest.food / food_cap).clamp(0.0, 1.0);
        let fill_color = Self::nest_team_color(nest.team);

        draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(0.15, 0.15, 0.15, 0.75));
        if progress > 0.0 {
            draw_rectangle(bar_x, bar_y, bar_width * progress, bar_height, fill_color);
        }
        draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 1.5, Color::new(1.0, 1.0, 1.0, 0.85));
    }
    
    pub fn draw_nest_siege_bar(&self, nest: &Nest) {
        let (center_x, center_y) = self.hex_to_pixel(&nest.position);
        if center_x < -self.hex_size * 2.0 || center_x > screen_width() + self.hex_size * 2.0
            || center_y < -self.hex_size * 2.0 || center_y > screen_height() + self.hex_size * 2.0
        {
            return;
        }

        let bar_width = self.hex_size * 1.4;
        let bar_height = 7.0 * self.zoom_level;
        let bar_x = center_x - bar_width / 2.0;
        let bar_y = center_y + self.hex_size * 0.55;
        let progress = (nest.siege_progress / crate::game::nest::SIEGE_DINO_SECONDS_TARGET).clamp(0.0, 1.0);
        let fill_color = nest
            .siege_team
            .map(Self::nest_team_color)
            .unwrap_or(Color::new(0.8, 0.2, 0.2, 0.95));

        draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(0.15, 0.15, 0.15, 0.75));
        if progress > 0.0 {
            draw_rectangle(bar_x, bar_y, bar_width * progress, bar_height, fill_color);
        }
        draw_rectangle_lines(bar_x, bar_y, bar_width, bar_height, 1.5, Color::new(1.0, 0.85, 0.3, 0.9));
    }

    pub fn draw_player(&self, coord: &HexCoord, team: usize) {
        self.draw_player_with_offset(coord, team, 0.0, false);
    }
    
    pub fn draw_player_with_offset(&self, coord: &HexCoord, team: usize, offset_factor: f32, flip_x: bool) {
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
                    flip_x,
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
        let vertices = Self::hex_vertices(center_x, center_y, self.hex_size);
        
        // Draw thick yellow border for selection
        let highlight_color = Color::new(1.0, 1.0, 0.0, 0.8); // Yellow
        for i in 0..6 {
            let next = (i + 1) % 6;
            draw_line(
                vertices[i].x, vertices[i].y,
                vertices[next].x, vertices[next].y,
                3.0,
                highlight_color
            );
        }
    }
    
    fn draw_dashed_segment(&self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();
        if length < 1.0 {
            return;
        }
        let dir_x = dx / length;
        let dir_y = dy / length;
        let z = self.zoom_level;
        let dash = 7.0 * z;
        let gap = 5.0 * z;
        let mut dist = 0.0;
        while dist < length {
            let seg_start = dist;
            let seg_end = (dist + dash).min(length);
            draw_line(
                x1 + dir_x * seg_start,
                y1 + dir_y * seg_start,
                x1 + dir_x * seg_end,
                y1 + dir_y * seg_end,
                2.0 * z,
                color,
            );
            dist += dash + gap;
        }
    }

    fn draw_path_node(&self, coord: &HexCoord, is_final: bool) {
        let (x, y) = self.hex_to_pixel(coord);
        let z = self.zoom_level;
        let dest_ring = Color::new(0.25, 0.62, 0.98, 0.75);
        let dot_color = Color::new(0.35, 0.78, 1.0, 0.55);

        if is_final {
            let dest_radius = self.hex_size * 0.2;
            draw_circle(x, y, dest_radius + 2.0 * z, Color::new(0.2, 0.55, 0.95, 0.18));
            draw_circle_lines(x, y, dest_radius, 2.0 * z, dest_ring);
            draw_circle(x, y, 3.5 * z, dot_color);
        } else {
            draw_circle(x, y, 5.0 * z, Color::new(0.35, 0.78, 1.0, 0.18));
            draw_circle(x, y, 3.0 * z, dot_color);
        }
    }

    fn draw_active_leg(&self, from: &HexCoord, to: &HexCoord, progress: f32) {
        let (from_x, from_y) = self.hex_to_pixel(from);
        let (to_x, to_y) = self.hex_to_pixel(to);

        let dx = to_x - from_x;
        let dy = to_y - from_y;
        let total_length = (dx * dx + dy * dy).sqrt();
        if total_length < 1.0 {
            return;
        }

        let dir_x = dx / total_length;
        let dir_y = dy / total_length;
        let perp_x = -dir_y;
        let perp_y = dir_x;
        let z = self.zoom_level;

        let end_x = from_x + dir_x * total_length * progress;
        let end_y = from_y + dir_y * total_length * progress;

        let outline = Color::new(0.06, 0.14, 0.28, 0.9);
        let shaft_color = Color::new(0.28, 0.72, 1.0, 0.92);
        let highlight = Color::new(0.82, 0.96, 1.0, 0.85);
        let ghost = Color::new(0.45, 0.78, 1.0, 0.32);

        let traveled = total_length * progress;
        let dash = 7.0 * z;
        let gap = 5.0 * z;
        let mut dist = traveled;
        while dist < total_length {
            let seg_start = dist;
            let seg_end = (dist + dash).min(total_length);
            draw_line(
                from_x + dir_x * seg_start,
                from_y + dir_y * seg_start,
                from_x + dir_x * seg_end,
                from_y + dir_y * seg_end,
                2.0 * z,
                ghost,
            );
            dist += dash + gap;
        }

        if progress < 0.02 {
            return;
        }

        let head_len = 13.0 * z;
        let show_head = progress > 0.1;
        let (shaft_end_x, shaft_end_y) = if show_head {
            (end_x - dir_x * head_len, end_y - dir_y * head_len)
        } else {
            (end_x, end_y)
        };

        let shaft = 4.5 * z;
        draw_line(from_x, from_y, shaft_end_x, shaft_end_y, shaft + 2.0 * z, outline);
        draw_line(from_x, from_y, shaft_end_x, shaft_end_y, shaft, shaft_color);
        let stripe_off = 0.85 * z;
        draw_line(
            from_x + perp_x * stripe_off,
            from_y + perp_y * stripe_off,
            shaft_end_x + perp_x * stripe_off,
            shaft_end_y + perp_y * stripe_off,
            1.6 * z,
            highlight,
        );

        if show_head {
            let head_width = 6.5 * z;
            let base_x = shaft_end_x;
            let base_y = shaft_end_y;
            let tip = Vec2::new(end_x, end_y);
            let left = Vec2::new(base_x + perp_x * head_width, base_y + perp_y * head_width);
            let right = Vec2::new(base_x - perp_x * head_width, base_y - perp_y * head_width);
            let outline_tip = Vec2::new(end_x + dir_x * z, end_y + dir_y * z);
            let outline_base_x = base_x - dir_x * z;
            let outline_base_y = base_y - dir_y * z;
            let outline_left = Vec2::new(
                outline_base_x + perp_x * (head_width + z),
                outline_base_y + perp_y * (head_width + z),
            );
            let outline_right = Vec2::new(
                outline_base_x - perp_x * (head_width + z),
                outline_base_y - perp_y * (head_width + z),
            );

            draw_triangle(outline_tip, outline_left, outline_right, outline);
            draw_triangle(tip, left, right, shaft_color);
            draw_line(
                base_x + perp_x * head_width * 0.35,
                base_y + perp_y * head_width * 0.35,
                end_x,
                end_y,
                1.2 * z,
                highlight,
            );
        }
    }

    pub fn draw_movement_path(&self, route: &[HexCoord], active_progress: Option<f32>) {
        if route.len() < 2 {
            return;
        }

        let ghost = Color::new(0.45, 0.78, 1.0, 0.32);
        let first_planned = if active_progress.is_some() { 1 } else { 0 };

        for i in first_planned..route.len() - 1 {
            let (x1, y1) = self.hex_to_pixel(&route[i]);
            let (x2, y2) = self.hex_to_pixel(&route[i + 1]);
            self.draw_dashed_segment(x1, y1, x2, y2, ghost);
        }

        for (index, coord) in route.iter().enumerate().skip(1) {
            self.draw_path_node(coord, index == route.len() - 1);
        }

        if let Some(progress) = active_progress {
            self.draw_active_leg(&route[0], &route[1], progress);
        }
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
    
    pub fn draw_nest(&self, nest: &Nest) {
        let (center_x, center_y) = self.hex_to_pixel(&nest.position);
        let arm_length = self.hex_size * 0.45;
        let thickness = 3.0 * self.zoom_level;
        let color = Self::nest_team_color(nest.team);

        // Six-point asterisk aligned with the hex grid
        for i in 0..6 {
            let angle = std::f32::consts::PI / 3.0 * i as f32;
            let end_x = center_x + arm_length * angle.cos();
            let end_y = center_y + arm_length * angle.sin();
            draw_line(center_x, center_y, end_x, end_y, thickness, color);
        }

        draw_circle(center_x, center_y, 4.0 * self.zoom_level, color);
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