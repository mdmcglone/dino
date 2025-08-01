import random
import math
from .base_map import BaseMap
from .terrain import TerrainType

class PangaeaMap(BaseMap):
    """A map shaped like the supercontinent Pangaea with realistic terrain"""
    
    def __init__(self):
        super().__init__(width=35, height=30)
    
    def generate_map(self):
        """Generate a Pangaea-shaped continent"""
        # First, fill everything with water
        for q in range(self.width):
            for r in range(self.height):
                self.tiles[(q, r)] = TerrainType.WATER
        
        # Create the main landmass shape - irregular blob
        self._create_pangaea_shape()
        
        # Add terrain features
        self._add_mountain_ranges()
        self._add_deserts()
        self._add_forests_and_jungles()
        self._add_coastal_features()
        self._add_terrain_variation()
    
    def _create_pangaea_shape(self):
        """Create the basic Pangaea landmass shape"""
        center_q, center_r = self.width // 2, self.height // 2
        
        # Define multiple overlapping regions to create irregular shape
        regions = [
            # Main body (central mass)
            {"center": (center_q, center_r), "radius": 8, "stretch_x": 1.2, "stretch_y": 1.0},
            # Northern extension (Laurasia)
            {"center": (center_q - 3, center_r - 5), "radius": 6, "stretch_x": 1.5, "stretch_y": 0.8},
            # Southern extension (Gondwana)
            {"center": (center_q + 2, center_r + 6), "radius": 7, "stretch_x": 1.3, "stretch_y": 1.2},
            # Western bulge
            {"center": (center_q - 7, center_r + 2), "radius": 5, "stretch_x": 0.8, "stretch_y": 1.4},
            # Eastern peninsula
            {"center": (center_q + 8, center_r - 2), "radius": 4, "stretch_x": 1.6, "stretch_y": 0.7},
        ]
        
        # Create landmass from overlapping elliptical regions
        for region in regions:
            cx, cr = region["center"]
            radius = region["radius"]
            stretch_x = region["stretch_x"]
            stretch_y = region["stretch_y"]
            
            for q in range(self.width):
                for r in range(self.height):
                    # Calculate stretched distance
                    dx = (q - cx) / stretch_x
                    dy = (r - cr) / stretch_y
                    distance = math.sqrt(dx*dx + dy*dy)
                    
                    # Add some noise for irregular coastline
                    noise = random.random() * 1.5
                    if distance < radius + noise:
                        self.tiles[(q, r)] = TerrainType.GRASS
    
    def _add_mountain_ranges(self):
        """Add mountain ranges where tectonic plates would have collided"""
        # Central mountain range (like Appalachians/Hercynian)
        for i in range(15):
            q = self.width // 2 + random.randint(-2, 2)
            r = self.height // 2 - 8 + i + random.randint(-1, 1)
            self._place_mountain_cluster(q, r, 2)
        
        # Eastern range
        for i in range(10):
            q = self.width // 2 + 6 + random.randint(-1, 1)
            r = self.height // 2 - 5 + i
            self._place_mountain_cluster(q, r, 1)
    
    def _place_mountain_cluster(self, center_q: int, center_r: int, size: int):
        """Place a cluster of mountains"""
        if self.get_tile(center_q, center_r) != TerrainType.WATER:
            self.set_tile(center_q, center_r, TerrainType.MOUNTAIN)
            
            if size > 0:
                for nq, nr in self.get_neighbors(center_q, center_r):
                    if random.random() < 0.6 and self.get_tile(nq, nr) != TerrainType.WATER:
                        self.set_tile(nq, nr, TerrainType.MOUNTAIN)
    
    def _add_deserts(self):
        """Add deserts in the interior (far from coasts)"""
        for q in range(self.width):
            for r in range(self.height):
                if self.get_tile(q, r) == TerrainType.GRASS:
                    # Check distance to nearest water
                    water_distance = self._distance_to_water(q, r)
                    
                    # Interior areas become desert
                    if water_distance > 4:
                        if random.random() < 0.7:
                            self.set_tile(q, r, TerrainType.DESERT)
                    elif water_distance > 3 and random.random() < 0.3:
                        self.set_tile(q, r, TerrainType.SAVANNA)
    
    def _add_forests_and_jungles(self):
        """Add forests near coasts and jungles in tropical areas"""
        for q in range(self.width):
            for r in range(self.height):
                terrain = self.get_tile(q, r)
                if terrain in [TerrainType.GRASS, TerrainType.SAVANNA]:
                    water_distance = self._distance_to_water(q, r)
                    
                    # Coastal areas get forests
                    if water_distance <= 2:
                        # Southern areas get jungle (tropical)
                        if r > self.height * 0.6:
                            if random.random() < 0.5:
                                self.set_tile(q, r, TerrainType.JUNGLE)
                        else:
                            if random.random() < 0.4:
                                self.set_tile(q, r, TerrainType.FOREST)
    
    def _add_coastal_features(self):
        """Add shallow water around coasts"""
        for q in range(self.width):
            for r in range(self.height):
                if self.get_tile(q, r) == TerrainType.WATER:
                    # Check if adjacent to land
                    for nq, nr in self.get_neighbors(q, r):
                        if (0 <= nq < self.width and 0 <= nr < self.height and 
                            self.get_tile(nq, nr) != TerrainType.WATER and
                            self.get_tile(nq, nr) != TerrainType.SHALLOW_WATER):
                            self.set_tile(q, r, TerrainType.SHALLOW_WATER)
                            break
    
    def _add_terrain_variation(self):
        """Add snow caps to mountains and other variations"""
        for q in range(self.width):
            for r in range(self.height):
                terrain = self.get_tile(q, r)
                
                # Add snow to some mountain peaks
                if terrain == TerrainType.MOUNTAIN:
                    # Check if surrounded by other mountains (peak)
                    mountain_neighbors = sum(1 for nq, nr in self.get_neighbors(q, r)
                                           if 0 <= nq < self.width and 0 <= nr < self.height 
                                           and self.get_tile(nq, nr) == TerrainType.MOUNTAIN)
                    if mountain_neighbors >= 4 and random.random() < 0.3:
                        self.set_tile(q, r, TerrainType.SNOW)
    
    def _distance_to_water(self, q: int, r: int) -> int:
        """Calculate minimum distance to water (BFS)"""
        if self.get_tile(q, r) == TerrainType.WATER:
            return 0
        
        visited = set()
        queue = [(q, r, 0)]
        visited.add((q, r))
        
        while queue:
            cq, cr, dist = queue.pop(0)
            
            for nq, nr in self.get_neighbors(cq, cr):
                if (0 <= nq < self.width and 0 <= nr < self.height and 
                    (nq, nr) not in visited):
                    
                    if self.get_tile(nq, nr) in [TerrainType.WATER, TerrainType.SHALLOW_WATER]:
                        return dist + 1
                    
                    visited.add((nq, nr))
                    if dist < 8:  # Limit search distance
                        queue.append((nq, nr, dist + 1))
        
        return 10  # Max distance 