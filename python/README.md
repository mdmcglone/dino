# Neon Pangaea

A static hex map visualization of the Pangaea supercontinent with a cyberpunk/neon aesthetic.

## Features

- **Neon color scheme** on black background
- **Static Pangaea map** with realistic terrain distribution
- **Smooth camera panning** with arrow keys
- **Cyberpunk grid effect** for enhanced atmosphere
- **Glowing hex borders** for the neon look

## Installation

```bash
pip install pygame
```

## Running the Visualization

```bash
python main.py
```

## Controls

- **Arrow Keys**: Pan the camera to explore the map
- **ESC**: Exit the application

## Map Features

The Pangaea supercontinent includes:
- **Neon cyan oceans** with bright cyan shallow waters
- **Neon green plains** forming the main landmass
- **Neon orange deserts** in the continental interior
- **Neon magenta forests** along the coasts
- **Neon teal jungles** in tropical southern regions
- **Neon purple mountains** where tectonic plates collided
- **Neon yellow savannas** as transition zones
- **Pure white snow** on mountain peaks

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

## Project Structure

```
dino/
├── main.py           # Main renderer and game loop
├── maps/             # Map package
│   ├── __init__.py
│   ├── terrain.py    # Neon terrain colors
│   ├── base_map.py   # Base map class
│   └── pangaea.py    # Pangaea generation
└── README.md
```

## Creating Custom Maps

To create a new map with different terrain distribution:

1. Create a new file in `maps/` (e.g., `maps/custom_map.py`)
2. Inherit from `BaseMap`
3. Override `generate_map()` to place terrain
4. Import and use in `main.py`

Example:
```python
from .base_map import BaseMap
from .terrain import TerrainType

class CustomMap(BaseMap):
    def generate_map(self):
        # Your terrain generation logic
        pass
```

## Technical Details

- Uses **axial coordinates** for hexagonal grid
- Procedural terrain generation creates natural-looking continents
- Efficient rendering only draws visible hexagons
- Smooth 60 FPS performance

## Future Enhancements

- Add particle effects for enhanced neon glow
- Implement zoom functionality
- Add ambient animations (pulsing borders, etc.)
- Create additional map presets
- Add screenshot capability 