from enum import Enum

# Define terrain types - shared across all maps
class TerrainType(Enum):
    WATER = (64, 164, 223)         # Blue ocean
    SHALLOW_WATER = (100, 184, 233) # Light blue coastal water
    GRASS = (34, 139, 34)          # Green plains
    FOREST = (34, 80, 34)          # Dark green forest
    DESERT = (238, 203, 173)       # Sandy desert
    MOUNTAIN = (139, 90, 43)       # Brown mountains
    SNOW = (240, 240, 240)         # White snow caps
    SAVANNA = (189, 183, 107)      # Yellowish grassland
    JUNGLE = (0, 100, 0)           # Deep green tropical 