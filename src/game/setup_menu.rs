use macroquad::prelude::*;
use crate::maps::MapKind;
use super::team_setup::{
    self, MatchConfig, TeamSetup, MAX_TEAMS, MIN_TEAMS, PLAYER_DINO_COUNT,
};

const PANEL_X: f32 = 80.0;
const PANEL_W: f32 = 1240.0;
const ROW_HEIGHT: f32 = 88.0;
const ROW_START_Y: f32 = 248.0;
const BTN_H: f32 = 44.0;
const TEAM_ROW_Y: f32 = 132.0;
const MAP_ROW_Y: f32 = 188.0;

const TEAM_COUNT_OPTIONS: usize = MAX_TEAMS - MIN_TEAMS + 1;

struct Layout {
    back: Rect,
    team_minus: Rect,
    team_plus: Rect,
    team_count_buttons: [Rect; TEAM_COUNT_OPTIONS],
    map_pangaea: Rect,
    map_random: Rect,
    start: Rect,
    rows: Vec<RowLayout>,
    color_grid: Option<ColorGridLayout>,
}

struct RowLayout {
    dino_prev: Rect,
    dino_next: Rect,
    color_swatch: Rect,
    dino_chips: [Rect; PLAYER_DINO_COUNT],
}

struct ColorGridLayout {
    cells: Vec<(Rect, Color)>,
    cancel: Rect,
}

#[derive(Clone, Copy)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn contains(&self, mx: f32, my: f32) -> bool {
        mx >= self.x && mx <= self.x + self.w && my >= self.y && my <= self.y + self.h
    }
}

pub struct SetupMenu {
    num_teams: usize,
    teams: Vec<TeamSetup>,
    map_kind: MapKind,
    color_picker_for: Option<usize>,
}

#[derive(Clone)]
pub enum SetupAction {
    None,
    Back,
    Start(MatchConfig),
}

impl SetupMenu {
    pub fn new() -> Self {
        let num_teams = MIN_TEAMS;
        Self {
            num_teams,
            teams: (0..num_teams).map(TeamSetup::default_for_slot).collect(),
            map_kind: MapKind::Pangaea,
            color_picker_for: None,
        }
    }

    pub fn update(&mut self) -> SetupAction {
        if is_key_pressed(KeyCode::Escape) {
            return if self.color_picker_for.is_some() {
                self.color_picker_for = None;
                SetupAction::None
            } else {
                SetupAction::Back
            };
        }

        let layout = self.build_layout();

        if !is_mouse_button_pressed(MouseButton::Left) {
            return SetupAction::None;
        }

        let (mx, my) = mouse_position();

        if let Some(grid) = &layout.color_grid {
            for (rect, color) in &grid.cells {
                if rect.contains(mx, my) {
                    if let Some(team_index) = self.color_picker_for {
                        self.teams[team_index].color = *color;
                    }
                    self.color_picker_for = None;
                    return SetupAction::None;
                }
            }
            if grid.cancel.contains(mx, my) {
                self.color_picker_for = None;
            }
            return SetupAction::None;
        }

        if layout.back.contains(mx, my) {
            return SetupAction::Back;
        }
        if layout.start.contains(mx, my) {
            return SetupAction::Start(MatchConfig {
                teams: self.teams[..self.num_teams].to_vec(),
                map_kind: self.map_kind,
            });
        }
        if layout.team_minus.contains(mx, my) && self.num_teams > MIN_TEAMS {
            self.num_teams -= 1;
            self.sync_team_rows();
            return SetupAction::None;
        }
        if layout.team_plus.contains(mx, my) && self.num_teams < MAX_TEAMS {
            self.num_teams += 1;
            self.sync_team_rows();
            return SetupAction::None;
        }
        for (index, rect) in layout.team_count_buttons.iter().enumerate() {
            let count = index + MIN_TEAMS;
            if rect.contains(mx, my) && count >= MIN_TEAMS && count <= MAX_TEAMS {
                self.num_teams = count;
                self.sync_team_rows();
                return SetupAction::None;
            }
        }

        if layout.map_pangaea.contains(mx, my) {
            self.map_kind = MapKind::Pangaea;
            return SetupAction::None;
        }
        if layout.map_random.contains(mx, my) {
            self.map_kind = MapKind::Random;
            return SetupAction::None;
        }

        for (team_index, row) in layout.rows.iter().enumerate() {
            if row.dino_prev.contains(mx, my) {
                self.teams[team_index].dino_type =
                    team_setup::cycle_dino(self.teams[team_index].dino_type, -1);
                return SetupAction::None;
            }
            if row.dino_next.contains(mx, my) {
                self.teams[team_index].dino_type =
                    team_setup::cycle_dino(self.teams[team_index].dino_type, 1);
                return SetupAction::None;
            }
            if row.color_swatch.contains(mx, my) {
                self.color_picker_for = Some(team_index);
                return SetupAction::None;
            }
            for (dino, chip) in row.dino_chips.iter().enumerate() {
                if chip.contains(mx, my) {
                    self.teams[team_index].dino_type = dino;
                    return SetupAction::None;
                }
            }
        }

        SetupAction::None
    }

