// Terrain types and their associated colors

use macroquad::prelude::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            TerrainType::Water => Color::new(64.0/255.0, 164.0/255.0, 223.0/255.0, 1.0),
            TerrainType::ShallowWater => Color::new(100.0/255.0, 184.0/255.0, 233.0/255.0, 1.0),
            TerrainType::Grass => Color::new(34.0/255.0, 139.0/255.0, 34.0/255.0, 1.0),
            TerrainType::Forest => Color::new(34.0/255.0, 85.0/255.0, 34.0/255.0, 1.0),
            TerrainType::Desert => Color::new(238.0/255.0, 203.0/255.0, 173.0/255.0, 1.0),
            TerrainType::Mountain => Color::new(139.0/255.0, 90.0/255.0, 43.0/255.0, 1.0),
            TerrainType::Snow => Color::new(245.0/255.0, 245.0/255.0, 250.0/255.0, 1.0),
            TerrainType::Savanna => Color::new(189.0/255.0, 183.0/255.0, 107.0/255.0, 1.0),
            TerrainType::Jungle => Color::new(0.0/255.0, 100.0/255.0, 0.0/255.0, 1.0),
        }
    }
    
    pub fn border_color(&self) -> Color {
        match self {
            TerrainType::Water => Color::new(40.0/255.0, 120.0/255.0, 180.0/255.0, 1.0),
            TerrainType::ShallowWater => Color::new(70.0/255.0, 150.0/255.0, 200.0/255.0, 1.0),
            TerrainType::Grass => Color::new(20.0/255.0, 100.0/255.0, 20.0/255.0, 1.0),
            TerrainType::Forest => Color::new(20.0/255.0, 60.0/255.0, 20.0/255.0, 1.0),
            TerrainType::Desert => Color::new(200.0/255.0, 170.0/255.0, 140.0/255.0, 1.0),
            TerrainType::Mountain => Color::new(100.0/255.0, 60.0/255.0, 30.0/255.0, 1.0),
            TerrainType::Snow => Color::new(200.0/255.0, 200.0/255.0, 210.0/255.0, 1.0),
            TerrainType::Savanna => Color::new(150.0/255.0, 145.0/255.0, 80.0/255.0, 1.0),
            TerrainType::Jungle => Color::new(0.0/255.0, 70.0/255.0, 0.0/255.0, 1.0),
        }
    }
} 