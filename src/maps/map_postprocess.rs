// Shared terrain decoration for procedural maps

use std::collections::{HashMap, HashSet, VecDeque};
use rand::prelude::*;
use crate::core::HexCoord;
use super::terrain::TerrainType;

pub const DEFAULT_MAP_WIDTH: i32 = 35;
pub const DEFAULT_MAP_HEIGHT: i32 = 35;

pub fn fill_water(tiles: &mut HashMap<HexCoord, TerrainType>, width: i32, height: i32) {
    for q in 0..width {
        for r in 0..height {
            tiles.insert(HexCoord::new(q, r), TerrainType::Water);
        }
    }
}

pub fn get_tile(tiles: &HashMap<HexCoord, TerrainType>, coord: &HexCoord) -> TerrainType {
    tiles.get(coord).copied().unwrap_or(TerrainType::Water)
}

/// Tiles a standard land dino can path through (matches pathfinding: not deep water or mountain).
pub fn is_traversible(terrain: TerrainType) -> bool {
    !matches!(terrain, TerrainType::Water | TerrainType::Mountain)
}

pub fn traversible_components(
    tiles: &HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
) -> Vec<HashSet<HexCoord>> {
    let mut remaining = HashSet::new();
    for q in 0..width {
        for r in 0..height {
            let coord = HexCoord::new(q, r);
            if is_traversible(get_tile(tiles, &coord)) {
                remaining.insert(coord);
            }
        }
    }

    let mut components = Vec::new();
    while let Some(start) = remaining.iter().next().cloned() {
        let mut component = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        remaining.remove(&start);
        component.insert(start);

        while let Some(current) = queue.pop_front() {
            for neighbor in current.offset_neighbors() {
                if neighbor.q < 0
                    || neighbor.q >= width
                    || neighbor.r < 0
                    || neighbor.r >= height
                    || !remaining.remove(&neighbor)
                {
                    continue;
                }
                component.insert(neighbor);
                queue.push_back(neighbor);
            }
        }
        components.push(component);
    }

    components.sort_by_key(|component| std::cmp::Reverse(component.len()));
    components
}

const BRIDGE_WATER_RADIUS: i32 = 2;

fn bridge_tile_for(terrain: TerrainType) -> Option<TerrainType> {
    match terrain {
        TerrainType::Water => Some(TerrainType::ShallowWater),
        TerrainType::Mountain => Some(TerrainType::Grass),
        _ => None,
    }
}

fn in_bounds(coord: &HexCoord, width: i32, height: i32) -> bool {
    coord.q >= 0 && coord.q < width && coord.r >= 0 && coord.r < height
}

fn collect_greedy_path(from: HexCoord, to: HexCoord, width: i32, height: i32) -> Vec<HexCoord> {
    let mut path = vec![from];
    let mut current = from;
    let mut guard = 0;

    while current != to && guard < (width * height * 2) as usize {
        guard += 1;
        let mut best = None;
        let mut best_dist = current.distance(&to);
        for neighbor in current.offset_neighbors() {
            if !in_bounds(&neighbor, width, height) {
                continue;
            }
            let dist = neighbor.distance(&to);
            if dist < best_dist {
                best_dist = dist;
                best = Some(neighbor);
            }
        }
        let Some(next) = best else { break };
        current = next;
        path.push(current);
    }

    path
}

fn tiles_around_path(path: &[HexCoord], radius: i32, width: i32, height: i32) -> HashSet<HexCoord> {
    let mut tiles = HashSet::new();
    for &center in path {
        let mut queue = VecDeque::from([(center, 0)]);
        let mut visited = HashSet::from([center]);
        while let Some((coord, dist)) = queue.pop_front() {
            if dist <= radius {
                tiles.insert(coord);
            }
            if dist >= radius {
                continue;
            }
            for neighbor in coord.offset_neighbors() {
                if !in_bounds(&neighbor, width, height) || !visited.insert(neighbor) {
                    continue;
                }
                queue.push_back((neighbor, dist + 1));
            }
        }
    }
    tiles
}