    pub fn draw(&self) {
        clear_background(Color::new(0.08, 0.1, 0.14, 1.0));
        let layout = self.build_layout();
        let (mx, my) = mouse_position();

        draw_text(
            "Match Setup",
            PANEL_X,
            64.0,
            48.0,
            Color::new(0.95, 0.92, 0.85, 1.0),
        );
        draw_text(
            "Set team count, map type, pick a dino, then click the color square to choose a team color.",
            PANEL_X,
            100.0,
            20.0,
            Color::new(0.7, 0.75, 0.82, 1.0),
        );

        draw_button(
            "Back",
            &layout.back,
            layout.back.contains(mx, my),
            false,
        );

        draw_text(
            "Number of teams",
            PANEL_X,
            TEAM_ROW_Y + 6.0,
            22.0,
            Color::new(0.85, 0.88, 0.92, 1.0),
        );
        draw_button("-", &layout.team_minus, layout.team_minus.contains(mx, my), false);
        draw_button("+", &layout.team_plus, layout.team_plus.contains(mx, my), false);

        for (index, rect) in layout.team_count_buttons.iter().enumerate() {
            let count = index + MIN_TEAMS;
            let selected = count == self.num_teams;
            let hover = rect.contains(mx, my);
            draw_count_chip(count, rect, selected, hover);
        }

        draw_text(
            "Map type",
            PANEL_X,
            MAP_ROW_Y + 6.0,
            22.0,
            Color::new(0.85, 0.88, 0.92, 1.0),
        );
        draw_map_chip(
            MapKind::Pangaea,
            &layout.map_pangaea,
            self.map_kind == MapKind::Pangaea,
            layout.map_pangaea.contains(mx, my),
        );
        draw_map_chip(
            MapKind::Random,
            &layout.map_random,
            self.map_kind == MapKind::Random,
            layout.map_random.contains(mx, my),
        );

        draw_rectangle(
            PANEL_X,
            ROW_START_Y - 20.0,
            PANEL_W,
            self.num_teams as f32 * ROW_HEIGHT + 12.0,
            Color::new(0.12, 0.14, 0.18, 0.95),
        );
        draw_rectangle_lines(
            PANEL_X,
            ROW_START_Y - 20.0,
            PANEL_W,
            self.num_teams as f32 * ROW_HEIGHT + 12.0,
            2.0,
            Color::new(0.35, 0.4, 0.48, 1.0),
        );

        draw_text("Team", PANEL_X + 16.0, ROW_START_Y, 18.0, label_color());
        draw_text("Dino", PANEL_X + 160.0, ROW_START_Y, 18.0, label_color());
        draw_text("Color (click)", PANEL_X + 470.0, ROW_START_Y, 18.0, label_color());
        draw_text("Or pick dino", PANEL_X + 590.0, ROW_START_Y, 18.0, label_color());

        for (team_index, row) in layout.rows.iter().enumerate() {
            let row_y = ROW_START_Y + 24.0 + team_index as f32 * ROW_HEIGHT;
            draw_text(
                &format!("Team {}", team_index + 1),
                PANEL_X + 16.0,
                row_y + 34.0,
                24.0,
                Color::new(0.92, 0.92, 0.92, 1.0),
            );

            draw_button(
                "<",
                &row.dino_prev,
                row.dino_prev.contains(mx, my),
                false,
            );
            let dino_name = team_setup::player_dino_name(self.teams[team_index].dino_type);
            draw_text(
                dino_name,
                PANEL_X + 210.0,
                row_y + 34.0,
                22.0,
                Color::new(0.88, 0.9, 0.95, 1.0),
            );
            draw_button(
                ">",
                &row.dino_next,
                row.dino_next.contains(mx, my),
                false,
            );

            draw_color_swatch(
                self.teams[team_index].color,
                &row.color_swatch,
                row.color_swatch.contains(mx, my),
            );

            for (dino, chip) in row.dino_chips.iter().enumerate() {
                draw_dino_chip(
                    dino,
                    self.teams[team_index].dino_type == dino,
                    chip,
                    chip.contains(mx, my),
                );
            }
        }

        draw_button(
            "Start Game",
            &layout.start,
            layout.start.contains(mx, my),
            true,
        );

        if let Some(grid) = &layout.color_grid {
            self.draw_color_picker_overlay(grid, mx, my);
        }
    }

