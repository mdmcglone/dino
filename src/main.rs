mod core;
mod maps;
mod rendering;
mod input;
mod game;

use macroquad::prelude::*;
use game::{GameState, team_abilities};

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

#[macroquad::main(window_conf)]
async fn main() {
    // Create game state
    let mut game_state = GameState::new();
    
    // Load grayscale sprites; team color is applied at draw time
    game_state.load_team_sprite(team_abilities::TREX_TEAM, "sprites/trex_clear.png").await;
    game_state.load_team_sprite(team_abilities::BRONTO_TEAM, "sprites/bronto_clear.png").await;
    game_state.load_team_sprite(team_abilities::PTERO_TEAM, "sprites/ptero_clear.png").await;
    game_state.load_team_sprite(team_abilities::TRICERA_TEAM, "sprites/tricera_clear.png").await;
    game_state.load_team_sprite(team_abilities::KRONO_TEAM, "sprites/krono_clear.png").await;
    
    // Main game loop
    loop {
        // Update game state and check for exit
        if game_state.update() {
            break;
        }
        
        // Draw everything
        game_state.draw();
        
        // Next frame
        next_frame().await;
    }
}