fn carve_bridge(
    tiles: &mut HashMap<HexCoord, TerrainType>,
    from: HexCoord,
    to: HexCoord,
    width: i32,
    height: i32,
) {
    let path = collect_greedy_path(from, to, width, height);
    let footprint = tiles_around_path(&path, BRIDGE_WATER_RADIUS, width, height);

    for coord in footprint {
        let terrain = get_tile(tiles, &coord);
        if let Some(bridge) = bridge_tile_for(terrain) {
            tiles.insert(coord, bridge);
        }
    }
}

fn closest_pair_between(
    left: &HashSet<HexCoord>,
    right: &HashSet<HexCoord>,
) -> (HexCoord, HexCoord) {
    let mut best_pair = (HexCoord::new(0, 0), HexCoord::new(0, 0));
    let mut best_dist = i32::MAX;

    for a in left {
        for b in right {
            let dist = a.distance(b);
            if dist < best_dist {
                best_dist = dist;
                best_pair = (*a, *b);
            }
        }
    }

    best_pair
}

/// Connect isolated traversible regions with shallow-water channels (over ocean) or grass (through mountains).
pub fn ensure_traversible_connectivity(
    tiles: &mut HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
) {
    loop {
        let components = traversible_components(tiles, width, height);
        if components.len() <= 1 {
            return;
        }

        let main = &components[0];
        let (from, to) = (0..components.len())
            .skip(1)
            .map(|index| closest_pair_between(main, &components[index]))
            .min_by_key(|(a, b)| a.distance(b))
            .unwrap();

        carve_bridge(tiles, from, to, width, height);
    }
}

pub fn place_mountain_cluster(
    tiles: &mut HashMap<HexCoord, TerrainType>,
    center_q: i32,
    center_r: i32,
    size: i32,
    rng: &mut impl Rng,
) {
    let coord = HexCoord::new(center_q, center_r);
    if get_tile(tiles, &coord) == TerrainType::Water {
        return;
    }
    tiles.insert(coord, TerrainType::Mountain);

    if size > 0 {
        for neighbor in coord.neighbors() {
            if rng.gen::<f32>() < 0.6 && get_tile(tiles, &neighbor) != TerrainType::Water {
                tiles.insert(neighbor, TerrainType::Mountain);
            }
        }
    }
}

pub fn add_hills(
    tiles: &mut HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
    rng: &mut impl Rng,
) {
    let coords: Vec<_> = tiles.keys().cloned().collect();
    for coord in coords {
        if get_tile(tiles, &coord) != TerrainType::Grass {
            continue;
        }
        let near_mountain = coord.neighbors().iter().any(|neighbor| {
            neighbor.q >= 0
                && neighbor.q < width
                && neighbor.r >= 0
                && neighbor.r < height
                && get_tile(tiles, neighbor) == TerrainType::Mountain
        });
        if near_mountain && rng.gen::<f32>() < 0.55 {
            tiles.insert(coord, TerrainType::Hills);
        }
    }
}

pub fn add_deserts(
    tiles: &mut HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
    rng: &mut impl Rng,
) {
    let coords: Vec<_> = tiles.keys().cloned().collect();
    for coord in coords {
        if get_tile(tiles, &coord) != TerrainType::Grass {
            continue;
        }
        let water_distance = distance_to_water(tiles, &coord, width, height);
        if water_distance > 4 {
            if rng.gen::<f32>() < 0.7 {
                tiles.insert(coord, TerrainType::Desert);
            }
        } else if water_distance > 3 && rng.gen::<f32>() < 0.3 {
            tiles.insert(coord, TerrainType::Savanna);
        }
    }
}

