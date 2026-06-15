// Balanced team spawn placement: spread spawns apart and give similar access to walkable land.

use std::collections::{HashMap, HashSet, VecDeque};

use rand::Rng;

use crate::core::HexCoord;

/// How far to BFS when estimating local expansion room around a candidate site.
const ACCESS_RADIUS: i32 = 12;
/// When several sites tie, pick randomly among those within this fraction of the best score.
const SCORE_TIE_THRESHOLD: f64 = 0.98;

fn reachable_land_count(start: HexCoord, walkable: &HashSet<HexCoord>, radius: i32) -> usize {
    let mut visited = HashSet::from([start]);
    let mut queue = VecDeque::from([(start, 0)]);

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= radius {
            continue;
        }
        for neighbor in current.offset_neighbors() {
            if walkable.contains(&neighbor) && visited.insert(neighbor) {
                queue.push_back((neighbor, depth + 1));
            }
        }
    }

    visited.len()
}

/// 1.0 when Voronoi regions (by hex distance) over walkable tiles are equal size.
fn voronoi_balance(centers: &[HexCoord], walkable: &HashSet<HexCoord>) -> f64 {
    if centers.len() < 2 {
        return 1.0;
    }

    let mut counts = vec![0usize; centers.len()];
    for tile in walkable {
        let idx = centers
            .iter()
            .enumerate()
            .min_by_key(|(_, center)| tile.distance(center))
            .map(|(idx, _)| idx)
            .unwrap();
        counts[idx] += 1;
    }

    let min = *counts.iter().min().unwrap_or(&0);
    let max = *counts.iter().max().unwrap_or(&1);
    if max == 0 {
        1.0
    } else {
        min as f64 / max as f64
    }
}

fn site_score(
    site: HexCoord,
    chosen: &[HexCoord],
    access: usize,
    walkable: &HashSet<HexCoord>,
    min_distance: i32,
) -> f64 {
    if chosen.is_empty() {
        return access as f64;
    }

    let min_dist = chosen
        .iter()
        .map(|center| site.distance(center))
        .min()
        .unwrap_or(0);
    if min_dist < min_distance {
        return f64::NEG_INFINITY;
    }

    let mut tentative = chosen.to_vec();
    tentative.push(site);
    let balance = voronoi_balance(&tentative, walkable);

    min_dist as f64 * balance * (access as f64).sqrt()
}

fn pick_from_top_scorers(candidates: &[(HexCoord, f64)], rng: &mut impl Rng) -> HexCoord {
    let best = candidates
        .iter()
        .map(|(_, score)| *score)
        .fold(f64::NEG_INFINITY, f64::max);
    let cutoff = best * SCORE_TIE_THRESHOLD;
    let top: Vec<HexCoord> = candidates
        .iter()
        .filter(|(_, score)| *score >= cutoff)
        .map(|(coord, _)| *coord)
        .collect();
    top[rng.gen_range(0..top.len())]
}

fn pick_balanced_pair(
    walkable: &HashSet<HexCoord>,
    min_distance: i32,
    is_valid: &mut impl FnMut(HexCoord, &HashSet<HexCoord>) -> bool,
    occupied_for_nest: &impl Fn(HexCoord) -> Vec<HexCoord>,
    access_scores: &HashMap<HexCoord, usize>,
    rng: &mut impl Rng,
) -> Vec<HexCoord> {
    let mut best_pairs: Vec<(HexCoord, HexCoord, f64)> = Vec::new();
    let mut best_score = f64::NEG_INFINITY;

    let seeds: Vec<HexCoord> = walkable
        .iter()
        .copied()
        .filter(|coord| is_valid(*coord, &HashSet::new()))
        .collect();

    for a in &seeds {
        let mut occupied_a = HashSet::new();
        for tile in occupied_for_nest(*a) {
            occupied_a.insert(tile);
        }

        for b in &seeds {
            if b.q <= a.q && !(b.q == a.q && b.r > a.r) {
                continue;
            }
            if a.distance(b) < min_distance || !is_valid(*b, &occupied_a) {
                continue;
            }

            let dist = a.distance(b) as f64;
            let balance = voronoi_balance(&[*a, *b], walkable);
            let access_a = *access_scores.get(a).unwrap_or(&1) as f64;
            let access_b = *access_scores.get(b).unwrap_or(&1) as f64;
            let score = dist * balance * (access_a * access_b).sqrt();

            if score > best_score {
                best_score = score;
                best_pairs.clear();
                best_pairs.push((*a, *b, score));
            } else if (score - best_score).abs() < f64::EPSILON {
                best_pairs.push((*a, *b, score));
            }
        }
    }

    if best_pairs.is_empty() {
        return Vec::new();
    }

    let (a, b, _) = best_pairs[rng.gen_range(0..best_pairs.len())];
    vec![a, b]
}

