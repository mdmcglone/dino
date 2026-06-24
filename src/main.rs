mod core;
mod maps;
mod rendering;
mod input;
mod game;

use macroquad::prelude::*;
use game::{App, AppBootstrap, AppScreen};

fn window_conf() -> Conf {
    Conf {
        window_title: "Pangaea".to_owned(),
        window_width: 1400,
        window_height: 900,
        fullscreen: false,
        window_resizable: false,
        ..Default::default()
    }
}

fn bootstrap_mode() -> AppBootstrap {
    if std::env::args().any(|arg| arg == "--normal") {
        AppBootstrap::Normal
    } else {
        AppBootstrap::Debug
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = game::app::bootstrap_app(bootstrap_mode()).await;

    loop {
        if app.update() {
            break;
        }

        app.draw();
        app.tick_loading().await;

        if let AppScreen::Playing(game) = app.screen_mut() {
            if !game.sprites_loaded() {
                App::load_sprites(game).await;
            }
        }

        next_frame().await;
    }
}