    fn build_layout(&self) -> Layout {
        let back = Rect {
            x: 40.0,
            y: 40.0,
            w: 120.0,
            h: BTN_H,
        };
        let team_minus = Rect {
            x: PANEL_X + 200.0,
            y: TEAM_ROW_Y,
            w: BTN_H,
            h: BTN_H,
        };
        let team_plus = Rect {
            x: PANEL_X + 260.0,
            y: TEAM_ROW_Y,
            w: BTN_H,
            h: BTN_H,
        };
        let mut team_count_buttons = [Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }; TEAM_COUNT_OPTIONS];
        for index in 0..TEAM_COUNT_OPTIONS {
            team_count_buttons[index] = Rect {
                x: PANEL_X + 340.0 + index as f32 * 72.0,
                y: TEAM_ROW_Y,
                w: 60.0,
                h: BTN_H,
            };
        }
        let map_pangaea = Rect {
            x: PANEL_X + 200.0,
            y: MAP_ROW_Y,
            w: 140.0,
            h: BTN_H,
        };
        let map_random = Rect {
            x: PANEL_X + 360.0,
            y: MAP_ROW_Y,
            w: 140.0,
            h: BTN_H,
        };
        let start = Rect {
            x: screen_width() / 2.0 - 130.0,
            y: screen_height() - 88.0,
            w: 260.0,
            h: 52.0,
        };

        let mut rows = Vec::with_capacity(self.num_teams);
        for team_index in 0..self.num_teams {
            let row_y = ROW_START_Y + 24.0 + team_index as f32 * ROW_HEIGHT;
            let mut dino_chips = [Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }; PLAYER_DINO_COUNT];
            for dino in 0..PLAYER_DINO_COUNT {
                dino_chips[dino] = Rect {
                    x: PANEL_X + 590.0 + dino as f32 * 112.0,
                    y: row_y + 8.0,
                    w: 104.0,
                    h: BTN_H,
                };
            }
            rows.push(RowLayout {
                dino_prev: Rect {
                    x: PANEL_X + 160.0,
                    y: row_y + 8.0,
                    w: BTN_H,
                    h: BTN_H,
                },
                dino_next: Rect {
                    x: PANEL_X + 360.0,
                    y: row_y + 8.0,
                    w: BTN_H,
                    h: BTN_H,
                },
                color_swatch: Rect {
                    x: PANEL_X + 470.0,
                    y: row_y + 8.0,
                    w: BTN_H,
                    h: BTN_H,
                },
                dino_chips,
            });
        }

        let color_grid = self.color_picker_for.map(|_team_index| {
            let palette = team_setup::color_palette();
            let cols = 8;
            let cell = 46.0;
            let gap = 10.0;
            let grid_w = cols as f32 * cell + (cols - 1) as f32 * gap;
            let rows = palette.len().div_ceil(cols);
            let grid_h = rows as f32 * cell + (rows - 1) as f32 * gap;
            let grid_x = screen_width() / 2.0 - grid_w / 2.0;
            let grid_y = screen_height() / 2.0 - grid_h / 2.0;

            let cells = palette
                .iter()
                .enumerate()
                .map(|(index, color)| {
                    let col = index % cols;
                    let row = index / cols;
                    (
                        Rect {
                            x: grid_x + col as f32 * (cell + gap),
                            y: grid_y + row as f32 * (cell + gap),
                            w: cell,
                            h: cell,
                        },
                        *color,
                    )
                })
                .collect();

            let cancel = Rect {
                x: screen_width() / 2.0 - 70.0,
                y: grid_y + grid_h + 24.0,
                w: 140.0,
                h: BTN_H,
            };

            ColorGridLayout { cells, cancel }
        });

        Layout {
            back,
            team_minus,
            team_plus,
            team_count_buttons,
            map_pangaea,
            map_random,
            start,
            rows,
            color_grid,
        }
    }

    fn draw_color_picker_overlay(&self, grid: &ColorGridLayout, mx: f32, my: f32) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.6),
        );

        if let Some(team_index) = self.color_picker_for {
            let title = format!("Pick a color for Team {}", team_index + 1);
            let title_width = measure_text(&title, None, 28, 1.0).width;
            draw_text(
                &title,
                screen_width() / 2.0 - title_width / 2.0,
                grid.cells[0].0.y - 48.0,
                28.0,
                Color::new(0.95, 0.95, 0.95, 1.0),
            );
        }

        for (rect, color) in &grid.cells {
            draw_color_swatch(*color, rect, rect.contains(mx, my));
        }

        draw_button(
            "Cancel",
            &grid.cancel,
            grid.cancel.contains(mx, my),
            false,
        );
    }

    fn sync_team_rows(&mut self) {
        while self.teams.len() < self.num_teams {
            self.teams
                .push(TeamSetup::default_for_slot(self.teams.len()));
        }
        self.teams.truncate(self.num_teams);
    }
}