fn pick_balanced_greedy(
    walkable: &HashSet<HexCoord>,
    num_teams: usize,
    min_distance: i32,
    is_valid: &mut impl FnMut(HexCoord, &HashSet<HexCoord>) -> bool,
    occupied_for_nest: &impl Fn(HexCoord) -> Vec<HexCoord>,
    access_scores: &HashMap<HexCoord, usize>,
    rng: &mut impl Rng,
) -> Vec<HexCoord> {
    let mut chosen = Vec::new();
    let mut occupied = HashSet::new();

    for _ in 0..num_teams {
        let scored: Vec<(HexCoord, f64)> = walkable
            .iter()
            .copied()
            .filter(|coord| is_valid(*coord, &occupied))
            .map(|coord| {
                let access = *access_scores.get(&coord).unwrap_or(&1);
                (coord, site_score(coord, &chosen, access, walkable, min_distance))
            })
            .filter(|(_, score)| score.is_finite())
            .collect();

        if scored.is_empty() {
            break;
        }

        let pick = pick_from_top_scorers(&scored, rng);
        chosen.push(pick);
        for tile in occupied_for_nest(pick) {
            occupied.insert(tile);
        }
    }

    chosen
}

/// Pick `num_teams` nest centers that are far apart and split walkable land fairly.
pub fn pick_balanced_spawn_centers(
    walkable: &HashSet<HexCoord>,
    num_teams: usize,
    min_distance: i32,
    mut is_valid: impl FnMut(HexCoord, &HashSet<HexCoord>) -> bool,
    occupied_for_nest: impl Fn(HexCoord) -> Vec<HexCoord>,
    rng: &mut impl Rng,
) -> Vec<HexCoord> {
    if num_teams == 0 {
        return Vec::new();
    }

    let access_scores: HashMap<HexCoord, usize> = walkable
        .iter()
        .map(|coord| (*coord, reachable_land_count(*coord, walkable, ACCESS_RADIUS)))
        .collect();

    if num_teams == 2 {
        let pair = pick_balanced_pair(
            walkable,
            min_distance,
            &mut is_valid,
            &occupied_for_nest,
            &access_scores,
            rng,
        );
        if pair.len() == 2 {
            return pair;
        }
    }

    let greedy = pick_balanced_greedy(
        walkable,
        num_teams,
        min_distance,
        &mut is_valid,
        &occupied_for_nest,
        &access_scores,
        rng,
    );
    if greedy.len() == num_teams {
        return greedy;
    }

    // Last resort: any valid site, still preferring distance + access.
    let mut fallback = Vec::new();
    let mut occupied = HashSet::new();
    for _ in 0..num_teams {
        let scored: Vec<(HexCoord, f64)> = walkable
            .iter()
            .copied()
            .filter(|coord| is_valid(*coord, &occupied))
            .map(|coord| {
                let access = *access_scores.get(&coord).unwrap_or(&1) as f64;
                let min_dist = fallback
                    .iter()
                    .map(|center: &HexCoord| coord.distance(center))
                    .min()
                    .unwrap_or(min_distance) as f64;
                (coord, min_dist * access.sqrt())
            })
            .collect();
        if scored.is_empty() {
            break;
        }
        let pick = pick_from_top_scorers(&scored, rng);
        fallback.push(pick);
        for tile in occupied_for_nest(pick) {
            occupied.insert(tile);
        }
    }
    fallback
}
