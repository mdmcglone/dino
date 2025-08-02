# Pangaea - Rust Version

A Rust implementation of the Pangaea hex map visualization using Macroquad, now with a modular architecture designed for extensibility.

## Features

- **Realistic terrain colors** with defined borders for clarity
- **Pangaea supercontinent** with realistic terrain distribution
- **Smooth camera panning** with arrow keys
- **Zoom controls** for different viewing scales
- **Overlay system** for comparing with reference maps
- **Modular architecture** for easy extension
- **60 FPS performance** with immediate mode rendering

## Installation

Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/).

## Building and Running

```bash
cd rust
cargo run --release
```

For development with faster compilation:
```bash
cargo run
```

## Controls

### Camera & View
- **Arrow Keys**: Pan the camera
- **+/-**: Zoom in/out
- **0**: Reset zoom to 100%

### Overlay
- **O**: Toggle overlay on/off
- **[/]**: Adjust overlay opacity
- **WASD**: Shift overlay position
- **R**: Reset overlay position

### General
- **ESC**: Exit

## Project Structure

```
rust/
├── src/
│   ├── main.rs         # Entry point
│   ├── core/           # Core types and traits
│   │   ├── mod.rs
│   │   └── hex_coord.rs    # Hexagonal coordinate system
│   ├── maps/           # Map generation and terrain
│   │   ├── mod.rs
│   │   ├── terrain.rs      # Terrain types and colors
│   │   ├── base_map.rs     # Map trait definition
│   │   └── pangaea.rs      # Pangaea map implementation
│   ├── rendering/      # Rendering system
│   │   ├── mod.rs
│   │   ├── hex_renderer.rs     # Core hex map rendering
│   │   ├── overlay_renderer.rs # Overlay display
│   │   └── ui_renderer.rs      # UI elements
│   ├── input/          # Input handling
│   │   ├── mod.rs
│   │   └── keyboard_handler.rs # Keyboard controls
│   └── game/           # Game state and logic
│       ├── mod.rs
│       └── game_state.rs       # Main game state management
├── Cargo.toml       # Dependencies
└── README.md        # This file
```

## Technical Details

### Dependencies
- **macroquad**: Simple and easy to use game library for Rust
- **rand**: Random number generation for procedural terrain

### Architecture

The codebase is organized into distinct modules for better maintainability:

**Core Module (`core/`)**
- `HexCoord`: Basic hexagonal coordinate system with neighbor calculation

**Maps Module (`maps/`)**
- `TerrainType`: Enum defining terrain types with associated colors
- `Map` trait: Abstraction for different map types
- `PangaeaMap`: Concrete implementation generating Pangaea-shaped continent

**Rendering Module (`rendering/`)**
- `HexMapRenderer`: Main renderer coordinating all visual elements
- `OverlayRenderer`: Handles overlay texture loading and display
- `UIRenderer`: Manages UI text and controls display

**Input Module (`input/`)**
- `KeyboardHandler`: Processes all keyboard input and controls

**Game Module (`game/`)**
- `GameState`: Main game state management and update loop

### Hex Coordinate System
Uses axial coordinates (q, r) where:
- q = column (left to right)
- r = row (diagonal)

### Performance Features
- Only renders visible hexagons
- Immediate mode rendering with Macroquad
- Efficient hex coordinate conversion
- Hardware-accelerated graphics

## Customization

### Adding New Terrain Types
Edit `src/maps/terrain.rs`:
```rust
pub enum TerrainType {
    // ... existing types
    Lava,  // Add new type
}

impl TerrainType {
    pub fn color(&self) -> Color {
        match self {
            // ... existing matches
            TerrainType::Lava => Color::new(255.0/255.0, 69.0/255.0, 0.0/255.0, 1.0),
        }
    }
    
    pub fn border_color(&self) -> Color {
        match self {
            // ... existing matches
            TerrainType::Lava => Color::new(200.0/255.0, 50.0/255.0, 0.0/255.0, 1.0),
        }
    }
}
```

### Creating New Maps
1. Create a new file in `src/maps/` (e.g., `island.rs`)
2. Implement the `Map` trait for your new map type
3. Export it from `src/maps/mod.rs`
4. Use it in `src/game/game_state.rs` instead of `PangaeaMap`

Example `src/maps/island.rs`:
```rust
use std::collections::HashMap;
use crate::core::HexCoord;
use super::{terrain::TerrainType, base_map::Map};

pub struct IslandMap {
    tiles: HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
}

impl IslandMap {
    pub fn new() -> Self {
        let mut map = Self {
            tiles: HashMap::new(),
            width: 35,
            height: 35,
        };
        map.generate();
        map
    }
    
    fn generate(&mut self) {
        // Your island generation logic here
    }
}

impl Map for IslandMap {
    fn get_tile(&self, coord: &HexCoord) -> TerrainType {
        self.tiles.get(coord).copied().unwrap_or(TerrainType::Water)
    }
    
    fn get_tiles(&self) -> &HashMap<HexCoord, TerrainType> {
        &self.tiles
    }
    
    fn width(&self) -> i32 { self.width }
    fn height(&self) -> i32 { self.height }
}
```

Then add to `src/maps/mod.rs`:
```rust
pub mod island;
pub use island::IslandMap;
```

## Terrain Color Palette

| Terrain | Color | RGB |
|---------|-------|-----|
| Deep Water | Blue Ocean | (64, 164, 223) |
| Shallow Water | Light Blue | (100, 184, 233) |
| Grassland | Green Plains | (34, 139, 34) |
| Forest | Dark Green | (34, 85, 34) |
| Desert | Sandy | (238, 203, 173) |
| Mountain | Brown | (139, 90, 43) |
| Snow | Off White | (245, 245, 250) |
| Savanna | Yellowish | (189, 183, 107) |
| Jungle | Deep Green | (0, 100, 0) |

## Extending the Architecture

The modular design makes it easy to add new features:

### Adding New Input Methods
Create a new handler in `src/input/` (e.g., `mouse_handler.rs`) and integrate it with `GameState`.

### Adding New Renderers
Create new renderers in `src/rendering/` for additional visual elements (e.g., `minimap_renderer.rs`, `particle_renderer.rs`).

### Adding Game Features
Extend the `game/` module with new systems like:
- Save/load functionality
- Map editing capabilities
- Multiplayer support
- Animation systems

### Adding New Map Types
Follow the map creation guide above to add different world generators (archipelago, continents, etc.).

## Advantages of Rust Version

1. **Memory Safety**: No null pointer exceptions or data races
2. **Performance**: Zero-cost abstractions and no garbage collector
3. **Type Safety**: Strong type system catches errors at compile time
4. **Cross-Platform**: Works on Windows, macOS, and Linux
5. **Easy Distribution**: Single binary with no runtime dependencies
6. **Modular Architecture**: Easy to extend and maintain

## Building for Release

To create an optimized binary:
```bash
cargo build --release
```

The executable will be in `target/release/pangaea`.

## Cross-Compilation

To build for other platforms:

### Windows (from macOS/Linux)
```bash
cargo install cross
cross build --release --target x86_64-pc-windows-gnu
```

### Linux (from macOS/Windows)
```bash
cross build --release --target x86_64-unknown-linux-gnu
```

## Troubleshooting

### Graphics Issues
- Macroquad should handle most graphics setup automatically
- If you see rendering issues, try updating your graphics drivers

### Performance
- Make sure to run with `--release` flag for best performance
- The debug build is significantly slower

### Compilation Errors
- Run `cargo update` to ensure latest compatible versions
- Check that you have Rust 2021 edition or later 