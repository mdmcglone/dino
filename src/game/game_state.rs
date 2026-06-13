// Game state management

use macroquad::prelude::*;
use crate::maps::{PangaeaMap, Map, TerrainType};
use crate::rendering::HexMapRenderer;
use crate::input::{KeyboardHandler, MouseHandler};
use crate::core::HexCoord;
use crate::game::Nest;
use ::rand::prelude::*;
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;

const MEMBERS_PER_TEAM: usize = 3;
const MIN_NEST_DISTANCE: i32 = 5;
const BASE_FOOD_CAP: f32 = 100.0;
const NEST_CREATION_DINO_COST: usize = 10;
const MIN_SELECTED_FOR_NEST: usize = 10;
const MIN_TEAM_DINOS_FOR_NEST: usize = 11;

const MOVEMENT_TIME: f64 = 1.0;

// Base combat cycle (in seconds) — modifiers adjust each team's effective rate
const BASE_BATTLE_CYCLE: f64 = 2.0;
const ADVANTAGE_CYCLE_REDUCTION: f64 = 0.10;
const MIN_CYCLE_MULTIPLIER: f64 = 0.1;

// Retreat time (in seconds) - minimum time in battle before retreat is allowed
const RETREAT_TIME: f64 = 1.0;

struct BattleTeamTimer {
    last_kill_time: f64,
    cycle_duration: f64,
}

impl BattleTeamTimer {
    fn new(current_time: f64, cycle_duration: f64) -> Self {
        Self { last_kill_time: current_time, cycle_duration }
    }

    fn start_next_cycle(&mut self, current_time: f64, cycle_duration: f64) {
        self.last_kill_time = current_time;
        self.cycle_duration = cycle_duration;
    }

    fn is_ready(&self, current_time: f64) -> bool {
        current_time - self.last_kill_time >= self.cycle_duration
    }
}

fn compute_combat_cycle(base: f64, team_count: usize, enemy_count: usize) -> f64 {
    let advantage = team_count.saturating_sub(enemy_count);
    if advantage == 0 {
        base
    } else {
        let multiplier = (1.0 - ADVANTAGE_CYCLE_REDUCTION * advantage as f64).max(MIN_CYCLE_MULTIPLIER);
        base * multiplier
    }
}

// A* pathfinding node
#[derive(Clone, Eq, PartialEq)]
struct PathNode {
    coord: HexCoord,
    g_cost: i32, // Cost from start
    h_cost: i32, // Heuristic cost to goal
}

impl PathNode {
    fn f_cost(&self) -> i32 {
        self.g_cost + self.h_cost
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost().cmp(&self.f_cost()).then_with(|| other.h_cost.cmp(&self.h_cost))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone)]
struct MovementState {
    current_step_destination: HexCoord,
    current_step_start_time: f64,
    current_step_origin: HexCoord,
    remaining_path: Vec<HexCoord>, // Queue of remaining tiles to visit
}

struct Player {
    id: usize,
    team: usize,
    position: HexCoord,
    selected: bool,
    movement: Option<MovementState>,
    waypoint_queue: Vec<HexCoord>, // Queue of destinations for shift-click chaining
    combat_entry_time: Option<f64>, // When this player entered combat (for retreat timer)
}

impl Player {
    fn new(id: usize, team: usize, position: HexCoord) -> Self {
        Self { id, team, position, selected: false, movement: None, waypoint_queue: Vec::new(), combat_entry_time: None }
    }
    
    fn is_moving(&self) -> bool {
        self.movement.is_some()
    }
    
    fn start_path_movement(&mut self, path: Vec<HexCoord>, current_time: f64) {
        if path.is_empty() {
            return;
        }
        
        // Check if in combat and not enough time has passed for retreat
        if let Some(entry_time) = self.combat_entry_time {
            if current_time - entry_time < RETREAT_TIME {
                // Cannot retreat yet
                return;
            }
        }
        
        let mut path = path;
        
        // Check if we're already moving
        if let Some(movement) = &mut self.movement {
            // Get the final destination of the new path
            let new_final_dest = path.last().copied();
            
            // Get our current final destination (last in remaining_path, or current_step_destination if no remaining)
            let current_final_dest = movement.remaining_path.last().copied()
                .unwrap_or(movement.current_step_destination);
            
            // If we're already going to the same final destination, ignore this command
            if new_final_dest == Some(current_final_dest) {
                return;
            }
            
            // If the first step of the new path is where we're currently headed, preserve progress
            if !path.is_empty() && path[0] == movement.current_step_destination {
                // Remove the first step (since we're already going there)
                path.remove(0);
                // Replace the remaining path with the new one (don't append, replace)
                movement.remaining_path = path;
                return;
            }
        }
        
        // Otherwise, start a new movement (cancels current movement)
        let first_step = path.remove(0);
        
        self.movement = Some(MovementState {
            current_step_destination: first_step,
            current_step_start_time: current_time,
            current_step_origin: self.position,
            remaining_path: path,
        });
    }
    
