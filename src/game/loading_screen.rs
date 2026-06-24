use macroquad::prelude::*;
use ::rand::prelude::*;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use super::game_state::GameState;
use super::team_abilities;
use super::team_setup::{self, MatchConfig};

const PARADE_SPEED: f32 = 120.0;
const SPRITE_SIZE: f32 = 64.0;
const PARADE_SPACING: f32 = 78.0;

const GAME_SPRITES: [(usize, &str); 5] = [
    (team_abilities::TREX_TEAM, "sprites/trex_clear.png"),
    (team_abilities::BRONTO_TEAM, "sprites/bronto_clear.png"),
    (team_abilities::PTERO_TEAM, "sprites/ptero_clear.png"),
    (team_abilities::TRICERA_TEAM, "sprites/tricera_clear.png"),
    (team_abilities::KRONO_TEAM, "sprites/krono.png"),
];

struct ParadeSprite {
    dino_type: usize,
    x: f32,
    tint: Color,
}

struct ParadeAssets {
    textures: Vec<Texture2D>,
}

struct DinoParade {
    entries: Vec<ParadeSprite>,
    assets: ParadeAssets,
}

pub struct LoadingScreen {
    config: MatchConfig,
    game: Option<GameState>,
    parade: Option<DinoParade>,
    build_rx: Option<Receiver<GameState>>,
    next_sprite: usize,
}

impl LoadingScreen {
    pub fn new(config: MatchConfig) -> Self {
        Self {
            config,
            game: None,
            parade: None,
            build_rx: None,
            next_sprite: 0,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.game.as_ref().is_some_and(|game| game.sprites_loaded())
    }

    pub fn drain_game(&mut self) -> GameState {
        self.game.take().expect("loading finished without a game")
    }

    pub async fn advance(&mut self) {
        if self.parade.is_none() {
            self.parade = Some(DinoParade::load().await);
            return;
        }

        if self.game.is_none() && self.build_rx.is_none() {
            let config = self.config.clone();
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                let _ = tx.send(GameState::new_with_config(config));
            });
            self.build_rx = Some(rx);
            return;
        }

        if self.game.is_none() {
            let Some(rx) = &self.build_rx else {
                return;
            };
            if let Ok(game) = rx.try_recv() {
                self.game = Some(game);
                self.build_rx = None;
            }
            return;
        }

        let Some(game) = &mut self.game else {
            return;
        };
        if game.sprites_loaded() {
            return;
        }

        if self.next_sprite < GAME_SPRITES.len() {
            let (team, path) = GAME_SPRITES[self.next_sprite];
            game.load_team_sprite(team, path).await;
            self.next_sprite += 1;
            if self.next_sprite >= GAME_SPRITES.len() {
                game.mark_sprites_loaded();
            }
        }
    }

    pub fn tick_parade(&mut self) {
        let dt = get_frame_time();
        let dt = if dt > 0.0 { dt } else { 1.0 / 60.0 };
        if let Some(parade) = &mut self.parade {
            parade.update(dt);
        }
    }

    pub fn draw(&mut self) {
        self.tick_parade();

        clear_background(Color::new(0.06, 0.08, 0.12, 1.0));

        let center_y = screen_height() * 0.5;
        let label_y = center_y - screen_height() * 0.15;
        let parade_y = center_y + screen_height() * 0.15;

        let label = "Loading";
        let label_size = 48.0;
        let label_width = measure_text(label, None, label_size as u16, 1.0).width;
        draw_text(
            label,
            screen_width() / 2.0 - label_width / 2.0,
            label_y,
            label_size,
            Color::new(0.95, 0.92, 0.85, 1.0),
        );

        if let Some(parade) = &self.parade {
            parade.draw(parade_y);
        }
    }
}

impl DinoParade {
    async fn load() -> Self {
        let paths = [
            "sprites/trex_clear.png",
            "sprites/bronto_clear.png",
            "sprites/ptero_clear.png",
            "sprites/tricera_clear.png",
            "sprites/krono.png",
        ];

        let mut textures = Vec::with_capacity(paths.len());
        for path in paths {
            let texture = load_texture(path).await.expect("failed to load parade sprite");
            texture.set_filter(FilterMode::Nearest);
            textures.push(texture);
        }

        let mut parade = Self {
            entries: Vec::new(),
            assets: ParadeAssets { textures },
        };
        parade.fill_screen();
        parade
    }

    fn random_entry(&self, x: f32) -> ParadeSprite {
        let mut rng = thread_rng();
        let dino_type = rng.gen_range(0..self.assets.textures.len());
        let tint = if dino_type == team_abilities::KRONO_TEAM {
            WHITE
        } else {
            let palette = team_setup::color_palette();
            palette[rng.gen_range(0..palette.len())]
        };
        ParadeSprite { dino_type, x, tint }
    }

    fn fill_screen(&mut self) {
        self.entries.clear();
        let width = screen_width().max(800.0);
        let mut x = SPRITE_SIZE * 0.5;
        while x < width + SPRITE_SIZE {
            self.entries.push(self.random_entry(x));
            x += PARADE_SPACING;
        }
    }

    fn spawn_entry(&mut self, x: f32) {
        self.entries.push(self.random_entry(x));
    }

    fn update(&mut self, dt: f32) {
        for entry in &mut self.entries {
            entry.x -= PARADE_SPEED * dt;
        }

        self.entries.retain(|entry| entry.x > -SPRITE_SIZE);

        let rightmost = self
            .entries
            .iter()
            .map(|entry| entry.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        if rightmost < screen_width() + PARADE_SPACING {
            self.spawn_entry(rightmost + PARADE_SPACING);
        }
    }

    fn draw(&self, center_y: f32) {
        let top = center_y - SPRITE_SIZE / 2.0;
        for entry in &self.entries {
            let texture = &self.assets.textures[entry.dino_type];
            draw_texture_ex(
                *texture,
                entry.x - SPRITE_SIZE / 2.0,
                top,
                entry.tint,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                    ..Default::default()
                },
            );
        }
    }
}
