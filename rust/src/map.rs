use std::collections::{HashMap, HashSet, VecDeque};
use rand::prelude::*;
use crate::terrain::TerrainType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }
    
    pub fn neighbors(&self) -> Vec<HexCoord> {
        vec![
            HexCoord::new(self.q + 1, self.r),
            HexCoord::new(self.q - 1, self.r),
            HexCoord::new(self.q, self.r + 1),
            HexCoord::new(self.q, self.r - 1),
            HexCoord::new(self.q + 1, self.r - 1),
            HexCoord::new(self.q - 1, self.r + 1),
        ]
    }
}

pub trait Map {
    fn get_tile(&self, coord: &HexCoord) -> TerrainType;
    fn get_tiles(&self) -> &HashMap<HexCoord, TerrainType>;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
}

pub struct PangaeaMap {
    tiles: HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
}

struct Region {
    center: HexCoord,
    radius: f32,
    stretch_x: f32,
    stretch_y: f32,
}

impl PangaeaMap {
    pub fn new() -> Self {
        let mut map = Self {
            tiles: HashMap::new(),
            width: 35,
            height: 30,
        };
        map.generate();
        map
    }
    
    fn generate(&mut self) {
        // Fill with water
        for q in 0..self.width {
            for r in 0..self.height {
                self.tiles.insert(HexCoord::new(q, r), TerrainType::Water);
            }
        }
        
        // Generate features
        self.create_pangaea_shape();
        self.add_mountain_ranges();
        self.add_deserts();
        self.add_forests_and_jungles();
        self.add_coastal_features();
        self.add_terrain_variation();
    }
    
    fn create_pangaea_shape(&mut self) {
        let center_q = self.width / 2;
        let center_r = self.height / 2;
        
        let regions = vec![
            // Main body
            Region {
                center: HexCoord::new(center_q, center_r),
                radius: 8.0,
                stretch_x: 1.2,
                stretch_y: 1.0,
            },
            // Northern extension (Laurasia)
            Region {
                center: HexCoord::new(center_q - 3, center_r - 5),
                radius: 6.0,
                stretch_x: 1.5,
                stretch_y: 0.8,
            },
            // Southern extension (Gondwana)
            Region {
                center: HexCoord::new(center_q + 2, center_r + 6),
                radius: 7.0,
                stretch_x: 1.3,
                stretch_y: 1.2,
            },
            // Western bulge
            Region {
                center: HexCoord::new(center_q - 7, center_r + 2),
                radius: 5.0,
                stretch_x: 0.8,
                stretch_y: 1.4,
            },
            // Eastern peninsula
            Region {
                center: HexCoord::new(center_q + 8, center_r - 2),
                radius: 4.0,
                stretch_x: 1.6,
                stretch_y: 0.7,
            },
        ];
        
        let mut rng = thread_rng();
        
        for region in regions {
            for q in 0..self.width {
                for r in 0..self.height {
                    let dx = (q - region.center.q) as f32 / region.stretch_x;
                    let dy = (r - region.center.r) as f32 / region.stretch_y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    let noise = rng.gen_range(0.0..1.5);
                    if distance < region.radius + noise {
                        self.tiles.insert(HexCoord::new(q, r), TerrainType::Grass);
                    }
                }
            }
        }
    }
    
    fn add_mountain_ranges(&mut self) {
        let mut rng = thread_rng();
        
        // Central mountain range
        for i in 0..15 {
            let q = self.width / 2 + rng.gen_range(-2..=2);
            let r = self.height / 2 - 8 + i + rng.gen_range(-1..=1);
            self.place_mountain_cluster(q, r, 2);
        }
        
        // Eastern range
        for i in 0..10 {
            let q = self.width / 2 + 6 + rng.gen_range(-1..=1);
            let r = self.height / 2 - 5 + i;
            self.place_mountain_cluster(q, r, 1);
        }
    }
    
    fn place_mountain_cluster(&mut self, center_q: i32, center_r: i32, size: i32) {
        let coord = HexCoord::new(center_q, center_r);
        if self.get_tile(&coord) != TerrainType::Water {
            self.tiles.insert(coord, TerrainType::Mountain);
            
            if size > 0 {
                let mut rng = thread_rng();
                for neighbor in coord.neighbors() {
                    if rng.gen::<f32>() < 0.6 && self.get_tile(&neighbor) != TerrainType::Water {
                        self.tiles.insert(neighbor, TerrainType::Mountain);
                    }
                }
            }
        }
    }
    