    fn update_movement(&mut self, current_time: f64) -> bool {
        if let Some(movement) = &mut self.movement {
            if current_time - movement.current_step_start_time >= MOVEMENT_TIME {
                // Complete current step
                self.position = movement.current_step_destination;
                
                // Check if there are more steps in the path
                if !movement.remaining_path.is_empty() {
                    let next_step = movement.remaining_path.remove(0);
                    movement.current_step_origin = self.position;
                    movement.current_step_destination = next_step;
                    movement.current_step_start_time = current_time;
                    return false; // Still moving
                } else {
                    // Path complete
                    self.movement = None;
                    return true; // Signals that we should check waypoint queue
                }
            }
        }
        false
    }
    
    fn has_waypoints(&self) -> bool {
        !self.waypoint_queue.is_empty()
    }
    
    fn peek_next_waypoint(&self) -> Option<HexCoord> {
        self.waypoint_queue.first().copied()
    }

    fn get_next_waypoint(&mut self) -> Option<HexCoord> {
        if !self.waypoint_queue.is_empty() {
            Some(self.waypoint_queue.remove(0))
        } else {
            None
        }
    }
    
    fn add_waypoint(&mut self, waypoint: HexCoord) {
        self.waypoint_queue.push(waypoint);
    }
    
    fn clear_waypoints(&mut self) {
        self.waypoint_queue.clear();
    }

    fn cancel_pathing(&mut self) {
        self.movement = None;
        self.waypoint_queue.clear();
    }
    
    fn get_movement_progress(&self, current_time: f64) -> Option<f32> {
        if let Some(movement) = &self.movement {
            let progress = ((current_time - movement.current_step_start_time) / MOVEMENT_TIME) as f32;
            Some(progress.min(1.0))
        } else {
            None
        }
    }
    
    fn is_stationary(&self) -> bool {
        !self.is_moving() && self.waypoint_queue.is_empty()
    }
    
    fn get_current_step(&self) -> Option<(&HexCoord, &HexCoord)> {
        self.movement.as_ref().map(|m| (&m.current_step_origin, &m.current_step_destination))
    }
}

pub struct GameState {
    map: PangaeaMap,
    renderer: HexMapRenderer,
    keyboard_handler: KeyboardHandler,
    mouse_handler: MouseHandler,
    players: HashMap<usize, Player>,
    selected_player_ids: Vec<usize>,
    selection_box_start: Option<(f32, f32)>,
    selection_box_current: Option<(f32, f32)>,
    current_team: usize,
    num_teams: usize,
    nests: Vec<Nest>,
    next_player_id: usize,
    last_food_update: f64,
    battle_states: HashMap<HexCoord, HashMap<usize, BattleTeamTimer>>,
}

impl GameState {
    fn is_walkable_land(&self, coord: &HexCoord) -> bool {
        match self.map.get_tile(coord) {
            TerrainType::Water | TerrainType::ShallowWater | TerrainType::Mountain => false,
            _ => true,
        }
    }

    fn claimed_territory(&self) -> HashSet<HexCoord> {
        self.nests
            .iter()
            .flat_map(|nest| nest.farm_within().iter().copied())
            .collect()
    }

    fn is_in_claimed_territory(&self, coord: &HexCoord) -> bool {
        self.nests.iter().any(|nest| nest.farm_within().contains(coord))
    }

    fn is_valid_nest_site(
        &self,
        position: HexCoord,
        occupied: &HashSet<HexCoord>,
        claimed: &HashSet<HexCoord>,
    ) -> bool {
        if claimed.contains(&position)
            || !self.is_walkable_land(&position)
            || occupied.contains(&position)
        {
            return false;
        }

        let nest = Nest::new(0, position, claimed);
        nest.member_spawn_positions().into_iter().all(|coord| {
            self.is_walkable_land(&coord)
                && !occupied.contains(&coord)
                && !claimed.contains(&coord)
        })
    }

    fn is_far_enough_from_other_nests(&self, position: HexCoord, existing_nests: &[HexCoord]) -> bool {
        existing_nests
            .iter()
            .all(|nest_pos| position.distance(nest_pos) >= MIN_NEST_DISTANCE)
    }

    fn pick_nest_position(
        &self,
        land_tiles: &[HexCoord],
        occupied: &HashSet<HexCoord>,
        claimed: &HashSet<HexCoord>,
        existing_nests: &[HexCoord],
        rng: &mut impl Rng,
    ) -> HexCoord {
        let valid_sites: Vec<HexCoord> = land_tiles
            .iter()
            .copied()
            .filter(|coord| {
                self.is_valid_nest_site(*coord, occupied, claimed)
                    && self.is_far_enough_from_other_nests(*coord, existing_nests)
            })
            .collect();

        if !valid_sites.is_empty() {
            valid_sites[rng.gen_range(0..valid_sites.len())]
        } else if !land_tiles.is_empty() {
            land_tiles[rng.gen_range(0..land_tiles.len())]
        } else {
            HexCoord::new(17, 17)
        }
    }

