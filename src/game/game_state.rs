// Game state management

use macroquad::prelude::*;
use crate::maps::{PangaeaMap, Map, TerrainType};
use crate::rendering::HexMapRenderer;
use crate::input::{KeyboardHandler, MouseHandler};
use crate::core::HexCoord;
use ::rand::prelude::*;
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;

// Movement time constant (in seconds) - change this to adjust movement speed
const MOVEMENT_TIME: f64 = 1.0;

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
    position: HexCoord,
    selected: bool,
    movement: Option<MovementState>,
    waypoint_queue: Vec<HexCoord>, // Queue of destinations for shift-click chaining
}

impl Player {
    fn new(id: usize, position: HexCoord) -> Self {
        Self { id, position, selected: false, movement: None, waypoint_queue: Vec::new() }
    }
    
    fn is_moving(&self) -> bool {
        self.movement.is_some()
    }
    
    fn start_path_movement(&mut self, path: Vec<HexCoord>, current_time: f64) {
        if path.is_empty() {
            return;
        }
        
        let mut path = path;
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
    current_player_id: usize,
}

impl GameState {
    /// A* pathfinding algorithm to find the shortest path between two hexes
    fn find_path(&self, start: &HexCoord, goal: &HexCoord) -> Option<Vec<HexCoord>> {
        if start == goal {
            return Some(vec![]);
        }
        
        // Check if goal is walkable
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
                
                // Check if neighbor is walkable
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
        
        // Find a random land tile for the stick figure
        let mut rng = thread_rng();
        let land_tiles: Vec<HexCoord> = map.get_tiles()
            .iter()
            .filter(|(_, terrain)| **terrain != TerrainType::Water && **terrain != TerrainType::ShallowWater)
            .map(|(coord, _)| *coord)
            .collect();
        
        let stick_figure_pos = if !land_tiles.is_empty() {
            land_tiles[rng.gen_range(0..land_tiles.len())]
        } else {
            HexCoord::new(17, 17) // Fallback to center if no land found
        };
        
        // Initialize players HashMap with the first player
        let mut players = HashMap::new();
        players.insert(0, Player::new(0, stick_figure_pos));
        
        println!("\nMap generated!");
        println!("Left-click character to select, then click tiles to move!");
        println!("Hold SHIFT while clicking to queue multiple destinations!");
        println!("Right-click to deselect");
        println!("Movement takes {:.1} second(s) per tile!", MOVEMENT_TIME);
        println!("Use arrow keys to pan the camera, +/- to zoom");
        
        Self {
            map,
            renderer: HexMapRenderer::new(),
            keyboard_handler: KeyboardHandler::new(),
            mouse_handler: MouseHandler::new(),
            players,
            current_player_id: 0,
        }
    }
    
    pub async fn load_overlay(&mut self, path: &str) {
        self.renderer.load_overlay(path).await;
    }
    
    pub async fn load_player_sprite(&mut self, path: &str) {
        self.renderer.load_player_sprite(path).await;
    }
    
    /// Add a new player to the game at the specified position
    pub fn add_player(&mut self, position: HexCoord) -> usize {
        let new_id = self.players.len();
        self.players.insert(new_id, Player::new(new_id, position));
        new_id
    }
    
    /// Switch control to a different player
    pub fn set_current_player(&mut self, player_id: usize) {
        if self.players.contains_key(&player_id) {
            self.current_player_id = player_id;
        }
    }
    
    pub fn update(&mut self) -> bool {
        let current_time = get_time();
        
        // Update all players' movements and check for waypoint continuation
        let player_ids: Vec<usize> = self.players.keys().copied().collect();
        for player_id in player_ids {
            let (movement_complete, has_waypoint, next_waypoint, player_pos) = {
                let player = self.players.get_mut(&player_id).unwrap();
                let complete = player.update_movement(current_time);
                let waypoint = if complete && player.has_waypoints() {
                    player.get_next_waypoint()
                } else {
                    None
                };
                (complete, waypoint.is_some(), waypoint.unwrap_or(HexCoord::new(0, 0)), player.position)
            };
            
            // If movement completed and there's a next waypoint, calculate path and start movement
            if movement_complete && has_waypoint {
                if let Some(path) = self.find_path(&player_pos, &next_waypoint) {
                    if let Some(player) = self.players.get_mut(&player_id) {
                        player.start_path_movement(path, current_time);
                    }
                }
            }
        }
        
        // Handle right-click to deselect
        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some(player) = self.players.get_mut(&self.current_player_id) {
                player.selected = false;
            }
        }
        
        // Handle mouse clicks for current player
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(clicked_hex) = self.mouse_handler.get_mouse_hex(&self.renderer) {
                let shift_held = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                
                // First check player state without mutable borrow
                let should_select = self.players.get(&self.current_player_id)
                    .map(|p| clicked_hex == p.position && !p.selected)
                    .unwrap_or(false);
                
                let (should_move, player_pos, is_moving, is_selected) = self.players.get(&self.current_player_id)
                    .map(|p| (p.selected && clicked_hex != p.position, p.position, p.is_moving(), p.selected))
                    .unwrap_or((false, HexCoord::new(0, 0), false, false));
                
                // Determine the start position for pathfinding
                // If shift-queueing and player is moving, find path from final waypoint/destination
                let path_start = if shift_held && (is_moving || is_selected) {
                    // Get the last waypoint if exists, otherwise current destination or position
                    self.players.get(&self.current_player_id)
                        .and_then(|p| {
                            if !p.waypoint_queue.is_empty() {
                                p.waypoint_queue.last().copied()
                            } else if let Some(movement) = &p.movement {
                                // Get final destination in current path
                                movement.remaining_path.last().copied()
                                    .or(Some(movement.current_step_destination))
                            } else {
                                Some(p.position)
                            }
                        })
                        .unwrap_or(player_pos)
                } else {
                    player_pos
                };
                
                // Calculate path BEFORE getting mutable borrow
                let path = if should_move || (shift_held && is_selected) {
                    self.find_path(&path_start, &clicked_hex)
                } else {
                    None
                };
                
                // Now apply the changes with mutable borrow
                if let Some(player) = self.players.get_mut(&self.current_player_id) {
                    if should_select {
                        player.selected = true;
                    } else if should_move {
                        if shift_held {
                            // Shift-click: add to waypoint queue
                            if player.is_moving() {
                                // Already moving, add to waypoint queue
                                player.add_waypoint(clicked_hex);
                            } else if let Some(path) = path {
                                if !path.is_empty() {
                                    // Start first movement, stay selected
                                    player.start_path_movement(path, current_time);
                                }
                            }
                        } else {
                            // Normal click: clear queue and move directly, stay selected
                            player.clear_waypoints();
                            if let Some(path) = path {
                                if !path.is_empty() {
                                    player.start_path_movement(path, current_time);
                                }
                            }
                        }
                        // Keep player selected for chaining commands
                    }
                }
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
        
        // Draw all players
        for player in self.players.values() {
            // Draw selection highlight if selected
            if player.selected {
                self.renderer.draw_selection_highlight(&player.position);
            }
            
            // Draw the player at their current position
            self.renderer.draw_player(&player.position);
            
            // If moving, draw movement arrow animation for current step
            if let Some((origin, destination)) = player.get_current_step() {
                let progress = player.get_movement_progress(current_time).unwrap_or(0.0);
                self.renderer.draw_movement_arrow(origin, destination, progress);
            }
        }
        
        // Draw overlay on top
        self.renderer.draw_overlay();
        
        // Draw UI
        self.renderer.draw_ui();
    }
} 