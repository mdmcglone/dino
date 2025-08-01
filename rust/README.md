# Neon Pangaea - Rust Version

A Rust implementation of the neon Pangaea hex map visualization using Macroquad.

## Features

- **Neon aesthetic** with glowing hex borders on black background
- **Pangaea supercontinent** with realistic terrain distribution
- **Smooth camera panning** with arrow keys
- **Cyberpunk grid effect**
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

- **Arrow Keys**: Pan the camera
- **ESC**: Exit

## Project Structure

```
rust/
├── src/
│   ├── main.rs      # Main game loop
│   ├── terrain.rs   # Terrain types and neon colors
│   ├── map.rs       # Map trait and Pangaea generation
│   └── renderer.rs  # Hex rendering system
├── Cargo.toml       # Dependencies
└── README.md        # This file
```

## Technical Details

### Dependencies
- **macroquad**: Simple and easy to use game library for Rust
- **rand**: Random number generation for procedural terrain

### Architecture
- **TerrainType**: Enum defining terrain types with associated neon colors
- **Map trait**: Abstraction for different map types
- **PangaeaMap**: Concrete implementation generating Pangaea-shaped continent
- **HexMapRenderer**: Handles all rendering including hexagons, UI, and grid

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
Edit `src/terrain.rs`:
```rust
pub enum TerrainType {
    // ... existing types
    Lava,  // Add new type
}

impl TerrainType {
    pub fn color(&self) -> Color {
        match self {
            // ... existing matches
            TerrainType::Lava => Color::from_rgba(255, 69, 0, 255),
        }
    }
}
```

### Creating New Maps
1. Create a new struct implementing the `Map` trait
2. Override terrain generation in your implementation
3. Use it in `main.rs` instead of `PangaeaMap`

Example:
```rust
pub struct IslandMap {
    tiles: HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
}

impl Map for IslandMap {
    // Implementation...
}
```

## Neon Color Palette

| Terrain | Color | RGB |
|---------|-------|-----|
| Deep Water | Neon Cyan | (0, 180, 255) |
| Shallow Water | Bright Cyan | (0, 255, 255) |
| Grassland | Neon Green | (0, 255, 100) |
| Forest | Neon Magenta | (255, 0, 255) |
| Desert | Neon Orange | (255, 180, 0) |
| Mountain | Neon Purple | (150, 0, 255) |
| Snow | Pure White | (255, 255, 255) |
| Savanna | Neon Yellow | (255, 255, 0) |
| Jungle | Neon Teal | (0, 255, 200) |

## Advantages of Rust Version

1. **Memory Safety**: No null pointer exceptions or data races
2. **Performance**: Zero-cost abstractions and no garbage collector
3. **Type Safety**: Strong type system catches errors at compile time
4. **Cross-Platform**: Works on Windows, macOS, and Linux
5. **Easy Distribution**: Single binary with no runtime dependencies

## Building for Release

To create an optimized binary:
```bash
cargo build --release
```

The executable will be in `target/release/neon-pangaea`.

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