    fn add_deserts(&mut self) {
        let mut rng = thread_rng();
        let coords: Vec<_> = self.tiles.keys().cloned().collect();
        
        for coord in coords {
            if self.get_tile(&coord) == TerrainType::Grass {
                let water_distance = self.distance_to_water(&coord);
                
                if water_distance > 4 {
                    if rng.gen::<f32>() < 0.7 {
                        self.tiles.insert(coord, TerrainType::Desert);
                    }
                } else if water_distance > 3 && rng.gen::<f32>() < 0.3 {
                    self.tiles.insert(coord, TerrainType::Savanna);
                }
            }
        }
    }
    
    fn add_forests_and_jungles(&mut self) {
        let mut rng = thread_rng();
        let coords: Vec<_> = self.tiles.keys().cloned().collect();
        
        for coord in coords {
            let terrain = self.get_tile(&coord);
            if terrain == TerrainType::Grass || terrain == TerrainType::Savanna {
                let water_distance = self.distance_to_water(&coord);
                
                if water_distance <= 2 {
                    // Southern areas get jungle
                    if coord.r > (self.height as f32 * 0.6) as i32 {
                        if rng.gen::<f32>() < 0.5 {
                            self.tiles.insert(coord, TerrainType::Jungle);
                        }
                    } else if rng.gen::<f32>() < 0.4 {
                        self.tiles.insert(coord, TerrainType::Forest);
                    }
                }
            }
        }
    }
    
    fn add_coastal_features(&mut self) {
        let coords: Vec<_> = self.tiles.keys().cloned().collect();
        
        for coord in coords {
            if self.get_tile(&coord) == TerrainType::Water {
                let mut has_land_neighbor = false;
                for neighbor in coord.neighbors() {
                    if neighbor.q >= 0 && neighbor.q < self.width &&
                       neighbor.r >= 0 && neighbor.r < self.height {
                        let neighbor_terrain = self.get_tile(&neighbor);
                        if neighbor_terrain != TerrainType::Water &&
                           neighbor_terrain != TerrainType::ShallowWater {
                            has_land_neighbor = true;
                            break;
                        }
                    }
                }
                if has_land_neighbor {
                    self.tiles.insert(coord, TerrainType::ShallowWater);
                }
            }
        }
    }
    
    fn add_terrain_variation(&mut self) {
        let mut rng = thread_rng();
        let coords: Vec<_> = self.tiles.keys().cloned().collect();
        
        for coord in coords {
            if self.get_tile(&coord) == TerrainType::Mountain {
                let mut mountain_neighbors = 0;
                for neighbor in coord.neighbors() {
                    if neighbor.q >= 0 && neighbor.q < self.width &&
                       neighbor.r >= 0 && neighbor.r < self.height &&
                       self.get_tile(&neighbor) == TerrainType::Mountain {
                        mountain_neighbors += 1;
                    }
                }
                
                if mountain_neighbors >= 4 && rng.gen::<f32>() < 0.3 {
                    self.tiles.insert(coord, TerrainType::Snow);
                }
            }
        }
    }
    
    fn distance_to_water(&self, coord: &HexCoord) -> i32 {
        if self.get_tile(coord) == TerrainType::Water {
            return 0;
        }
        
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((coord.clone(), 0));
        visited.insert(coord.clone());
        
        while let Some((current, dist)) = queue.pop_front() {
            for neighbor in current.neighbors() {
                if neighbor.q >= 0 && neighbor.q < self.width &&
                   neighbor.r >= 0 && neighbor.r < self.height &&
                   !visited.contains(&neighbor) {
                    
                    let terrain = self.get_tile(&neighbor);
                    if terrain == TerrainType::Water || terrain == TerrainType::ShallowWater {
                        return dist + 1;
                    }
                    
                    visited.insert(neighbor.clone());
                    if dist < 8 {
                        queue.push_back((neighbor, dist + 1));
                    }
                }
            }
        }
        
        10 // Max distance
    }
}

impl Map for PangaeaMap {
    fn get_tile(&self, coord: &HexCoord) -> TerrainType {
        self.tiles.get(coord).copied().unwrap_or(TerrainType::Water)
    }
    
    fn get_tiles(&self) -> &HashMap<HexCoord, TerrainType> {
        &self.tiles
    }
    
    fn width(&self) -> i32 {
        self.width
    }
    
    fn height(&self) -> i32 {
        self.height
    }
} 