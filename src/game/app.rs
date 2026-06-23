use macroquad::prelude::*;
use super::game_state::GameState;
use super::setup_menu::{SetupAction, SetupMenu};
use super::team_abilities;

pub enum AppScreen {
    Launcher,
    Setup(SetupMenu),
    Playing(GameState),
}

pub struct App {
    screen: AppScreen,
}

struct LauncherLayout {
    normal: Rect,
    debug: Rect,
}

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

impl App {
    pub fn new_debug() -> Self {
        Self {
            screen: AppScreen::Playing(GameState::new_debug()),
        }
    }

    pub fn new_normal() -> Self {
        Self {
            screen: AppScreen::Launcher,
        }
    }

    pub async fn load_sprites(game: &mut GameState) {
        game.load_team_sprite(team_abilities::TREX_TEAM, "sprites/trex_clear.png")
            .await;
        game.load_team_sprite(team_abilities::BRONTO_TEAM, "sprites/bronto_clear.png")
            .await;
        game.load_team_sprite(team_abilities::PTERO_TEAM, "sprites/ptero_clear.png")
            .await;
        game.load_team_sprite(team_abilities::TRICERA_TEAM, "sprites/tricera_clear.png")
            .await;
        game.load_team_sprite(team_abilities::KRONO_TEAM, "sprites/krono.png")
            .await;
        game.mark_sprites_loaded();
    }

    pub fn screen_mut(&mut self) -> &mut AppScreen {
        &mut self.screen
    }

    pub fn update(&mut self) -> bool {
        match &mut self.screen {
            AppScreen::Launcher => {
                if is_key_pressed(KeyCode::Escape) {
                    return true;
                }
                if is_mouse_button_pressed(MouseButton::Left) {
                    let layout = launcher_layout();
                    let (mx, my) = mouse_position();
                    if layout.normal.contains(mx, my) {
                        self.screen = AppScreen::Setup(SetupMenu::new());
                    } else if layout.debug.contains(mx, my) {
                        self.screen = AppScreen::Playing(GameState::new_debug());
                    }
                }
                false
            }
            AppScreen::Setup(menu) => match menu.update() {
                SetupAction::None => false,
                SetupAction::Back => {
                    self.screen = AppScreen::Launcher;
                    false
                }
                SetupAction::Start(config) => {
                    self.screen = AppScreen::Playing(GameState::new_with_config(config));
                    false
                }
            },
            AppScreen::Playing(game) => game.update(),
        }
    }

    pub fn draw(&self) {
        match &self.screen {
            AppScreen::Launcher => draw_launcher(),
            AppScreen::Setup(menu) => menu.draw(),
            AppScreen::Playing(game) => game.draw(),
        }
    }

    pub fn needs_sprite_load(&self) -> bool {
        matches!(self.screen, AppScreen::Playing(_))
    }
}

pub async fn bootstrap_app(config: AppBootstrap) -> App {
    let mut app = match config {
        AppBootstrap::Debug => App::new_debug(),
        AppBootstrap::Normal => App::new_normal(),
    };
    if app.needs_sprite_load() {
        if let AppScreen::Playing(game) = app.screen_mut() {
            App::load_sprites(game).await;
        }
    }
    app
}

pub enum AppBootstrap {
    Debug,
    Normal,
}

fn launcher_layout() -> LauncherLayout {
    LauncherLayout {
        normal: Rect {
            x: screen_width() / 2.0 - 180.0,
            y: screen_height() / 2.0 - 40.0,
            w: 360.0,
            h: 72.0,
        },
        debug: Rect {
            x: screen_width() / 2.0 - 180.0,
            y: screen_height() / 2.0 + 56.0,
            w: 360.0,
            h: 72.0,
        },
    }
}

fn draw_launcher() {
    clear_background(Color::new(0.06, 0.08, 0.12, 1.0));
    let layout = launcher_layout();
    let (mx, my) = mouse_position();

    let title = "PANGAEA";
    let title_size = 64.0;
    let title_width = measure_text(title, None, title_size as u16, 1.0).width;
    draw_text(
        title,
        screen_width() / 2.0 - title_width / 2.0,
        screen_height() / 2.0 - 120.0,
        title_size,
        Color::new(0.95, 0.9, 0.75, 1.0),
    );

    draw_text(
        "Choose a game mode",
        screen_width() / 2.0 - 110.0,
        screen_height() / 2.0 - 72.0,
        22.0,
        Color::new(0.65, 0.7, 0.78, 1.0),
    );

    draw_launcher_button(
        "Normal Mode",
        "Configure teams, dinos, and colors",
        &layout.normal,
        layout.normal.contains(mx, my),
    );
    draw_launcher_button(
        "Debug Mode",
        "Quick start with default 4-team setup",
        &layout.debug,
        layout.debug.contains(mx, my),
    );
}

fn draw_launcher_button(title: &str, subtitle: &str, rect: &Rect, hover: bool) {
    let bg = if hover {
        Color::new(0.2, 0.3, 0.42, 1.0)
    } else {
        Color::new(0.14, 0.17, 0.22, 1.0)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        Color::new(0.45, 0.52, 0.62, 1.0),
    );
    draw_text(
        title,
        rect.x + 20.0,
        rect.y + 30.0,
        26.0,
        Color::new(0.95, 0.95, 0.95, 1.0),
    );
    draw_text(
        subtitle,
        rect.x + 20.0,
        rect.y + 56.0,
        16.0,
        Color::new(0.7, 0.75, 0.82, 1.0),
    );
}
