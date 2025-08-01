mod terrain;
mod map;
mod renderer;

use macroquad::prelude::*;
use map::PangaeaMap;
use renderer::HexMapRenderer;

fn window_conf() -> Conf {
    Conf {
        window_title: "Neon Pangaea".to_owned(),
        window_width: 1400,
        window_height: 900,
        fullscreen: false,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    println!("\n=== PANGAEA ===");
    println!("Generating supercontinent...");
    
    // Create map and renderer
    let pangaea_map = PangaeaMap::new();
    let mut renderer = HexMapRenderer::new();
    
    println!("\nMap generated!");
    println!("Use arrow keys to explore the world");
    
    loop {
        // Handle input
        let pan_speed = 10.0;
        
        if is_key_down(KeyCode::Left) {
            renderer.pan_camera(pan_speed, 0.0);
        }
        if is_key_down(KeyCode::Right) {
            renderer.pan_camera(-pan_speed, 0.0);
        }
        if is_key_down(KeyCode::Up) {
            renderer.pan_camera(0.0, pan_speed);
        }
        if is_key_down(KeyCode::Down) {
            renderer.pan_camera(0.0, -pan_speed);
        }
        
        // Exit on ESC
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        
        // Clear screen with light blue-gray background
        clear_background(Color::new(0.85, 0.85, 0.9, 1.0));
        
        // Draw grid effect
        renderer.draw_grid_effect();
        
        // Draw hex map
        renderer.draw_map(&pangaea_map);
        
        // Draw UI
        renderer.draw_ui();
        
        next_frame().await;
    }
}