    fn spawn_teams_from_nests(
        &mut self,
        land_tiles: &[HexCoord],
        rng: &mut impl Rng,
    ) -> (HashMap<usize, Player>, Vec<Nest>) {
        let mut players = HashMap::new();
        let mut nests: Vec<Nest> = Vec::new();
        let mut occupied = HashSet::new();
        let mut claimed = HashSet::new();

        for team in 0..self.num_teams {
            let existing_nest_positions: Vec<HexCoord> =
                nests.iter().map(|nest| nest.position).collect();
            let nest_position = self.pick_nest_position(
                land_tiles,
                &occupied,
                &claimed,
                &existing_nest_positions,
                rng,
            );
            let nest = Nest::new(team, nest_position, &claimed);
            claimed.extend(nest.farm_within().iter().copied());
            for coord in nest.occupied_tiles() {
                occupied.insert(coord);
            }
            nests.push(nest);

            for spawn_pos in nests.last().unwrap().member_spawn_positions() {
                let player_id = self.next_player_id;
                self.next_player_id += 1;
                players.insert(player_id, Player::new(player_id, team, spawn_pos));
            }
        }

        (players, nests)
    }

    fn is_nest_tile(&self, coord: &HexCoord) -> bool {
        self.nests.iter().any(|nest| nest.position == *coord)
    }

    fn tile_food_rate(&self, coord: &HexCoord) -> f64 {
        if self.is_nest_tile(coord) {
            0.0
        } else {
            self.map.get_tile(coord).food_gather_rate()
        }
    }

    fn can_farm_tile(&self, coord: &HexCoord, team: usize) -> bool {
        if self.is_nest_tile(coord) {
            return false;
        }
        self.nests
            .iter()
            .any(|nest| nest.team == team && nest.is_in_farm_range(coord))
    }

    fn can_place_nest_at(&self, coord: &HexCoord) -> bool {
        match self.map.get_tile(coord) {
            TerrainType::Water | TerrainType::ShallowWater | TerrainType::Mountain => false,
            _ => !self.is_nest_tile(coord) && !self.is_in_claimed_territory(coord),
        }
    }

    fn try_create_nest(&mut self) {
        if self.selected_player_ids.len() < MIN_SELECTED_FOR_NEST {
            return;
        }

        let team = self.current_team;
        let team_dino_count = self.players.values().filter(|player| player.team == team).count();
        if team_dino_count < MIN_TEAM_DINOS_FOR_NEST {
            return;
        }

        let Some(first) = self.players.get(&self.selected_player_ids[0]) else {
            return;
        };
        let position = first.position;

        let selection_valid = self.selected_player_ids.iter().all(|id| {
            self.players
                .get(id)
                .map(|player| player.team == team && player.position == position)
                .unwrap_or(false)
        });
        if !selection_valid || !self.can_place_nest_at(&position) {
            return;
        }

        let claimed = self.claimed_territory();
        let preview = Nest::new(team, position, &claimed);
        if preview.farm_within().is_empty() {
            return;
        }

        let to_remove: Vec<usize> = self
            .selected_player_ids
            .iter()
            .take(NEST_CREATION_DINO_COST)
            .copied()
            .collect();
        for id in &to_remove {
            self.players.remove(id);
        }

        self.nests.push(preview);
        self.selected_player_ids.retain(|id| self.players.contains_key(id));
        for id in &self.selected_player_ids {
            if let Some(player) = self.players.get_mut(id) {
                player.selected = true;
            }
        }

        println!(
            "Team {} nest established at ({}, {})",
            team, position.q, position.r,
        );
    }

    fn spawn_dino_at(&mut self, team: usize, position: HexCoord) {
        let player_id = self.next_player_id;
        self.next_player_id += 1;
        self.players.insert(player_id, Player::new(player_id, team, position));
    }

    fn update_food_gathering(&mut self, current_time: f64) {
        if self.last_food_update == 0.0 {
            self.last_food_update = current_time;
            return;
        }

        let dt = current_time - self.last_food_update;
        self.last_food_update = current_time;
        if dt <= 0.0 {
            return;
        }

        let mut gathering_tiles: HashMap<usize, HashSet<HexCoord>> = HashMap::new();
        for player in self.players.values() {
            if !player.is_stationary() {
                continue;
            }
            if !self.can_farm_tile(&player.position, player.team) {
                continue;
            }
            gathering_tiles
                .entry(player.team)
                .or_default()
                .insert(player.position);
        }

        let mut food_gains: Vec<(usize, f32)> = Vec::new();

        for (index, nest) in self.nests.iter().enumerate() {
            let Some(tiles) = gathering_tiles.get(&nest.team) else {
                continue;
            };

            let gather_rate: f64 = tiles
                .iter()
                .filter(|coord| nest.is_in_farm_range(coord))
                .map(|coord| self.tile_food_rate(coord))
                .sum();

            if gather_rate > 0.0 {
                food_gains.push((index, (gather_rate * dt) as f32));
            }
        }

        let mut spawns: Vec<(usize, HexCoord)> = Vec::new();

        for (index, gain) in food_gains {
            let nest = &mut self.nests[index];
            nest.food += gain;

            while nest.food >= BASE_FOOD_CAP {
                nest.food -= BASE_FOOD_CAP;
                spawns.push((nest.team, nest.position));
            }
        }

        for (team, position) in spawns {
            self.spawn_dino_at(team, position);
        }
    }