pub fn add_forests_and_jungles(
    tiles: &mut HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
    rng: &mut impl Rng,
) {
    let coords: Vec<_> = tiles.keys().cloned().collect();
    for coord in coords {
        let terrain = get_tile(tiles, &coord);
        if terrain != TerrainType::Grass && terrain != TerrainType::Savanna {
            continue;
        }
        let water_distance = distance_to_water(tiles, &coord, width, height);
        if water_distance > 2 {
            continue;
        }
        if coord.r > (height as f32 * 0.6) as i32 {
            if rng.gen::<f32>() < 0.5 {
                tiles.insert(coord, TerrainType::Jungle);
            }
        } else if rng.gen::<f32>() < 0.4 {
            tiles.insert(coord, TerrainType::Forest);
        }
    }
}

pub fn add_coastal_features(
    tiles: &mut HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
) {
    let coords: Vec<_> = tiles.keys().cloned().collect();
    for coord in coords {
        if get_tile(tiles, &coord) != TerrainType::Water {
            continue;
        }
        let has_land_neighbor = coord.neighbors().iter().any(|neighbor| {
            neighbor.q >= 0
                && neighbor.q < width
                && neighbor.r >= 0
                && neighbor.r < height
                && !matches!(
                    get_tile(tiles, neighbor),
                    TerrainType::Water | TerrainType::ShallowWater
                )
        });
        if has_land_neighbor {
            tiles.insert(coord, TerrainType::ShallowWater);
        }
    }
}

pub fn fill_mountain_enclosures(
    tiles: &mut HashMap<HexCoord, TerrainType>,
    width: i32,
    height: i32,
) {
    let mut exterior = HashSet::new();
    let mut queue = VecDeque::new();

    for q in 0..width {
        for r in 0..height {
            let on_border = q == 0 || q == width - 1 || r == 0 || r == height - 1;
            if !on_border {
                continue;
            }
            let coord = HexCoord::new(q, r);
            if get_tile(tiles, &coord) != TerrainType::Mountain && exterior.insert(coord) {
                queue.push_back(coord);
            }
        }
    }

    while let Some(current) = queue.pop_front() {
        for neighbor in current.offset_neighbors() {
            if neighbor.q < 0
                || neighbor.q >= width
                || neighbor.r < 0
                || neighbor.r >= height
            {
                continue;
            }
            if get_tile(tiles, &neighbor) == TerrainType::Mountain || !exterior.insert(neighbor) {
                continue;
            }
            queue.push_back(neighbor);
        }
    }

    for q in 0..width {
        for r in 0..height {
            let coord = HexCoord::new(q, r);
            if get_tile(tiles, &coord) != TerrainType::Mountain && !exterior.contains(&coord) {
                tiles.insert(coord, TerrainType::Mountain);
            }
        }
    }
}

pub fn distance_to_water(
    tiles: &HashMap<HexCoord, TerrainType>,
    coord: &HexCoord,
    width: i32,
    height: i32,
) -> i32 {
    if matches!(
        get_tile(tiles, coord),
        TerrainType::Water | TerrainType::ShallowWater
    ) {
        return 0;
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((coord.clone(), 0));
    visited.insert(coord.clone());

    while let Some((current, dist)) = queue.pop_front() {
        for neighbor in current.neighbors() {
            if neighbor.q < 0
                || neighbor.q >= width
                || neighbor.r < 0
                || neighbor.r >= height
                || !visited.insert(neighbor.clone())
            {
                continue;
            }
            let terrain = get_tile(tiles, &neighbor);
            if matches!(terrain, TerrainType::Water | TerrainType::ShallowWater) {
                return dist + 1;
            }
            if dist < 8 {
                queue.push_back((neighbor, dist + 1));
            }
        }
    }

    10
}

pub fn decorate_land(tiles: &mut HashMap<HexCoord, TerrainType>, width: i32, height: i32) {
    let mut rng = thread_rng();
    add_hills(tiles, width, height, &mut rng);
    add_deserts(tiles, width, height, &mut rng);
    add_forests_and_jungles(tiles, width, height, &mut rng);
    add_coastal_features(tiles, width, height);
    fill_mountain_enclosures(tiles, width, height);
}
