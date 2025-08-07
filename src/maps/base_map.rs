// Base map trait and common map functionality

use std::collections::HashMap;
use crate::core::HexCoord;
use super::terrain::TerrainType;

pub trait Map {
    fn get_tile(&self, coord: &HexCoord) -> TerrainType;
    fn get_tiles(&self) -> &HashMap<HexCoord, TerrainType>;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
} 