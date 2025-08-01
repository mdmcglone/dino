# Neon Pangaea - C++ Version

A C++ implementation of the neon Pangaea hex map visualization using SDL2.

## Features

- **Neon aesthetic** with glowing hex borders on black background
- **Pangaea supercontinent** with realistic terrain distribution
- **Smooth camera panning** with arrow keys
- **Cyberpunk grid effect**
- **60 FPS performance** with hardware acceleration

## Dependencies

- SDL2 (Simple DirectMedia Layer 2)
- SDL2_ttf (TrueType font support)
- C++17 compiler

## Installation

### macOS (using Homebrew)
```bash
brew install sdl2 sdl2_ttf
```

### Linux (Ubuntu/Debian)
```bash
sudo apt-get install libsdl2-dev libsdl2-ttf-dev
```

### Windows
Download SDL2 and SDL2_ttf development libraries from:
- https://www.libsdl.org/download-2.0.php
- https://www.libsdl.org/projects/SDL_ttf/

## Building

### Using Make
```bash
cd c++
make
```

### Using CMake
```bash
cd c++
mkdir build
cd build
cmake ..
make
```

## Running
```bash
./neon_pangaea
```

## Controls

- **Arrow Keys**: Pan the camera
- **ESC**: Exit

## Project Structure

```
c++/
├── include/          # Header files
│   ├── terrain.h     # Terrain types and colors
│   ├── basemap.h     # Base map class
│   ├── pangaea.h     # Pangaea map generation
│   └── renderer.h    # Hex rendering system
├── src/              # Source files
│   ├── main.cpp      # Main game loop
│   ├── pangaea.cpp   # Pangaea implementation
│   └── renderer.cpp  # Rendering implementation
├── Makefile          # Simple build system
├── CMakeLists.txt    # CMake build configuration
└── README.md         # This file
```

## Technical Details

### Hex Coordinate System
Uses axial coordinates (q, r) where:
- q = column (left to right)
- r = row (diagonal)

### Rendering
- Uses SDL2's hardware-accelerated rendering
- Hexagons drawn using triangle fan method
- Glow effect created with colored borders

### Map Generation
- Overlapping elliptical regions create continent shape
- Distance-based terrain placement
- Procedural variation for natural appearance

## Customization

### Adding New Terrain Types
Edit `include/terrain.h`:
```cpp
case TerrainType::NEW_TERRAIN:
    return {{r, g, b, 255}, {r+50, g+50, b+50, 255}, "New Terrain"};
```

### Creating New Maps
1. Create new class inheriting from `BaseMap`
2. Override `generateMap()` method
3. Include and instantiate in `main.cpp`

## Performance Notes

- Only visible hexagons are rendered
- Hardware acceleration via SDL2
- Efficient hex coordinate conversion
- Optimized for 60 FPS

## Troubleshooting

### Font Loading Issues
The renderer tries multiple font paths. If fonts don't load:
1. Check font paths in `renderer.cpp`
2. Install system fonts or specify custom path

### Build Errors
- Ensure SDL2 and SDL2_ttf are properly installed
- Check include paths in Makefile/CMakeLists.txt
- Verify C++17 support in your compiler 