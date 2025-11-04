mod core;
mod maps;
mod rendering;
mod input;
mod game;

use macroquad::prelude::*;
use game::GameState;

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
    
    // Try to load the overlay image
    game_state.load_overlay("pangaea_overlay.png").await;
    
    // Load player sprite
    game_state.load_player_sprite("sprites/trex.png").await;
    
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
