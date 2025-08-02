// Game state management

use macroquad::prelude::*;
use crate::maps::PangaeaMap;
use crate::rendering::HexMapRenderer;
use crate::input::KeyboardHandler;

pub struct GameState {
    map: PangaeaMap,
    renderer: HexMapRenderer,
    input_handler: KeyboardHandler,
}

impl GameState {
    pub fn new() -> Self {
        println!("\n=== PANGAEA ===");
        println!("Generating supercontinent...");
        
        let map = PangaeaMap::new();
        
        println!("\nMap generated!");
        println!("Use arrow keys to explore the world");
        println!("Press 'O' to toggle overlay, +/- to zoom");
        
        Self {
            map,
            renderer: HexMapRenderer::new(),
            input_handler: KeyboardHandler::new(),
        }
    }
    
    pub async fn load_overlay(&mut self, path: &str) {
        self.renderer.load_overlay(path).await;
    }
    
    pub fn update(&mut self) -> bool {
        // Handle input and return true if should exit
        self.input_handler.handle_input(&mut self.renderer)
    }
    
    pub fn draw(&self) {
        // Clear screen with light blue-gray background
        clear_background(Color::new(0.85, 0.85, 0.9, 1.0));
        
        // Draw grid effect
        self.renderer.draw_grid_effect();
        
        // Draw hex map
        self.renderer.draw_map(&self.map);
        
        // Draw overlay on top
        self.renderer.draw_overlay();
        
        // Draw UI
        self.renderer.draw_ui();
    }
} 