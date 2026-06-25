// Procedurally generated maps with fractal coastlines and connected walkable land

use std::collections::HashMap;
use rand::prelude::*;
use crate::core::HexCoord;
use super::{
    base_map::Map,
    map_postprocess::{
        decorate_land, ensure_traversible_connectivity, fill_water, get_tile,
        place_mountain_cluster,
    },
    terrain::TerrainType,
    world_map::MapSize,
};

pub struct RandomMap {
    tiles: HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
}

impl RandomMap {
    pub fn new() -> Self {
        Self::with_size(MapSize::Medium)
    }

    pub fn with_size(size: MapSize) -> Self {
        let (width, height) = size.dimensions();
        let mut map = Self {
            tiles: HashMap::new(),
            width,
            height,
        };
        map.generate();
        map
    }

    fn generate(&mut self) {
        let mut rng = thread_rng();
        let seed: u32 = rng.gen();

        fill_water(&mut self.tiles, self.width, self.height);
        self.generate_fractal_land(seed, &mut rng);
        self.refine_coastlines();
        self.apply_elevation_mountains(seed);
        self.add_random_mountain_ranges(&mut rng);
        decorate_land(&mut self.tiles, self.width, self.height);
        ensure_traversible_connectivity(&mut self.tiles, self.width, self.height);
    }

    fn generate_fractal_land(&mut self, seed: u32, rng: &mut impl Rng) {
        let land_threshold = rng.gen_range(0.38..0.46);
        let scale = 0.11;

        for q in 0..self.width {
            for r in 0..self.height {
                let fq = q as f32 * scale;
                let fr = r as f32 * scale;

                let warp_x = fq
                    + 0.65 * fractal_noise(fq + 1.7, fr + 2.3, 3, seed.wrapping_add(101));
                let warp_y = fr
                    + 0.65 * fractal_noise(fq + 4.1, fr - 1.9, 3, seed.wrapping_add(202));

                let elevation = fractal_noise(warp_x, warp_y, 5, seed.wrapping_add(303));
                let ridged = 1.0 - (2.0 * (0.5 - fractal_noise(warp_x * 1.4, warp_y * 1.4, 4, seed.wrapping_add(404))).abs());
                let detail = fractal_noise(fq * 2.8, fr * 2.8, 2, seed.wrapping_add(505)) * 0.18;

                let combined = elevation * 0.72 + ridged * 0.2 + detail;

                let edge_q = (q as f32 / (self.width - 1) as f32 - 0.5).abs() * 2.0;
                let edge_r = (r as f32 / (self.height - 1) as f32 - 0.5).abs() * 2.0;
                let edge_falloff = (edge_q.max(edge_r) * 0.35).min(0.3);

                if combined - edge_falloff > land_threshold {
                    self.tiles.insert(HexCoord::new(q, r), TerrainType::Grass);
                }
            }
        }
    }

    fn refine_coastlines(&mut self) {
        for _ in 0..4 {
            let coords: Vec<_> = self.tiles.keys().cloned().collect();
            let mut next = self.tiles.clone();

            for coord in coords {
                let land_neighbors = count_land_neighbors(&self.tiles, &coord, self.width, self.height);
                let is_land = get_tile(&self.tiles, &coord) != TerrainType::Water;

                if is_land {
                    if land_neighbors <= 1 {
                        next.insert(coord, TerrainType::Water);
                    }
                } else if land_neighbors >= 5 {
                    next.insert(coord, TerrainType::Grass);
                } else if land_neighbors == 4 && coord.q % 3 == coord.r % 2 {
                    next.insert(coord, TerrainType::Grass);
                }
            }

            self.tiles = next;
        }
    }

    fn apply_elevation_mountains(&mut self, seed: u32) {
        let scale = 0.14;
        for q in 0..self.width {
            for r in 0..self.height {
                let coord = HexCoord::new(q, r);
                if get_tile(&self.tiles, &coord) == TerrainType::Water {
                    continue;
                }
                let peak = fractal_noise(q as f32 * scale, r as f32 * scale, 4, seed.wrapping_add(606));
                if peak > 0.78 {
                    self.tiles.insert(coord, TerrainType::Mountain);
                }
            }
        }
    }

    fn add_random_mountain_ranges(&mut self, rng: &mut impl Rng) {
        let range_count = rng.gen_range(1..=3);
        for _ in 0..range_count {
            let start_q = rng.gen_range(4..self.width - 4);
            let start_r = rng.gen_range(4..self.height - 4);
            if get_tile(&self.tiles, &HexCoord::new(start_q, start_r)) == TerrainType::Water {
                continue;
            }

            let length = rng.gen_range(6..12);
            let mut q = start_q;
            let mut r = start_r;
            for _ in 0..length {
                place_mountain_cluster(&mut self.tiles, q, r, 1, rng);
                match rng.gen_range(0..6) {
                    0 => q += 1,
                    1 => q -= 1,
                    2 => r += 1,
                    3 => r -= 1,
                    4 => {
                        q += 1;
                        r += 1;
                    }
                    _ => {
                        q -= 1;
                        r -= 1;
                    }
                }
                if q < 2 || q >= self.width - 2 || r < 2 || r >= self.height - 2 {
                    break;
                }
            }
        }
    }
}

impl Map for RandomMap {
    fn get_tile(&self, coord: &HexCoord) -> TerrainType {
        get_tile(&self.tiles, coord)
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

fn count_land_neighbors(
    tiles: &HashMap<HexCoord, TerrainType>,
    coord: &HexCoord,
    width: i32,
    height: i32,
) -> usize {
    coord
        .offset_neighbors()
        .iter()
        .filter(|neighbor| {
            neighbor.q >= 0
                && neighbor.q < width
                && neighbor.r >= 0
                && neighbor.r < height
                && get_tile(tiles, neighbor) != TerrainType::Water
        })
        .count()
}

fn lattice_value(q: i32, r: i32, seed: u32) -> f32 {
    let mut n = q
        .wrapping_mul(374761393)
        .wrapping_add(r.wrapping_mul(668265263))
        .wrapping_add(seed as i32) as u32;
    n ^= n >> 13;
    n = n.wrapping_mul(n.wrapping_mul(n));
    n ^= n >> 16;
    (n as f32 / u32::MAX as f32).clamp(0.0, 1.0)
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = smoothstep(x - x0 as f32);
    let ty = smoothstep(y - y0 as f32);

    let v00 = lattice_value(x0, y0, seed);
    let v10 = lattice_value(x0 + 1, y0, seed);
    let v01 = lattice_value(x0, y0 + 1, seed);
    let v11 = lattice_value(x0 + 1, y0 + 1, seed);

    let ix0 = v00 + (v10 - v00) * tx;
    let ix1 = v01 + (v11 - v01) * tx;
    ix0 + (ix1 - ix0) * ty
}

fn fractal_noise(x: f32, y: f32, octaves: u32, seed: u32) -> f32 {
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut sum = 0.0;
    let mut norm = 0.0;

    for octave in 0..octaves {
        sum += amplitude * value_noise(x * frequency, y * frequency, seed.wrapping_add(octave));
        norm += amplitude;
        amplitude *= 0.5;
        frequency *= 2.05;
    }

    if norm > 0.0 {
        sum / norm
    } else {
        0.0
    }
}
