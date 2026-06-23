use std::collections::HashMap;
use crate::core::HexCoord;
use super::{
    base_map::Map,
    pangaea::PangaeaMap,
    random_map::RandomMap,
    terrain::TerrainType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapKind {
    Pangaea,
    Random,
}

impl MapKind {
    pub fn label(self) -> &'static str {
        match self {
            MapKind::Pangaea => "Pangaea",
            MapKind::Random => "Random",
        }
    }
}

pub enum WorldMap {
    Pangaea(PangaeaMap),
    Random(RandomMap),
}

impl WorldMap {
    pub fn generate(kind: MapKind) -> Self {
        match kind {
            MapKind::Pangaea => Self::Pangaea(PangaeaMap::new()),
            MapKind::Random => Self::Random(RandomMap::new()),
        }
    }

    pub fn kind(&self) -> MapKind {
        match self {
            Self::Pangaea(_) => MapKind::Pangaea,
            Self::Random(_) => MapKind::Random,
        }
    }
}

impl Map for WorldMap {
    fn get_tile(&self, coord: &HexCoord) -> TerrainType {
        match self {
            Self::Pangaea(map) => map.get_tile(coord),
            Self::Random(map) => map.get_tile(coord),
        }
    }

    fn get_tiles(&self) -> &HashMap<HexCoord, TerrainType> {
        match self {
            Self::Pangaea(map) => map.get_tiles(),
            Self::Random(map) => map.get_tiles(),
        }
    }

    fn width(&self) -> i32 {
        match self {
            Self::Pangaea(map) => map.width(),
            Self::Random(map) => map.width(),
        }
    }

    fn height(&self) -> i32 {
        match self {
            Self::Pangaea(map) => map.height(),
            Self::Random(map) => map.height(),
        }
    }
}