fn label_color() -> Color {
    Color::new(0.55, 0.6, 0.68, 1.0)
}

fn draw_button(label: &str, rect: &Rect, hover: bool, primary: bool) {
    let bg = if primary {
        if hover {
            Color::new(0.28, 0.58, 0.38, 1.0)
        } else {
            Color::new(0.2, 0.48, 0.32, 1.0)
        }
    } else if hover {
        Color::new(0.28, 0.42, 0.62, 1.0)
    } else {
        Color::new(0.2, 0.24, 0.3, 1.0)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        Color::new(0.55, 0.62, 0.72, 1.0),
    );
    let font_size = if primary { 24.0 } else { 20.0 };
    let text_width = measure_text(label, None, font_size as u16, 1.0).width;
    draw_text(
        label,
        rect.x + rect.w / 2.0 - text_width / 2.0,
        rect.y + rect.h / 2.0 + font_size * 0.35,
        font_size,
        Color::new(0.95, 0.95, 0.95, 1.0),
    );
}

fn draw_map_chip(kind: MapKind, rect: &Rect, selected: bool, hover: bool) {
    let bg = if selected {
        Color::new(0.25, 0.45, 0.7, 1.0)
    } else if hover {
        Color::new(0.22, 0.28, 0.36, 1.0)
    } else {
        Color::new(0.16, 0.18, 0.22, 1.0)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        if selected {
            Color::new(0.85, 0.9, 1.0, 1.0)
        } else {
            Color::new(0.4, 0.45, 0.52, 1.0)
        },
    );
    let label = kind.label();
    let font_size = 20.0;
    let text_width = measure_text(label, None, font_size as u16, 1.0).width;
    draw_text(
        label,
        rect.x + rect.w / 2.0 - text_width / 2.0,
        rect.y + rect.h / 2.0 + font_size * 0.35,
        font_size,
        Color::new(0.95, 0.95, 0.95, 1.0),
    );
}

fn draw_count_chip(count: usize, rect: &Rect, selected: bool, hover: bool) {
    let bg = if selected {
        Color::new(0.25, 0.45, 0.7, 1.0)
    } else if hover {
        Color::new(0.22, 0.28, 0.36, 1.0)
    } else {
        Color::new(0.16, 0.18, 0.22, 1.0)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        if selected {
            Color::new(0.85, 0.9, 1.0, 1.0)
        } else {
            Color::new(0.4, 0.45, 0.52, 1.0)
        },
    );
    let label = count.to_string();
    let font_size = 22.0;
    let text_width = measure_text(&label, None, font_size as u16, 1.0).width;
    draw_text(
        &label,
        rect.x + rect.w / 2.0 - text_width / 2.0,
        rect.y + rect.h / 2.0 + font_size * 0.35,
        font_size,
        Color::new(0.95, 0.95, 0.95, 1.0),
    );
}

fn draw_color_swatch(color: Color, rect: &Rect, hover: bool) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
    let border = if hover {
        Color::new(1.0, 1.0, 1.0, 1.0)
    } else {
        Color::new(0.25, 0.25, 0.25, 1.0)
    };
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if hover { 3.0 } else { 2.0 },
        border,
    );
}

fn draw_dino_chip(dino: usize, selected: bool, rect: &Rect, hover: bool) {
    let bg = if selected {
        Color::new(0.25, 0.45, 0.7, 1.0)
    } else if hover {
        Color::new(0.22, 0.28, 0.36, 1.0)
    } else {
        Color::new(0.16, 0.18, 0.22, 1.0)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        if selected {
            Color::new(0.85, 0.9, 1.0, 1.0)
        } else {
            Color::new(0.4, 0.45, 0.52, 1.0)
        },
    );
    let label = team_setup::player_dino_name(dino);
    let font_size = 16.0;
    let text_width = measure_text(label, None, font_size as u16, 1.0).width;
    draw_text(
        label,
        rect.x + rect.w / 2.0 - text_width / 2.0,
        rect.y + rect.h / 2.0 + font_size * 0.35,
        font_size,
        Color::new(0.92, 0.92, 0.92, 1.0),
    );
}
