// Game state management

use macroquad::prelude::*;
use crate::maps::{PangaeaMap, Map, TerrainType};
use crate::rendering::HexMapRenderer;
use crate::input::{KeyboardHandler, MouseHandler};
use crate::core::HexCoord;
use ::rand::prelude::*;

pub struct GameState {
    map: PangaeaMap,
    renderer: HexMapRenderer,
    keyboard_handler: KeyboardHandler,
    mouse_handler: MouseHandler,
    stick_figure_pos: HexCoord,
    stick_figure_selected: bool,
}

impl GameState {
    pub fn new() -> Self {
        println!("\n=== PANGAEA ===");
        println!("Generating supercontinent...");
        
        let map = PangaeaMap::new();
        
        // Find a random land tile for the stick figure
        let mut rng = thread_rng();
        let land_tiles: Vec<HexCoord> = map.get_tiles()
            .iter()
            .filter(|(_, terrain)| **terrain != TerrainType::Water && **terrain != TerrainType::ShallowWater)
            .map(|(coord, _)| *coord)
            .collect();
        
        let stick_figure_pos = if !land_tiles.is_empty() {
            land_tiles[rng.gen_range(0..land_tiles.len())]
        } else {
            HexCoord::new(17, 17) // Fallback to center if no land found
        };
        
        println!("\nMap generated!");
        println!("Click on the stick figure to select it, then click a neighboring tile to move!");
        println!("Use arrow keys to pan the camera, +/- to zoom");
        
        Self {
            map,
            renderer: HexMapRenderer::new(),
            keyboard_handler: KeyboardHandler::new(),
            mouse_handler: MouseHandler::new(),
            stick_figure_pos,
            stick_figure_selected: false,
        }
    }
    
    pub async fn load_overlay(&mut self, path: &str) {
        self.renderer.load_overlay(path).await;
    }
    
    pub fn update(&mut self) -> bool {
        // Handle mouse clicks for stick figure
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(clicked_hex) = self.mouse_handler.get_mouse_hex(&self.renderer) {
                if clicked_hex == self.stick_figure_pos && !self.stick_figure_selected {
                    // Select the stick figure
                    self.stick_figure_selected = true;
                } else if self.stick_figure_selected {
                    // Try to move to clicked hex if it's a neighbor
                    let neighbors = self.stick_figure_pos.offset_neighbors();
                    if neighbors.contains(&clicked_hex) {
                        // Check if the target tile is not deep water (shallow water is OK)
                        let terrain = self.map.get_tile(&clicked_hex);
                        if terrain != TerrainType::Water {
                            self.stick_figure_pos = clicked_hex;
                        }
                    }
                    // Deselect after attempting to move
                    self.stick_figure_selected = false;
                } else {
                    // Clicked somewhere else, deselect
                    self.stick_figure_selected = false;
                }
            }
        }
        
        // Handle keyboard input and return true if should exit
        self.keyboard_handler.handle_input(&mut self.renderer)
    }
    
    pub fn draw(&self) {
        // Clear screen with light blue-gray background
        clear_background(Color::new(0.85, 0.85, 0.9, 1.0));
        
        // Draw grid effect
        self.renderer.draw_grid_effect();
        
        // Draw hex map
        self.renderer.draw_map(&self.map);
        
        // Draw stick figure and selection
        if self.stick_figure_selected {
            self.renderer.draw_selection_highlight(&self.stick_figure_pos);
        }
        self.renderer.draw_stick_figure(&self.stick_figure_pos);
        
        // Draw overlay on top
        self.renderer.draw_overlay();
        
        // Draw UI
        self.renderer.draw_ui();
    }
} 