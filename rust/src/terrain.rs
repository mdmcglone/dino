use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    Water,
    ShallowWater,
    Grass,
    Forest,
    Desert,
    Mountain,
    Snow,
    Savanna,
    Jungle,
}

impl TerrainType {
    pub fn color(&self) -> Color {
        match self {
            TerrainType::Water => Color::new(64.0/255.0, 164.0/255.0, 223.0/255.0, 1.0),         // Blue ocean
            TerrainType::ShallowWater => Color::new(100.0/255.0, 184.0/255.0, 233.0/255.0, 1.0), // Light blue coastal water
            TerrainType::Grass => Color::new(34.0/255.0, 139.0/255.0, 34.0/255.0, 1.0),          // Green plains
            TerrainType::Forest => Color::new(34.0/255.0, 80.0/255.0, 34.0/255.0, 1.0),          // Dark green forest
            TerrainType::Desert => Color::new(238.0/255.0, 203.0/255.0, 173.0/255.0, 1.0),       // Sandy desert
            TerrainType::Mountain => Color::new(139.0/255.0, 90.0/255.0, 43.0/255.0, 1.0),       // Brown mountains
            TerrainType::Snow => Color::new(240.0/255.0, 240.0/255.0, 240.0/255.0, 1.0),         // White snow caps
            TerrainType::Savanna => Color::new(189.0/255.0, 183.0/255.0, 107.0/255.0, 1.0),      // Yellowish grassland
            TerrainType::Jungle => Color::new(0.0/255.0, 100.0/255.0, 0.0/255.0, 1.0),           // Deep green tropical
        }
    }
    
    pub fn border_color(&self) -> Color {
        // Darker borders for better contrast
        match self {
            TerrainType::Water => Color::new(44.0/255.0, 114.0/255.0, 173.0/255.0, 1.0),
            TerrainType::ShallowWater => Color::new(70.0/255.0, 134.0/255.0, 183.0/255.0, 1.0),
            TerrainType::Grass => Color::new(24.0/255.0, 89.0/255.0, 24.0/255.0, 1.0),
            TerrainType::Forest => Color::new(24.0/255.0, 50.0/255.0, 24.0/255.0, 1.0),
            TerrainType::Desert => Color::new(188.0/255.0, 153.0/255.0, 123.0/255.0, 1.0),
            TerrainType::Mountain => Color::new(89.0/255.0, 60.0/255.0, 33.0/255.0, 1.0),
            TerrainType::Snow => Color::new(190.0/255.0, 190.0/255.0, 190.0/255.0, 1.0),
            TerrainType::Savanna => Color::new(139.0/255.0, 133.0/255.0, 77.0/255.0, 1.0),
            TerrainType::Jungle => Color::new(0.0/255.0, 70.0/255.0, 0.0/255.0, 1.0),
        }
    }
} 