    fn team_counts_at(&self, coord: HexCoord) -> HashMap<usize, usize> {
        let mut counts = HashMap::new();
        for player in self.players.values() {
            if player.position == coord {
                *counts.entry(player.team).or_insert(0) += 1;
            }
        }
        counts
    }

    fn primary_enemy_count(team_counts: &HashMap<usize, usize>, team: usize) -> usize {
        team_counts
            .iter()
            .filter(|(&t, &count)| t != team && count > 0)
            .map(|(_, &count)| count)
            .max()
            .unwrap_or(0)
    }

    fn eliminate_enemy_at(&mut self, coord: HexCoord, killer_team: usize) {
        if let Some(target_id) = self
            .players
            .iter()
            .find(|(_, player)| player.position == coord && player.team != killer_team)
            .map(|(id, _)| *id)
        {
            self.players.remove(&target_id);
            self.selected_player_ids.retain(|&id| id != target_id);
        }
    }

    fn init_battle_team_timer(
        &self,
        team_counts: &HashMap<usize, usize>,
        team: usize,
        current_time: f64,
    ) -> BattleTeamTimer {
        let team_count = team_counts.get(&team).copied().unwrap_or(0);
        let enemy_count = Self::primary_enemy_count(team_counts, team);
        let cycle = compute_combat_cycle(BASE_BATTLE_CYCLE, team_count, enemy_count);
        BattleTeamTimer::new(current_time, cycle)
    }

    fn sync_battle_state(&mut self, battle_coord: HexCoord, current_time: f64) {
        let team_counts = self.team_counts_at(battle_coord);
        let is_new = !self.battle_states.contains_key(&battle_coord);

        if is_new {
            let team_timers = team_counts
                .keys()
                .map(|&team| (team, self.init_battle_team_timer(&team_counts, team, current_time)))
                .collect();
            self.battle_states.insert(battle_coord, team_timers);
            return;
        }

        let new_team_timers: Vec<(usize, BattleTeamTimer)> = team_counts
            .keys()
            .copied()
            .filter(|team| {
                self.battle_states
                    .get(&battle_coord)
                    .map(|state| !state.contains_key(&team))
                    .unwrap_or(false)
            })
            .map(|team| (team, self.init_battle_team_timer(&team_counts, team, current_time)))
            .collect();

        let state = self.battle_states.get_mut(&battle_coord).unwrap();
        for (team, timer) in new_team_timers {
            state.insert(team, timer);
        }
        state.retain(|team, _| team_counts.contains_key(team));
    }

    fn process_battles(&mut self, battles: &[HexCoord], current_time: f64) {
        for battle_coord in battles {
            let is_new_battle = !self.battle_states.contains_key(battle_coord);
            self.sync_battle_state(*battle_coord, current_time);

            for player in self.players.values_mut() {
                if player.position != *battle_coord {
                    continue;
                }
                if is_new_battle || player.combat_entry_time.is_none() {
                    player.cancel_pathing();
                }
                if player.combat_entry_time.is_none() {
                    player.combat_entry_time = Some(current_time);
                }
            }

            let team_counts = self.team_counts_at(*battle_coord);
            let teams_present: Vec<usize> = team_counts.keys().copied().collect();
            let mut pending_kills: Vec<usize> = Vec::new();

            for team in &teams_present {
                if Self::primary_enemy_count(&team_counts, *team) == 0 {
                    continue;
                }
                if self
                    .battle_states
                    .get(battle_coord)
                    .and_then(|state| state.get(team))
                    .map(|timer| timer.is_ready(current_time))
                    .unwrap_or(false)
                {
                    pending_kills.push(*team);
                }
            }

            for killer_team in pending_kills {
                self.eliminate_enemy_at(*battle_coord, killer_team);

                let updated_counts = self.team_counts_at(*battle_coord);
                let team_count = updated_counts.get(&killer_team).copied().unwrap_or(0);
                let enemy_count = Self::primary_enemy_count(&updated_counts, killer_team);
                let next_cycle = compute_combat_cycle(BASE_BATTLE_CYCLE, team_count, enemy_count);

                if let Some(timer) = self.battle_states.get_mut(battle_coord).and_then(|s| s.get_mut(&killer_team)) {
                    timer.start_next_cycle(current_time, next_cycle);
                }
            }
        }

        let active_battles: HashSet<HexCoord> = battles.iter().copied().collect();
        self.battle_states.retain(|coord, _| active_battles.contains(coord));

        for player in self.players.values_mut() {
            if !active_battles.contains(&player.position) {
                player.combat_entry_time = None;
            }
        }
    }

