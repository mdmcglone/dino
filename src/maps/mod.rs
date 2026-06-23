// Map generation and terrain management

pub mod terrain;
pub mod base_map;
pub mod map_postprocess;
pub mod pangaea;
pub mod random_map;
pub mod world_map;

pub use terrain::TerrainType;
pub use base_map::Map;
pub use pangaea::PangaeaMap;
pub use random_map::RandomMap;
pub use world_map::{MapKind, WorldMap}; 