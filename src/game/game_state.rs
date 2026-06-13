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
const NEST_SPAWN_INTERVAL: f64 = 15.0;

const MOVEMENT_TIME: f64 = 1.0;

// Battle tick interval (in seconds) - time between eliminations in battle
const BATTLE_TICK: f64 = 2.0;

// Retreat time (in seconds) - minimum time in battle before retreat is allowed
const RETREAT_TIME: f64 = 1.0;

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
    battle_timers: HashMap<HexCoord, f64>, // Track last elimination time per contested tile
}

impl GameState {
    fn is_walkable_land(&self, coord: &HexCoord) -> bool {
        match self.map.get_tile(coord) {
            TerrainType::Water | TerrainType::ShallowWater | TerrainType::Mountain => false,
            _ => true,
        }
    }

    fn is_valid_nest_site(&self, position: HexCoord, occupied: &HashSet<HexCoord>) -> bool {
        if !self.is_walkable_land(&position) || occupied.contains(&position) {
            return false;
        }

        let nest = Nest::new(0, position, 0.0);
        nest.member_spawn_positions().into_iter().all(|coord| {
            self.is_walkable_land(&coord) && !occupied.contains(&coord)
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
        existing_nests: &[HexCoord],
        rng: &mut impl Rng,
    ) -> HexCoord {
        let valid_sites: Vec<HexCoord> = land_tiles
            .iter()
            .copied()
            .filter(|coord| {
                self.is_valid_nest_site(*coord, occupied)
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
        spawn_time: f64,
    ) -> (HashMap<usize, Player>, Vec<Nest>) {
        let mut players = HashMap::new();
        let mut nests: Vec<Nest> = Vec::new();
        let mut occupied = HashSet::new();

        for team in 0..self.num_teams {
            let existing_nest_positions: Vec<HexCoord> =
                nests.iter().map(|nest| nest.position).collect();
            let nest_position = self.pick_nest_position(
                land_tiles,
                &occupied,
                &existing_nest_positions,
                rng,
            );
            let nest = Nest::new(team, nest_position, spawn_time);
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

    fn update_nest_spawning(&mut self, current_time: f64) {
        let mut spawns: Vec<(usize, HexCoord)> = Vec::new();
        let mut spawned_teams: Vec<usize> = Vec::new();

        for nest in &self.nests {
            if nest.should_spawn(current_time, NEST_SPAWN_INTERVAL) {
                spawns.push((nest.team, nest.position));
                spawned_teams.push(nest.team);
            }
        }

        for (team, spawn_pos) in spawns {
            let player_id = self.next_player_id;
            self.next_player_id += 1;
            self.players.insert(player_id, Player::new(player_id, team, spawn_pos));
        }

        for nest in &mut self.nests {
            if spawned_teams.contains(&nest.team) {
                nest.mark_spawned(current_time);
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
            battle_timers: HashMap::new(),
        };

        let spawn_time = get_time();
        let (players, nests) = game_state.spawn_teams_from_nests(&land_tiles, &mut rng, spawn_time);
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
        println!("Q: Reset characters to new positions");
        println!("Space: Cycle between teams (current: Team {})", 0);
        println!("\n=== SPLITTING STACKS ===");
        println!("S: Select half of stack (rounded up)");
        println!("D: Select just one character");
        println!("1-9: Select that many characters");
        println!("\n=== CAMERA ===");
        println!("Arrow keys / WASD: Pan camera");
        println!("+/-: Zoom in/out");
        println!("0: Reset zoom");
        println!("\n=== BATTLE ===");
        println!("Movement: {:.1} second(s) per tile", MOVEMENT_TIME);
        println!("Battle tick: {:.1} second(s) between eliminations", BATTLE_TICK);
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
        let spawn_time = get_time();
        let (players, nests) = self.spawn_teams_from_nests(&land_tiles, &mut rng, spawn_time);
        self.players = players;
        self.nests = nests;
        self.selected_player_ids.clear();
        self.battle_timers.clear();
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

        self.update_nest_spawning(current_time);
        
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
        
        for battle_coord in &battles {
            let is_new_battle = !self.battle_timers.contains_key(battle_coord);
            if is_new_battle {
                self.battle_timers.insert(*battle_coord, current_time);
            }
            
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
            
            let last_tick = self.battle_timers.get(battle_coord).copied().unwrap();
            
            if current_time - last_tick >= BATTLE_TICK {
                let mut teams_at_tile: HashMap<usize, Vec<usize>> = HashMap::new();
                
                for (id, player) in &self.players {
                    if player.position == *battle_coord {
                        teams_at_tile.entry(player.team).or_insert_with(Vec::new).push(*id);
                    }
                }
                
                for player_ids_at_tile in teams_at_tile.values() {
                    if !player_ids_at_tile.is_empty() {
                        let to_eliminate = player_ids_at_tile[0];
                        self.players.remove(&to_eliminate);
                        self.selected_player_ids.retain(|&id| id != to_eliminate);
                    }
                }
                
                self.battle_timers.insert(*battle_coord, current_time);
            }
        }
        
        let active_battles: HashSet<HexCoord> = battles.into_iter().collect();
        self.battle_timers.retain(|coord, _| active_battles.contains(coord));
        
        for player in self.players.values_mut() {
            if !active_battles.contains(&player.position) {
                player.combat_entry_time = None;
            }
        }
        
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
        
        // Handle Q key to reset dinos
        if is_key_pressed(KeyCode::Q) {
            self.reset_players();
        }
        
        // Handle spacebar to cycle teams
        if is_key_pressed(KeyCode::Space) {
            self.cycle_team();
        }
        
        // Handle selection modification keys (S, D, 1-9) for splitting stacks
        if !self.selected_player_ids.is_empty() {
            // S key - select half (rounded up)
            if is_key_pressed(KeyCode::S) {
                let current_count = self.selected_player_ids.len();
                let half = (current_count + 1) / 2; // Round up
                self.select_subset(half);
            }
            
            // D key - select just one
            if is_key_pressed(KeyCode::D) {
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

        // Draw team nests
        for nest in &self.nests {
            self.renderer.draw_nest(nest);
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