    /// A* pathfinding algorithm to find the shortest path between two hexes
    fn find_path(&self, start: &HexCoord, goal: &HexCoord, _team: usize) -> Option<Vec<HexCoord>> {
        if start == goal {
            return Some(vec![]);
        }
        
        // Check if goal is walkable (teams can occupy same tiles for battles)
        let goal_terrain = self.map.get_tile(goal);
        if goal_terrain == TerrainType::Water || goal_terrain == TerrainType::Mountain {
            return None;
        }
        
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();
        let mut g_scores: HashMap<HexCoord, i32> = HashMap::new();
        
        g_scores.insert(*start, 0);
        open_set.push(PathNode {
            coord: *start,
            g_cost: 0,
            h_cost: start.distance(goal),
        });
        
        while let Some(current_node) = open_set.pop() {
            let current = current_node.coord;
            
            if current == *goal {
                // Reconstruct path
                let mut path = vec![current];
                let mut current_pos = current;
                
                while let Some(&prev) = came_from.get(&current_pos) {
                    path.push(prev);
                    current_pos = prev;
                }
                
                path.reverse();
                path.remove(0); // Remove starting position
                return Some(path);
            }
            
            if !closed_set.insert(current) {
                continue; // Already processed
            }
            
            let current_g = *g_scores.get(&current).unwrap_or(&i32::MAX);
            
            for neighbor in current.offset_neighbors() {
                if closed_set.contains(&neighbor) {
                    continue;
                }
                
                // Check if neighbor is walkable (teams can pass through each other)
                let terrain = self.map.get_tile(&neighbor);
                if terrain == TerrainType::Water || terrain == TerrainType::Mountain {
                    continue;
                }
                
                let tentative_g = current_g + 1;
                let neighbor_g = *g_scores.get(&neighbor).unwrap_or(&i32::MAX);
                
                if tentative_g < neighbor_g {
                    came_from.insert(neighbor, current);
                    g_scores.insert(neighbor, tentative_g);
                    
                    open_set.push(PathNode {
                        coord: neighbor,
                        g_cost: tentative_g,
                        h_cost: neighbor.distance(goal),
                    });
                }
            }
        }
        
        None // No path found
    }
    
    pub fn new() -> Self {
        println!("\n=== PANGAEA ===");
        println!("Generating supercontinent...");
        
        let map = PangaeaMap::new();
        
        // Find random land tiles for spawning characters (exclude water and mountains)
        let mut rng = thread_rng();
        let land_tiles: Vec<HexCoord> = map.get_tiles()
            .iter()
            .filter(|(_, terrain)| {
                **terrain != TerrainType::Water 
                && **terrain != TerrainType::ShallowWater 
                && **terrain != TerrainType::Mountain
            })
            .map(|(coord, _)| *coord)
            .collect();
        
        // Spawn each team around a randomly chosen nest
        let num_teams = 2;
        let mut game_state = Self {
            map,
            renderer: HexMapRenderer::new(),
            keyboard_handler: KeyboardHandler::new(),
            mouse_handler: MouseHandler::new(),
            players: HashMap::new(),
            selected_player_ids: Vec::new(),
            selection_box_start: None,
            selection_box_current: None,
            current_team: 0,
            num_teams,
            nests: Vec::new(),
            next_player_id: 0,
            last_food_update: 0.0,
            battle_states: HashMap::new(),
        };

        let (players, nests) = game_state.spawn_teams_from_nests(&land_tiles, &mut rng);
        game_state.players = players;
        game_state.nests = nests;

        println!("\nMap generated!");
        for nest in &game_state.nests {
            println!(
                "Team {} nest at ({}, {}) with {} members",
                nest.team, nest.position.q, nest.position.r, MEMBERS_PER_TEAM
            );
        }
        println!("\n=== CONTROLS ===");
        println!("Left-click: Select character(s) / Move selected characters");
        println!("Right-click & drag: Rectangle select multiple characters");
        println!("SHIFT + Click: Queue multiple destinations");
        println!("P: Place nest (10+ selected, 11+ team dinos)");
        println!("Space: Cycle between teams (current: Team {})", 0);
        println!("\n=== SPLITTING STACKS ===");
        println!("E: Select half of stack (rounded up)");
        println!("R: Select just one character");
        println!("1-9: Select that many characters");
        println!("\n=== CAMERA ===");
        println!("Arrow keys / WASD: Pan camera");
        println!("+/-: Zoom in/out");
        println!("0: Reset zoom");
        println!("\n=== BATTLE ===");
        println!("Movement: {:.1} second(s) per tile", MOVEMENT_TIME);
        println!("Base battle cycle: {:.1} second(s) between kills", BASE_BATTLE_CYCLE);
        println!("Retreat time: {:.1} second(s) minimum in combat", RETREAT_TIME);
        
        game_state
    }
    
    pub async fn load_team_sprite(&mut self, team: usize, path: &str) {
        self.renderer.load_team_sprite(team, path).await;
    }
    
