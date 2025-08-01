import math
from typing import Dict, Tuple
from .terrain import TerrainType

class BaseMap:
    """Base class for all hex maps"""
    
    def __init__(self, width: int = 30, height: int = 25):
        self.width = width
        self.height = height
        self.tiles: Dict[Tuple[int, int], TerrainType] = {}
        
        # Initialize the map
        self.generate_map()
    
    def generate_map(self):
        """Override this method in subclasses to create specific map layouts"""
        # Default: all water
        for q in range(self.width):
            for r in range(self.height):
                self.tiles[(q, r)] = TerrainType.WATER
    
    def set_tile(self, q: int, r: int, terrain: TerrainType):
        """Set terrain type for a tile"""
        if 0 <= q < self.width and 0 <= r < self.height:
            self.tiles[(q, r)] = terrain
    
    def get_tile(self, q: int, r: int) -> TerrainType:
        """Get terrain type for a tile"""
        return self.tiles.get((q, r), TerrainType.WATER)
    
    def distance_from_point(self, q: int, r: int, center_q: int, center_r: int) -> float:
        """Calculate hex distance from a point"""
        return (abs(q - center_q) + abs(q + r - center_q - center_r) + abs(r - center_r)) / 2
    
    def get_neighbors(self, q: int, r: int) -> list[Tuple[int, int]]:
        """Get all 6 neighbors of a hex"""
        return [
            (q+1, r), (q-1, r),
            (q, r+1), (q, r-1),
            (q+1, r-1), (q-1, r+1)
        ] 