    /// Add a new player to the game at the specified position
    pub fn add_player(&mut self, team: usize, position: HexCoord) -> usize {
        let new_id = self.next_player_id;
        self.next_player_id += 1;
        self.players.insert(new_id, Player::new(new_id, team, position));
        new_id
    }
    
    /// Cycle to the next team
    fn cycle_team(&mut self) {
        self.deselect_all();
        self.current_team = (self.current_team + 1) % self.num_teams;
        println!("Switched to Team {}", self.current_team);
    }
    
    /// Get all player IDs at a specific tile (filtered by current team)
    fn get_players_at_tile(&self, coord: &HexCoord) -> Vec<usize> {
        self.players
            .iter()
            .filter(|(_, player)| player.position == *coord && player.team == self.current_team)
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// Select all players at a given tile
    fn select_players_at_tile(&mut self, coord: &HexCoord) {
        self.selected_player_ids = self.get_players_at_tile(coord);
        for id in &self.selected_player_ids {
            if let Some(player) = self.players.get_mut(id) {
                player.selected = true;
            }
        }
    }
    
    /// Get all players whose tiles are inside the selection rectangle (filtered by current team)
    fn get_players_in_selection_box(&self) -> Vec<usize> {
        if self.selection_box_start.is_none() || self.selection_box_current.is_none() {
            return Vec::new();
        }
        
        let (start_x, start_y) = self.selection_box_start.unwrap();
        let (current_x, current_y) = self.selection_box_current.unwrap();
        
        // Calculate rectangle bounds
        let min_x = start_x.min(current_x);
        let max_x = start_x.max(current_x);
        let min_y = start_y.min(current_y);
        let max_y = start_y.max(current_y);
        
        let mut selected_ids = Vec::new();
        
        for (id, player) in &self.players {
            // Only select players from current team
            if player.team != self.current_team {
                continue;
            }
            
            let (px, py) = self.renderer.hex_to_pixel(&player.position);
            
            // Check if player's tile center is inside the rectangle
            if px >= min_x && px <= max_x && py >= min_y && py <= max_y {
                selected_ids.push(*id);
            }
        }
        
        selected_ids
    }
    
    /// Deselect all players
    fn deselect_all(&mut self) {
        self.selected_player_ids.clear();
        for player in self.players.values_mut() {
            player.selected = false;
        }
    }
    
    /// Reset all players to new nest positions
    fn reset_players(&mut self) {
        let mut rng = thread_rng();
        let land_tiles: Vec<HexCoord> = self.map.get_tiles()
            .iter()
            .filter(|(_, terrain)| {
                **terrain != TerrainType::Water 
                && **terrain != TerrainType::ShallowWater 
                && **terrain != TerrainType::Mountain
            })
            .map(|(coord, _)| *coord)
            .collect();

        self.next_player_id = 0;
        self.last_food_update = get_time();
        let (players, nests) = self.spawn_teams_from_nests(&land_tiles, &mut rng);
        self.players = players;
        self.nests = nests;
        self.selected_player_ids.clear();
        self.battle_states.clear();
        println!("Dinos reset to new nest positions!");
    }
    
    /// Adjust selection to only include N characters from the currently selected group
    fn select_subset(&mut self, count: usize) {
        if self.selected_player_ids.is_empty() {
            return;
        }
        
        // Get the position of the first selected player
        let first_selected_pos = self.players.get(&self.selected_player_ids[0])
            .map(|p| p.position);
        
        if let Some(pos) = first_selected_pos {
            // Get all players at that position
            let players_at_pos = self.get_players_at_tile(&pos);
            
            // Deselect all first
            self.deselect_all();
            
            // Select up to 'count' players from that position
            let to_select = count.min(players_at_pos.len());
            self.selected_player_ids = players_at_pos.into_iter().take(to_select).collect();
            
            // Mark them as selected
            for id in &self.selected_player_ids {
                if let Some(player) = self.players.get_mut(id) {
                    player.selected = true;
                }
            }
        }
    }
    
    pub fn update(&mut self) -> bool {
        let current_time = get_time();

        self.update_food_gathering(current_time);
        
        // Phase 1: advance movement only (battle checks happen before waypoint continuation)
        let player_ids: Vec<usize> = self.players.keys().copied().collect();
        let mut movement_results: Vec<(usize, bool, HexCoord, bool, usize)> = Vec::new();
        
        for player_id in &player_ids {
            let player = self.players.get_mut(player_id).unwrap();
            let complete = player.update_movement(current_time);
            movement_results.push((
                *player_id,
                complete,
                player.position,
                player.selected,
                player.team,
            ));
        }
        
        // Phase 2: detect battles and stop all pathing for participants entering combat
        let mut contested_tiles: HashMap<HexCoord, Vec<usize>> = HashMap::new();
        for &player_id in &player_ids {
            if let Some(player) = self.players.get(&player_id) {
                contested_tiles.entry(player.position).or_insert_with(Vec::new).push(player.team);
            }
        }
        
        let mut battles: Vec<HexCoord> = Vec::new();
        for (coord, teams) in &contested_tiles {
            let unique_teams: HashSet<usize> = teams.iter().copied().collect();
            if unique_teams.len() > 1 {
                battles.push(*coord);
            }
        }
        
        self.process_battles(&battles, current_time);
        
        // Phase 3: waypoint continuation and auto-select (after combat stops pathing)
        let mut positions_to_auto_select: Vec<HexCoord> = Vec::new();
        
        for (player_id, movement_complete, player_pos, was_selected, player_team) in movement_results {
            if movement_complete && was_selected {
                if let Some(player) = self.players.get(&player_id) {
                    if !player.has_waypoints() {
                        let players_at_pos = self.get_players_at_tile(&player_pos);
                        if players_at_pos.len() > 1 {
                            positions_to_auto_select.push(player_pos);
                        }
                    }
                }
            }
            
            if movement_complete {
                let (from_pos, next_waypoint) = match self.players.get(&player_id) {
                    Some(player) => match player.peek_next_waypoint() {
                        Some(waypoint) => (player.position, waypoint),
                        None => continue,
                    },
                    None => continue,
                };
                if let Some(path) = self.find_path(&from_pos, &next_waypoint, player_team) {
                    if let Some(player) = self.players.get_mut(&player_id) {
                        player.get_next_waypoint();
                        player.start_path_movement(path, current_time);
                    }
                }
            }
        }
        
        for pos in positions_to_auto_select {
            self.select_players_at_tile(&pos);
        }
        
        // Handle selection box (rectangle selection on right-click)
        let (mouse_x, mouse_y) = mouse_position();
        
        // Start selection box on right mouse down
        if is_mouse_button_pressed(MouseButton::Right) {
            self.deselect_all(); // Always deselect first
            self.selection_box_start = Some((mouse_x, mouse_y));
            self.selection_box_current = Some((mouse_x, mouse_y));
        }
        
        // Update selection box while dragging
        if is_mouse_button_down(MouseButton::Right) && self.selection_box_start.is_some() {
            self.selection_box_current = Some((mouse_x, mouse_y));
        }
        
        // Complete selection on mouse release
        if is_mouse_button_released(MouseButton::Right) && self.selection_box_start.is_some() {
            // Select all characters inside the rectangle
            let selected_ids = self.get_players_in_selection_box();
            if !selected_ids.is_empty() {
                self.selected_player_ids = selected_ids;
                for id in &self.selected_player_ids {
                    if let Some(player) = self.players.get_mut(id) {
                        player.selected = true;
                    }
                }
            }
            self.selection_box_start = None;
            self.selection_box_current = None;
        }
        
        // Handle mouse clicks for player selection and movement
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(clicked_hex) = self.mouse_handler.get_mouse_hex(&self.renderer) {
                let shift_held = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                
                // Check if there are any players at the clicked tile
                let players_at_tile = self.get_players_at_tile(&clicked_hex);
                let any_selected = !self.selected_player_ids.is_empty();
                
                if !players_at_tile.is_empty() && !any_selected {
                    // Clicking on a tile with players while nothing selected -> select all at that tile
                    self.select_players_at_tile(&clicked_hex);
                } else if any_selected {
                    // We have selected players - this is a movement command
                    // Calculate paths for each selected player
                    let mut movements: Vec<(usize, Option<Vec<HexCoord>>, HexCoord)> = Vec::new();
                    
                    for &player_id in &self.selected_player_ids {
                        if let Some(player) = self.players.get(&player_id) {
                            // Determine start position for pathfinding
                            let path_start = if shift_held {
                                // For shift-queue, start from last waypoint or destination
                                if !player.waypoint_queue.is_empty() {
                                    *player.waypoint_queue.last().unwrap()
                                } else if let Some(movement) = &player.movement {
                                    movement.remaining_path.last().copied()
                                        .unwrap_or(movement.current_step_destination)
                                } else {
                                    player.position
                                }
                            } else {
                                player.position
                            };
                            
                            let path = self.find_path(&path_start, &clicked_hex, player.team);
                            movements.push((player_id, path, player.position));
                        }
                    }
                    
                    // Apply movements to all selected players
                    for (player_id, path, _) in movements {
                        if let Some(player) = self.players.get_mut(&player_id) {
                            if shift_held {
                                // Shift-click: add to waypoint queue
                                if player.is_moving() {
                                    player.add_waypoint(clicked_hex);
                                } else if let Some(path) = path {
                                    if !path.is_empty() {
                                        player.start_path_movement(path, current_time);
                                    }
                                }
                            } else {
                                // Normal click: clear queue and move directly
                                player.clear_waypoints();
                                if let Some(path) = path {
                                    if !path.is_empty() {
                                        player.start_path_movement(path, current_time);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Handle P key to place a nest
        if is_key_pressed(KeyCode::P) {
            self.try_create_nest();
        }

        // Handle Q key to reset dinos
        if is_key_pressed(KeyCode::Q) {
            self.reset_players();
        }
        
        // Handle spacebar to cycle teams
        if is_key_pressed(KeyCode::Space) {
            self.cycle_team();
        }
        
        // Handle selection modification keys (E, R, 1-9) for splitting stacks
        if !self.selected_player_ids.is_empty() {
            // E key - select half (rounded up)
            if is_key_pressed(KeyCode::E) {
                let current_count = self.selected_player_ids.len();
                let half = (current_count + 1) / 2; // Round up
                self.select_subset(half);
            }
            
            // R key - select just one
            if is_key_pressed(KeyCode::R) {
                self.select_subset(1);
            }
            
            // Number keys 1-9
            if is_key_pressed(KeyCode::Key1) {
                self.select_subset(1);
            }
            if is_key_pressed(KeyCode::Key2) {
                self.select_subset(2);
            }
            if is_key_pressed(KeyCode::Key3) {
                self.select_subset(3);
            }
            if is_key_pressed(KeyCode::Key4) {
                self.select_subset(4);
            }
            if is_key_pressed(KeyCode::Key5) {
                self.select_subset(5);
            }
            if is_key_pressed(KeyCode::Key6) {
                self.select_subset(6);
            }
            if is_key_pressed(KeyCode::Key7) {
                self.select_subset(7);
            }
            if is_key_pressed(KeyCode::Key8) {
                self.select_subset(8);
            }
            if is_key_pressed(KeyCode::Key9) {
                self.select_subset(9);
            }
        }
        
        // Handle keyboard input and return true if should exit
        self.keyboard_handler.handle_input(&mut self.renderer)
    }
    
    pub fn draw(&self) {
        let current_time = get_time();
        
        // Clear screen with light blue-gray background
        clear_background(Color::new(0.85, 0.85, 0.9, 1.0));
        
        // Draw grid effect
        self.renderer.draw_grid_effect();
        
        // Draw hex map
        self.renderer.draw_map(&self.map);

        // Draw nest farm zones, nests, and food bars
        for nest in &self.nests {
            self.renderer.draw_nest_farm_zone(nest);
        }
        for nest in &self.nests {
            self.renderer.draw_nest(nest);
            self.renderer.draw_nest_food_bar(nest, BASE_FOOD_CAP);
        }
        
        // Group players by position
        let mut player_positions: HashMap<HexCoord, Vec<&Player>> = HashMap::new();
        for player in self.players.values() {
            player_positions.entry(player.position).or_insert_with(Vec::new).push(player);
        }
        
        // Draw players (grouped by team per position for battles)
        for (position, players_at_pos) in &player_positions {
            // Group players by team at this position
            let mut teams_at_pos: HashMap<usize, Vec<&Player>> = HashMap::new();
            for player in players_at_pos {
                teams_at_pos.entry(player.team).or_insert_with(Vec::new).push(*player);
            }
            
            // Sort teams by ID for consistent rendering (prevents flickering)
            let mut sorted_teams: Vec<(usize, Vec<&Player>)> = teams_at_pos.into_iter().collect();
            sorted_teams.sort_by_key(|(team_id, _)| *team_id);
            
            let num_teams_here = sorted_teams.len();
            let is_battle = num_teams_here > 1;
            
            // Draw selection highlight if any player at this position is selected
            let selected_count = players_at_pos.iter().filter(|p| p.selected).count();
            if selected_count > 0 {
                self.renderer.draw_selection_highlight(position);
            }
            
            // Draw sprites for each team at this position
            let mut team_index = 0;
            for (team_id, team_players) in &sorted_teams {
                let offset_factor = if is_battle {
                    // Side-by-side positioning: ±1.0 for full sprite width separation
                    (team_index as f32 - (num_teams_here - 1) as f32 / 2.0) * 1.0
                } else {
                    0.0
                };
                
                self.renderer.draw_player_with_offset(position, *team_id, offset_factor);
                
                // Draw count for this team if more than one
                if team_players.len() > 1 {
                    let team_selected = team_players.iter().filter(|p| p.selected).count();
                    let count_to_show = if team_selected > 0 { team_selected } else { team_players.len() };
                    self.renderer.draw_team_stack_count(position, *team_id, count_to_show, offset_factor);
                }
                
                team_index += 1;
            }
            
            // Draw battle indicator if multiple teams
            if is_battle {
                self.renderer.draw_battle_indicator(position);
            }
            
            // Draw movement arrow for the first moving player at this position
            if let Some(player) = players_at_pos.iter().find(|p| p.is_moving()) {
                if let Some((origin, destination)) = player.get_current_step() {
                    let progress = player.get_movement_progress(current_time).unwrap_or(0.0);
                    self.renderer.draw_movement_arrow(origin, destination, progress);
                }
            }
        }
        
        // Draw selection box if active
        if let (Some(start), Some(current)) = (self.selection_box_start, self.selection_box_current) {
            self.renderer.draw_selection_box(start, current);
        }
        
        // Draw UI
        self.renderer.draw_ui();
    }
} 