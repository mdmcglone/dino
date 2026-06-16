// Game state management

use macroquad::prelude::*;
use crate::maps::{PangaeaMap, Map, TerrainType};
use crate::rendering::HexMapRenderer;
use crate::input::{KeyboardHandler, MouseHandler};
use crate::game::{Nest, nest::{NEST_FARM_RADIUS, SIEGE_DINO_SECONDS_TARGET}, spawn_placement, team_abilities};
use crate::core::{HexCoord, OffsetFarmZone};
use ::rand::prelude::*;
use ::rand::seq::SliceRandom;
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;

const MEMBERS_PER_TEAM: usize = 3;
const STARTING_DINOS_PER_TEAM: usize = MEMBERS_PER_TEAM + 1;
const MIN_NEST_DISTANCE: i32 = 5;
const PLAYER_TEAMS: usize = team_abilities::PLAYER_TEAMS;
const KRONO_TEAM: usize = team_abilities::KRONO_TEAM;
const KRONO_HAZARD_COUNT: usize = 10;
const KRONO_CYCLE_MULTIPLIER: f64 = 0.5;
const KRONO_MOVEMENT_MULTIPLIER: f64 = 1.0 / 1.33;
const KRONO_HUNT_RANGE: i32 = 5;
const NESTLESS_ELIMINATION_TIME: f64 = 120.0;
const ROUGH_TERRAIN_ATTACK_CYCLE_MULTIPLIER: f64 = 1.2;
const ROUGH_TERRAIN_MOVEMENT_MULTIPLIER: f64 = 1.33;
const BASE_FOOD_CAP: f32 = 100.0;
const NEST_SIEGE_ATTACKER_CYCLE_MULTIPLIER: f64 = 1.5;
const FIRST_NEST_CREATION_COST: usize = 1;
const NEST_CREATION_COST_INCREMENT: usize = 5;

const MOVEMENT_TIME: f64 = 1.0;

// Base combat cycle (in seconds) — modifiers adjust each team's effective rate
const BASE_BATTLE_CYCLE: f64 = 2.0;
const ADVANTAGE_CYCLE_REDUCTION: f64 = 0.05;
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
    current_step_duration: f64,
    remaining_path: Vec<HexCoord>,
    remaining_durations: Vec<f64>,
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
        Self {
            id,
            team,
            position,
            selected: false,
            movement: None,
            waypoint_queue: Vec::new(),
            combat_entry_time: None,
        }
    }
    
    fn is_moving(&self) -> bool {
        self.movement.is_some()
    }
    
    fn start_path_movement(
        &mut self,
        path: Vec<HexCoord>,
        step_durations: Vec<f64>,
        current_time: f64,
        force_new: bool,
    ) {
        if path.is_empty() {
            return;
        }
        debug_assert_eq!(path.len(), step_durations.len());
        
        // Check if in combat and not enough time has passed for retreat
        if let Some(entry_time) = self.combat_entry_time {
            if current_time - entry_time < RETREAT_TIME {
                // Cannot retreat yet
                return;
            }
        }
        
        let mut path = path;
        let mut step_durations = step_durations;
        
        // Check if we're already moving
        if !force_new {
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
                    step_durations.remove(0);
                    // Replace the remaining path with the new one (don't append, replace)
                    movement.remaining_path = path;
                    movement.remaining_durations = step_durations;
                    return;
                }
            }
        }
        
        // Otherwise, start a new movement (cancels current movement)
        let first_step = path.remove(0);
        let first_duration = step_durations.remove(0);
        
        self.movement = Some(MovementState {
            current_step_destination: first_step,
            current_step_start_time: current_time,
            current_step_origin: self.position,
            current_step_duration: first_duration,
            remaining_path: path,
            remaining_durations: step_durations,
        });
    }

    fn cancel_movement(&mut self) {
        self.movement = None;
    }
    
    fn update_movement(&mut self, current_time: f64) -> bool {
        if let Some(movement) = &mut self.movement {
            if current_time - movement.current_step_start_time >= movement.current_step_duration {
                // Complete current step
                self.position = movement.current_step_destination;
                
                // Check if there are more steps in the path
                if !movement.remaining_path.is_empty() {
                    let next_step = movement.remaining_path.remove(0);
                    let next_duration = movement.remaining_durations.remove(0);
                    movement.current_step_origin = self.position;
                    movement.current_step_destination = next_step;
                    movement.current_step_start_time = current_time;
                    movement.current_step_duration = next_duration;
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
            let progress = ((current_time - movement.current_step_start_time) / movement.current_step_duration) as f32;
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

    fn has_planned_route(&self) -> bool {
        self.movement.is_some() || !self.waypoint_queue.is_empty()
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
    last_siege_update: f64,
    battle_states: HashMap<HexCoord, HashMap<usize, BattleTeamTimer>>,
    battle_defenders: HashMap<HexCoord, HashSet<usize>>,
    pre_battle_occupant: HashMap<HexCoord, usize>,
    show_controls: bool,
    nest_creations: HashMap<usize, usize>,
    eliminated_teams: HashSet<usize>,
    nestless_since: HashMap<usize, f64>,
    winner: Option<usize>,
    game_over_draw: bool,
}

impl GameState {
    fn is_walkable_land(&self, coord: &HexCoord) -> bool {
        if !self.is_on_map(coord) {
            return false;
        }
        match self.map.get_tile(coord) {
            TerrainType::Water | TerrainType::ShallowWater | TerrainType::Mountain => false,
            _ => true,
        }
    }

    fn is_on_map(&self, coord: &HexCoord) -> bool {
        self.map.get_tiles().contains_key(coord)
    }

    fn is_walkable_for_team(&self, coord: &HexCoord, team: usize) -> bool {
        if !self.is_on_map(coord) {
            return false;
        }
        if team_abilities::can_fly_over_terrain(team) {
            return true;
        }
        match self.map.get_tile(coord) {
            TerrainType::Mountain => false,
            TerrainType::Water => team == KRONO_TEAM,
            TerrainType::ShallowWater => true,
            _ => team != KRONO_TEAM,
        }
    }

    fn movement_step_duration(&self, from: &HexCoord, to: &HexCoord, team: usize) -> f64 {
        let crossing_rough_boundary =
            self.map.get_tile(from).is_rough() != self.map.get_tile(to).is_rough();
        let mut duration = if crossing_rough_boundary
            && !team_abilities::ignores_rough_terrain_movement_penalty(team)
        {
            MOVEMENT_TIME * ROUGH_TERRAIN_MOVEMENT_MULTIPLIER
        } else {
            MOVEMENT_TIME
        };
        if Self::is_hazard_team(team) {
            duration *= KRONO_MOVEMENT_MULTIPLIER;
        }
        duration
    }

    fn build_step_durations(&self, origin: HexCoord, path: &[HexCoord], team: usize) -> Vec<f64> {
        let mut durations = Vec::with_capacity(path.len());
        let mut from = origin;
        for step in path {
            durations.push(self.movement_step_duration(&from, step, team));
            from = *step;
        }
        durations
    }

    fn is_player_team(team: usize) -> bool {
        team < PLAYER_TEAMS
    }

    fn standing_player_teams(&self) -> Vec<usize> {
        (0..PLAYER_TEAMS)
            .filter(|team| !self.eliminated_teams.contains(team))
            .collect()
    }

    fn nestless_seconds_remaining(&self, team: usize, current_time: f64) -> Option<f64> {
        if self.eliminated_teams.contains(&team) || self.team_nest_count(team) > 0 {
            return None;
        }
        let since = self.nestless_since.get(&team).copied()?;
        Some((NESTLESS_ELIMINATION_TIME - (current_time - since)).max(0.0))
    }

    fn eliminate_team(&mut self, team: usize, reason: &str) {
        if !Self::is_player_team(team) || !self.eliminated_teams.insert(team) {
            return;
        }
        self.players.retain(|_, player| player.team != team);
        self.selected_player_ids.retain(|id| self.players.contains_key(id));
        self.nestless_since.remove(&team);
        if self.current_team == team {
            if let Some(&next) = self.standing_player_teams().first() {
                self.current_team = next;
                self.deselect_all();
            }
        }
        println!("Team {} eliminated — {}", team_abilities::team_name(team), reason);
    }

    fn update_elimination_and_victory(&mut self, current_time: f64) {
        if self.winner.is_some() || self.game_over_draw {
            return;
        }

        for team in 0..PLAYER_TEAMS {
            if self.eliminated_teams.contains(&team) {
                continue;
            }

            if self.team_dino_count(team) == 0 {
                self.eliminate_team(team, "no dinos remaining");
                continue;
            }

            if self.team_nest_count(team) > 0 {
                self.nestless_since.remove(&team);
                continue;
            }

            self.nestless_since.entry(team).or_insert(current_time);
            let since = self.nestless_since[&team];
            if current_time - since >= NESTLESS_ELIMINATION_TIME {
                self.eliminate_team(
                    team,
                    &format!("no nest held for {:.0} seconds", NESTLESS_ELIMINATION_TIME),
                );
            }
        }

        let standing = self.standing_player_teams();
        if standing.len() == 1 {
            self.winner = Some(standing[0]);
            println!("{} wins!", team_abilities::team_name(standing[0]));
        } else if standing.is_empty() {
            self.game_over_draw = true;
            println!("Draw — all teams eliminated");
        }
    }

    fn reset_victory_state(&mut self) {
        self.eliminated_teams.clear();
        self.nestless_since.clear();
        self.winner = None;
        self.game_over_draw = false;
    }

    fn is_hazard_team(team: usize) -> bool {
        team == KRONO_TEAM
    }

    fn is_water_tile(&self, coord: &HexCoord) -> bool {
        self.is_on_map(coord)
            && matches!(
                self.map.get_tile(coord),
                TerrainType::Water | TerrainType::ShallowWater
            )
    }

    fn deep_water_tiles(&self) -> Vec<HexCoord> {
        self.map
            .get_tiles()
            .iter()
            .filter(|(_, terrain)| **terrain == TerrainType::Water)
            .map(|(coord, _)| *coord)
            .collect()
    }

    fn apply_team_combat_cycle_modifier(
        &self,
        cycle: f64,
        team: usize,
        battle_coord: HexCoord,
    ) -> f64 {
        let mut cycle = self.apply_nest_siege_combat_penalty(cycle, battle_coord, team);
        if Self::is_hazard_team(team) {
            cycle *= KRONO_CYCLE_MULTIPLIER;
        } else {
            cycle *= team_abilities::combat_cycle_multiplier(team);
        }
        if !self.map.get_tile(&battle_coord).is_rough() {
            return cycle;
        }
        let is_defender = self
            .battle_defenders
            .get(&battle_coord)
            .map(|defenders| defenders.contains(&team))
            .unwrap_or(false);
        if !is_defender {
            cycle *= ROUGH_TERRAIN_ATTACK_CYCLE_MULTIPLIER;
        }
        cycle
    }

    fn record_battle_defenders(&mut self, battle_coord: HexCoord, teams_present: &HashMap<usize, usize>) {
        let mut defenders = HashSet::new();
        if let Some(&occupant) = self.pre_battle_occupant.get(&battle_coord) {
            if teams_present.contains_key(&occupant) {
                defenders.insert(occupant);
            }
        }
        self.battle_defenders.insert(battle_coord, defenders);
    }

    fn update_pre_battle_occupancy(&mut self) {
        let mut teams_at: HashMap<HexCoord, HashSet<usize>> = HashMap::new();
        for player in self.players.values() {
            teams_at
                .entry(player.position)
                .or_default()
                .insert(player.team);
        }
        self.pre_battle_occupant = teams_at
            .into_iter()
            .filter(|(_, teams)| teams.len() == 1)
            .map(|(coord, teams)| (coord, *teams.iter().next().unwrap()))
            .collect();
    }

    fn find_krono_hunt_target(&self, krono_pos: HexCoord) -> Option<HexCoord> {
        self.players
            .values()
            .filter(|player| !Self::is_hazard_team(player.team))
            .filter(|player| self.is_water_tile(&player.position))
            .filter(|player| krono_pos.distance(&player.position) <= KRONO_HUNT_RANGE)
            .min_by_key(|player| krono_pos.distance(&player.position))
            .map(|player| player.position)
    }

    fn claimed_territory_except(&self, exclude_index: usize) -> HashSet<HexCoord> {
        self.nests
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != exclude_index)
            .flat_map(|(_, nest)| nest.farm_within().iter().copied())
            .collect()
    }

    fn nest_index_at(&self, coord: &HexCoord) -> Option<usize> {
        self.nests.iter().position(|nest| nest.position == *coord)
    }

    fn apply_nest_siege_combat_penalty(&self, cycle: f64, battle_coord: HexCoord, team: usize) -> f64 {
        if let Some(index) = self.nest_index_at(&battle_coord) {
            if self.nests[index].team != team {
                return cycle * NEST_SIEGE_ATTACKER_CYCLE_MULTIPLIER;
            }
        }
        cycle
    }

    fn capture_nest(&mut self, nest_index: usize, new_team: usize) {
        let position = self.nests[nest_index].position;
        let claimed = self.claimed_territory_except(nest_index);
        let farm_within = OffsetFarmZone::compute_claimed(&position, NEST_FARM_RADIUS, &claimed);
        let nest = &mut self.nests[nest_index];
        let old_team = nest.team;
        nest.team = new_team;
        nest.set_farm_within(farm_within);
        nest.food = 0.0;
        nest.reset_siege();
        println!(
            "Team {} captured nest at ({}, {}) from team {}",
            new_team, position.q, position.r, old_team
        );
    }

    fn visible_tiles_for_team(&self, team: usize) -> HashSet<HexCoord> {
        let mut visible = HashSet::new();

        for nest in self.nests.iter().filter(|nest| nest.team == team) {
            visible.extend(nest.farm_within().iter().copied());
        }

        let territory_adjacent: Vec<HexCoord> = visible
            .iter()
            .flat_map(|coord| coord.offset_neighbors())
            .collect();
        visible.extend(territory_adjacent);

        for player in self.players.values().filter(|player| player.team == team) {
            visible.insert(player.position);
            visible.extend(player.position.offset_neighbors());
        }

        visible
    }

    fn battle_coords(&self) -> HashSet<HexCoord> {
        let mut teams_at: HashMap<HexCoord, HashSet<usize>> = HashMap::new();
        for player in self.players.values() {
            teams_at.entry(player.position).or_default().insert(player.team);
        }
        teams_at
            .into_iter()
            .filter(|(_, teams)| teams.len() > 1)
            .map(|(coord, _)| coord)
            .collect()
    }

    fn update_sieges(&mut self, current_time: f64, battle_coords: &HashSet<HexCoord>) {
        if self.last_siege_update == 0.0 {
            self.last_siege_update = current_time;
            return;
        }

        let dt = current_time - self.last_siege_update;
        self.last_siege_update = current_time;
        if dt <= 0.0 {
            return;
        }

        let nest_positions: Vec<(usize, HexCoord, usize)> = self
            .nests
            .iter()
            .enumerate()
            .map(|(index, nest)| (index, nest.position, nest.team))
            .collect();

        let mut captures: Vec<(usize, usize)> = Vec::new();

        for (nest_index, nest_position, nest_team) in nest_positions {
            if battle_coords.contains(&nest_position) {
                continue;
            }

            let mut team_counts: HashMap<usize, usize> = HashMap::new();
            for player in self.players.values() {
                if player.position != nest_position || !player.is_stationary() {
                    continue;
                }
                *team_counts.entry(player.team).or_insert(0) += 1;
            }

            let defender_count = team_counts.get(&nest_team).copied().unwrap_or(0);
            let (attacker_team, attacker_count) = team_counts
                .iter()
                .filter(|(&team, _)| team != nest_team)
                .max_by_key(|(_, count)| *count)
                .map(|(&team, &count)| (team, count))
                .unwrap_or((nest_team, 0));

            let mut delta = 0.0f32;
            if attacker_count > 0 {
                delta += attacker_count as f32
                    * dt as f32
                    * team_abilities::siege_attack_rate_multiplier(attacker_team) as f32;
            }
            if defender_count > 0 {
                delta -= defender_count as f32
                    * dt as f32
                    * team_abilities::siege_repair_rate_multiplier(nest_team) as f32;
            }

            if delta == 0.0 {
                continue;
            }

            let nest = &mut self.nests[nest_index];
            if attacker_count > 0 {
                nest.siege_team = Some(attacker_team);
            }

            nest.siege_progress = (nest.siege_progress + delta).max(0.0);

            if nest.siege_progress <= 0.0 {
                nest.reset_siege();
            } else if nest.siege_progress >= SIEGE_DINO_SECONDS_TARGET {
                if let Some(capturing_team) = nest.siege_team {
                    captures.push((nest_index, capturing_team));
                }
            }
        }

        for (nest_index, new_team) in captures {
            self.capture_nest(nest_index, new_team);
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

    fn spawn_initial_teams(
        &mut self,
        walkable: &HashSet<HexCoord>,
        rng: &mut impl Rng,
    ) -> (HashMap<usize, Player>, Vec<Nest>, Vec<(usize, HexCoord)>) {
        let mut players = HashMap::new();
        let mut spawn_centers: Vec<(usize, HexCoord)> = Vec::new();

        let centers = spawn_placement::pick_balanced_spawn_centers(
            walkable,
            self.num_teams,
            MIN_NEST_DISTANCE,
            |position, occupied_tiles| {
                self.is_valid_nest_site(position, occupied_tiles, &HashSet::new())
            },
            |position| Nest::new(0, position, &HashSet::new()).occupied_tiles(),
            rng,
        );

        for (team, spawn_center) in centers.into_iter().enumerate() {
            let staging = Nest::new(team, spawn_center, &HashSet::new());
            spawn_centers.push((team, spawn_center));

            for spawn_pos in staging.occupied_tiles() {
                let player_id = self.next_player_id;
                self.next_player_id += 1;
                players.insert(player_id, Player::new(player_id, team, spawn_pos));
            }
        }

        (players, Vec::new(), spawn_centers)
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

    fn nest_creation_cost(&self, team: usize) -> usize {
        let times_created = self.nest_creations.get(&team).copied().unwrap_or(0);
        if times_created == 0 {
            FIRST_NEST_CREATION_COST
        } else {
            NEST_CREATION_COST_INCREMENT * times_created
        }
    }

    fn try_create_nest(&mut self) {
        let team = self.current_team;
        let cost = self.nest_creation_cost(team);

        if self.selected_player_ids.len() < cost {
            return;
        }

        let team_dino_count = self.team_dino_count(team);
        if team_dino_count <= cost {
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
            .take(cost)
            .copied()
            .collect();
        for id in &to_remove {
            self.players.remove(id);
        }

        self.nests.push(preview);
        *self.nest_creations.entry(team).or_insert(0) += 1;
        self.selected_player_ids.retain(|id| self.players.contains_key(id));
        for id in &self.selected_player_ids {
            if let Some(player) = self.players.get_mut(id) {
                player.selected = true;
            }
        }

        println!(
            "Team {} nest established at ({}, {}) — next nest costs {} dinos",
            team, position.q, position.r, self.nest_creation_cost(team),
        );
    }

    fn team_dino_count(&self, team: usize) -> usize {
        self.players.values().filter(|player| player.team == team).count()
    }

    fn team_nest_count(&self, team: usize) -> usize {
        self.nests.iter().filter(|nest| nest.team == team).count()
    }

    fn team_population_cap(&self, team: usize) -> usize {
        self.team_nest_count(team) * team_abilities::population_per_nest(team)
    }

    fn can_spawn_dino(&self, team: usize) -> bool {
        self.team_dino_count(team) < self.team_population_cap(team)
    }

    fn spawn_dino_at(&mut self, team: usize, position: HexCoord) -> bool {
        if !self.can_spawn_dino(team) {
            return false;
        }
        let player_id = self.next_player_id;
        self.next_player_id += 1;
        self.players.insert(player_id, Player::new(player_id, team, position));
        true
    }

    fn nest_food_economy_active(&self, nest: &Nest, battle_coords: &HashSet<HexCoord>) -> bool {
        self.can_spawn_dino(nest.team)
            && !battle_coords.contains(&nest.position)
            && !nest.has_siege_damage()
    }

    fn update_food_gathering(&mut self, current_time: f64, battle_coords: &HashSet<HexCoord>) {
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
            if Self::is_hazard_team(player.team) || !player.is_stationary() {
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
            if !self.nest_food_economy_active(nest, battle_coords) {
                continue;
            }

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

        for (index, gain) in food_gains {
            self.nests[index].food += gain;
        }

        let clamp_indices: Vec<usize> = self
            .nests
            .iter()
            .enumerate()
            .filter(|(_, nest)| !self.nest_food_economy_active(nest, battle_coords))
            .map(|(index, _)| index)
            .collect();

        for index in clamp_indices {
            self.nests[index].food = self.nests[index].food.min(BASE_FOOD_CAP - 0.01);
        }

        for index in 0..self.nests.len() {
            let nest = &self.nests[index];
            if !self.nest_food_economy_active(nest, battle_coords) {
                continue;
            }
            while self.nests[index].food >= BASE_FOOD_CAP {
                let team = self.nests[index].team;
                let position = self.nests[index].position;
                if !self.can_spawn_dino(team) {
                    break;
                }
                self.nests[index].food -= BASE_FOOD_CAP;
                self.spawn_dino_at(team, position);
            }
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
        let target = self
            .players
            .iter()
            .find(|(_, player)| player.position == coord && player.team != killer_team)
            .map(|(id, player)| (*id, player.team));

        if let Some((target_id, target_team)) = target {
            self.players.remove(&target_id);
            self.selected_player_ids.retain(|&id| id != target_id);
            if Self::is_hazard_team(target_team) {
                self.respawn_krono();
            }
        }
    }

    fn init_battle_team_timer(
        &self,
        team_counts: &HashMap<usize, usize>,
        team: usize,
        battle_coord: HexCoord,
        current_time: f64,
    ) -> BattleTeamTimer {
        let team_count = team_counts.get(&team).copied().unwrap_or(0);
        let enemy_count = Self::primary_enemy_count(team_counts, team);
        let cycle = compute_combat_cycle(BASE_BATTLE_CYCLE, team_count, enemy_count);
        let cycle = self.apply_team_combat_cycle_modifier(cycle, team, battle_coord);
        BattleTeamTimer::new(current_time, cycle)
    }

    fn sync_battle_state(&mut self, battle_coord: HexCoord, current_time: f64) {
        let team_counts = self.team_counts_at(battle_coord);
        let is_new = !self.battle_states.contains_key(&battle_coord);

        if is_new {
            let team_timers = team_counts
                .keys()
                .map(|&team| (team, self.init_battle_team_timer(&team_counts, team, battle_coord, current_time)))
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
            .map(|team| (team, self.init_battle_team_timer(&team_counts, team, battle_coord, current_time)))
            .collect();

        let state = self.battle_states.get_mut(&battle_coord).unwrap();
        for (team, timer) in new_team_timers {
            state.insert(team, timer);
        }
        state.retain(|team, _| team_counts.contains_key(team));
    }

    fn process_battles(&mut self, battles: &HashSet<HexCoord>, current_time: f64) {
        for battle_coord in battles {
            let is_new_battle = !self.battle_states.contains_key(battle_coord);
            if is_new_battle {
                self.record_battle_defenders(*battle_coord, &self.team_counts_at(*battle_coord));
            }
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
                let next_cycle =
                    self.apply_team_combat_cycle_modifier(next_cycle, killer_team, *battle_coord);

                if let Some(timer) = self.battle_states.get_mut(battle_coord).and_then(|s| s.get_mut(&killer_team)) {
                    timer.start_next_cycle(current_time, next_cycle);
                }
            }
        }

        self.battle_states.retain(|coord, _| battles.contains(coord));
        self.battle_defenders.retain(|coord, _| battles.contains(coord));

        for player in self.players.values_mut() {
            if !battles.contains(&player.position) {
                player.combat_entry_time = None;
            }
        }
    }

    fn movement_route_for_player(&self, player: &Player) -> Vec<HexCoord> {
        let mut route = vec![player.position];

        if let Some(movement) = &player.movement {
            if route.last() != Some(&movement.current_step_destination) {
                route.push(movement.current_step_destination);
            }
            for tile in &movement.remaining_path {
                if route.last() != Some(tile) {
                    route.push(*tile);
                }
            }
        }

        let mut from = *route.last().unwrap_or(&player.position);
        for waypoint in &player.waypoint_queue {
            if let Some(path) = self.find_path(&from, waypoint, player.team) {
                for tile in path {
                    if route.last() != Some(&tile) {
                        route.push(tile);
                    }
                }
                from = *waypoint;
            } else {
                break;
            }
        }

        route
    }

    /// A* pathfinding algorithm to find the shortest path between two hexes
    fn find_path(&self, start: &HexCoord, goal: &HexCoord, team: usize) -> Option<Vec<HexCoord>> {
        if start == goal {
            return Some(vec![]);
        }

        if !self.is_walkable_for_team(goal, team) {
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

                if !self.is_walkable_for_team(&neighbor, team) {
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

    fn spawn_krono_at(&mut self, coord: HexCoord) {
        let player_id = self.next_player_id;
        self.next_player_id += 1;
        self.players
            .insert(player_id, Player::new(player_id, KRONO_TEAM, coord));
    }

    fn respawn_krono(&mut self) {
        let mut rng = thread_rng();
        let mut deep_water = self.deep_water_tiles();
        if deep_water.is_empty() {
            return;
        }
        deep_water.shuffle(&mut rng);
        self.spawn_krono_at(deep_water[0]);
    }

    fn spawn_krono_hazards(&mut self, rng: &mut impl Rng) {
        let mut deep_water = self.deep_water_tiles();
        if deep_water.is_empty() {
            return;
        }

        deep_water.shuffle(rng);
        let spawn_count = KRONO_HAZARD_COUNT.min(deep_water.len());

        for coord in deep_water.into_iter().take(spawn_count) {
            self.spawn_krono_at(coord);
        }
    }

    fn krono_movement_destination(&self, player: &Player) -> HexCoord {
        player
            .movement
            .as_ref()
            .map(|movement| {
                movement
                    .remaining_path
                    .last()
                    .copied()
                    .unwrap_or(movement.current_step_destination)
            })
            .unwrap_or(player.position)
    }

    fn update_krono_hazards(&mut self, current_time: f64, battle_coords: &HashSet<HexCoord>) {
        let mut rng = thread_rng();
        let hazard_ids: Vec<usize> = self
            .players
            .iter()
            .filter(|(_, player)| {
                player.team == KRONO_TEAM && !battle_coords.contains(&player.position)
            })
            .map(|(id, _)| *id)
            .collect();

        for id in hazard_ids {
            let position = self.players.get(&id).unwrap().position;

            if let Some(target) = self.find_krono_hunt_target(position) {
                if let Some(path) = self.find_path(&position, &target, KRONO_TEAM) {
                    let current_dest = self.krono_movement_destination(self.players.get(&id).unwrap());
                    if current_dest != target {
                        let durations = self.build_step_durations(position, &path, KRONO_TEAM);
                        if let Some(player) = self.players.get_mut(&id) {
                            player.start_path_movement(path, durations, current_time, true);
                        }
                    }
                    continue;
                }
            }

            let player = self.players.get(&id).unwrap();
            if player.is_moving() {
                continue;
            }

            let neighbors: Vec<HexCoord> = position
                .offset_neighbors()
                .into_iter()
                .filter(|coord| self.is_walkable_for_team(coord, KRONO_TEAM))
                .collect();
            if neighbors.is_empty() {
                continue;
            }

            let destination = neighbors[rng.gen_range(0..neighbors.len())];
            let path = vec![destination];
            let durations = self.build_step_durations(position, &path, KRONO_TEAM);
            if let Some(player) = self.players.get_mut(&id) {
                player.start_path_movement(path, durations, current_time, true);
            }
        }
    }
    
    pub fn new() -> Self {
        println!("\n=== PANGAEA ===");
        println!("Generating supercontinent...");
        
        let map = PangaeaMap::new();
        
        // Find random land tiles for spawning characters (exclude water and mountains)
        let mut rng = thread_rng();
        let walkable: HashSet<HexCoord> = map.get_tiles()
            .iter()
            .filter(|(_, terrain)| {
                **terrain != TerrainType::Water 
                && **terrain != TerrainType::ShallowWater 
                && **terrain != TerrainType::Mountain
            })
            .map(|(coord, _)| *coord)
            .collect();
        
        // Spawn each team at balanced positions across the map
        let num_teams = PLAYER_TEAMS;
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
            last_siege_update: 0.0,
            battle_states: HashMap::new(),
            battle_defenders: HashMap::new(),
            pre_battle_occupant: HashMap::new(),
            show_controls: true,
            nest_creations: HashMap::new(),
            eliminated_teams: HashSet::new(),
            nestless_since: HashMap::new(),
            winner: None,
            game_over_draw: false,
        };

        let (players, nests, spawn_centers) = game_state.spawn_initial_teams(&walkable, &mut rng);
        game_state.players = players;
        game_state.nests = nests;
        game_state.spawn_krono_hazards(&mut rng);

        println!("\nMap generated!");
        for (team, center) in spawn_centers {
            println!(
                "{} starts with {} dinos near ({}, {}) — place first nest with P (costs 1)",
                team_abilities::team_name(team), STARTING_DINOS_PER_TEAM, center.q, center.r,
            );
        }
        println!(
            "{} Kronos spawned in the oceans — avoid shallow water!",
            KRONO_HAZARD_COUNT,
        );
        println!("\n=== CONTROLS ===");
        println!("Left-click: Select character(s) / Move selected characters");
        println!("Right-click & drag: Rectangle select multiple characters");
        println!("SHIFT + Click: Queue multiple destinations");
        println!("P: Place nest (costs 1, then 5, 10, 15...)");
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
        println!("Movement: {:.1}s per tile ({:.2}s when entering/exiting rough terrain)", MOVEMENT_TIME, MOVEMENT_TIME * ROUGH_TERRAIN_MOVEMENT_MULTIPLIER);
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
        if self.standing_player_teams().len() <= 1 {
            return;
        }
        for _ in 0..self.num_teams {
            self.current_team = (self.current_team + 1) % self.num_teams;
            if !self.eliminated_teams.contains(&self.current_team) {
                println!("Switched to {}", team_abilities::team_name(self.current_team));
                return;
            }
        }
    }
    
    /// Get all player IDs at a specific tile (filtered by current team)
    fn get_players_at_tile(&self, coord: &HexCoord) -> Vec<usize> {
        self.players
            .iter()
            .filter(|(_, player)| player.position == *coord && player.team == self.current_team)
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// All selected units for the current team (movement commands use this, not just selected_player_ids).
    fn command_selected_player_ids(&self) -> Vec<usize> {
        self.players
            .iter()
            .filter(|(_, player)| {
                player.selected
                    && player.team == self.current_team
                    && !Self::is_hazard_team(player.team)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    fn selection_spans_multiple_tiles(&self) -> bool {
        let mut positions = HashSet::new();
        for id in self.command_selected_player_ids() {
            if let Some(player) = self.players.get(&id) {
                positions.insert(player.position);
            }
        }
        positions.len() > 1
    }

    fn sync_selected_player_ids(&mut self) {
        self.selected_player_ids = self.command_selected_player_ids();
    }
    
    /// Select all players at a given tile
    fn select_players_at_tile(&mut self, coord: &HexCoord) {
        self.deselect_all();
        self.selected_player_ids = self.get_players_at_tile(coord);
        for id in &self.selected_player_ids {
            if let Some(player) = self.players.get_mut(id) {
                player.selected = true;
            }
        }
    }

    fn add_players_at_tile_to_selection(&mut self, coord: &HexCoord) {
        for id in self.get_players_at_tile(coord) {
            if self.players.get(&id).is_some_and(|player| player.selected) {
                continue;
            }
            self.selected_player_ids.push(id);
            if let Some(player) = self.players.get_mut(&id) {
                player.selected = true;
            }
        }
    }

    fn issue_movement_orders(
        &mut self,
        clicked_hex: HexCoord,
        selected_ids: &[usize],
        queue_waypoints: bool,
        current_time: f64,
    ) {
        let mut movements: Vec<(usize, Option<Vec<HexCoord>>, HexCoord, usize)> = Vec::new();

        for &player_id in selected_ids {
            let Some(player) = self.players.get(&player_id) else { continue };
            let path_start = if queue_waypoints {
                if !player.waypoint_queue.is_empty() {
                    *player.waypoint_queue.last().unwrap()
                } else if let Some(movement) = &player.movement {
                    movement
                        .remaining_path
                        .last()
                        .copied()
                        .unwrap_or(movement.current_step_destination)
                } else {
                    player.position
                }
            } else {
                player.position
            };

            let path = self.find_path(&path_start, &clicked_hex, player.team);
            movements.push((player_id, path, player.position, player.team));
        }

        for (player_id, path, origin, team) in movements {
            let Some(path) = path else { continue };
            let durations = self.build_step_durations(origin, &path, team);
            let Some(player) = self.players.get_mut(&player_id) else { continue };
            if queue_waypoints {
                if path.is_empty() {
                    continue;
                }
                if player.is_moving() {
                    player.add_waypoint(clicked_hex);
                } else {
                    player.start_path_movement(path, durations, current_time, false);
                }
            } else {
                player.clear_waypoints();
                if path.is_empty() {
                    player.cancel_movement();
                    continue;
                }
                player.start_path_movement(path, durations, current_time, false);
            }
        }
        self.sync_selected_player_ids();
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
    
    /// Full game restart — respawns teams, clears match state, recenters the camera.
    fn restart_game(&mut self) {
        self.reset_victory_state();
        self.deselect_all();
        self.selection_box_start = None;
        self.selection_box_current = None;
        self.current_team = 0;

        let mut rng = thread_rng();
        let walkable: HashSet<HexCoord> = self.map.get_tiles()
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
        self.last_siege_update = get_time();
        self.players.clear();
        self.nests.clear();
        self.battle_states.clear();
        self.battle_defenders.clear();
        self.pre_battle_occupant.clear();
        self.nest_creations.clear();

        let (players, nests, spawn_centers) = self.spawn_initial_teams(&walkable, &mut rng);
        self.players = players;
        self.nests = nests;
        self.spawn_krono_hazards(&mut rng);

        self.renderer.reset_zoom();
        if let Some((_, center)) = spawn_centers.iter().find(|(team, _)| *team == 0) {
            self.renderer.center_camera_on(center);
        }

        println!("Game restarted — place your first nest with P (costs 1 dino)");
    }

    fn game_over_restart_requested(&self) -> bool {
        if self.winner.is_none() && !self.game_over_draw {
            return false;
        }
        is_key_pressed(KeyCode::Q) || is_mouse_button_pressed(MouseButton::Left)
    }
    
    fn selected_player_ids_at_tile(&self, coord: &HexCoord) -> Vec<usize> {
        self.selected_player_ids
            .iter()
            .copied()
            .filter(|id| {
                self.players
                    .get(id)
                    .is_some_and(|player| player.position == *coord)
            })
            .collect()
    }

    /// Adjust selection to only include N characters from the active understack
    /// (selected units sharing the first selected unit's tile).
    fn select_subset(&mut self, count: usize) {
        if self.selected_player_ids.is_empty() {
            return;
        }

        let Some(first_selected_pos) = self
            .players
            .get(&self.selected_player_ids[0])
            .map(|p| p.position)
        else {
            return;
        };

        let selected_at_pos = self.selected_player_ids_at_tile(&first_selected_pos);
        if selected_at_pos.is_empty() {
            return;
        }

        self.deselect_all();

        let to_select = count.min(selected_at_pos.len());
        self.selected_player_ids = selected_at_pos.into_iter().take(to_select).collect();

        for id in &self.selected_player_ids {
            if let Some(player) = self.players.get_mut(id) {
                player.selected = true;
            }
        }
    }
    
    pub fn update(&mut self) -> bool {
        let current_time = get_time();

        if is_key_pressed(KeyCode::Escape) {
            return true;
        }

        if self.winner.is_some() || self.game_over_draw {
            if self.game_over_restart_requested() {
                self.restart_game();
            }
            return false;
        }

        self.update_elimination_and_victory(current_time);
        if self.winner.is_some() || self.game_over_draw {
            return false;
        }

        let battle_coords = self.battle_coords();

        self.update_food_gathering(current_time, &battle_coords);
        self.update_sieges(current_time, &battle_coords);
        
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
        
        // Phase 2: stop pathing for participants entering combat
        self.process_battles(&battle_coords, current_time);

        self.update_krono_hazards(current_time, &battle_coords);
        
        // Phase 3: waypoint continuation and auto-select (after combat stops pathing)
        let mut positions_to_auto_select: Vec<HexCoord> = Vec::new();
        
        for (player_id, movement_complete, player_pos, was_selected, player_team) in movement_results {
            if movement_complete && was_selected {
                if let Some(player) = self.players.get(&player_id) {
                    if !player.has_waypoints() && !self.selection_spans_multiple_tiles() {
                        let players_at_pos = self.get_players_at_tile(&player_pos);
                        if players_at_pos.len() > 1 {
                            positions_to_auto_select.push(player_pos);
                        }
                    }
                }
            }
            
            if movement_complete {
                if Self::is_hazard_team(player_team) {
                    continue;
                }
                let (from_pos, next_waypoint) = match self.players.get(&player_id) {
                    Some(player) => match player.peek_next_waypoint() {
                        Some(waypoint) => (player.position, waypoint),
                        None => continue,
                    },
                    None => continue,
                };
                if let Some(path) = self.find_path(&from_pos, &next_waypoint, player_team) {
                    let durations = self.build_step_durations(from_pos, &path, player_team);
                    if let Some(player) = self.players.get_mut(&player_id) {
                        player.get_next_waypoint();
                        player.start_path_movement(path, durations, current_time, false);
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
                let selected_ids = self.command_selected_player_ids();
                let any_selected = !selected_ids.is_empty();
                let clicked_friendly_stack = !players_at_tile.is_empty();

                if shift_held && clicked_friendly_stack {
                    if any_selected {
                        self.add_players_at_tile_to_selection(&clicked_hex);
                    } else {
                        self.select_players_at_tile(&clicked_hex);
                    }
                } else if !any_selected && clicked_friendly_stack {
                    self.select_players_at_tile(&clicked_hex);
                } else if any_selected {
                    self.issue_movement_orders(clicked_hex, &selected_ids, shift_held, current_time);
                }
            }
        }
        
        // Handle P key to place a nest
        if is_key_pressed(KeyCode::P) && !self.eliminated_teams.contains(&self.current_team) {
            self.try_create_nest();
        }

        // Handle Q key to restart (mid-game debug reset)
        if is_key_pressed(KeyCode::Q) {
            self.restart_game();
        }

        if is_key_pressed(KeyCode::Z) {
            self.show_controls = !self.show_controls;
        }
        
        // Handle spacebar to cycle teams
        if is_key_pressed(KeyCode::Space) {
            self.cycle_team();
        }
        
        // Handle selection modification keys (E, R, 1-9) for splitting stacks
        if !self.selected_player_ids.is_empty() {
            let understack_size = self
                .players
                .get(&self.selected_player_ids[0])
                .map(|first| self.selected_player_ids_at_tile(&first.position).len())
                .unwrap_or(0);

            // E key - select half (rounded up) of the active understack
            if is_key_pressed(KeyCode::E) && understack_size > 0 {
                let half = (understack_size + 1) / 2;
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
        
        self.update_pre_battle_occupancy();

        // Handle keyboard input and return true if should exit
        self.keyboard_handler.handle_input(&mut self.renderer)
    }
    
    pub fn draw(&self) {
        let current_time = get_time();
        let visible = self.visible_tiles_for_team(self.current_team);
        
        // Clear screen with light blue-gray background
        clear_background(Color::new(0.85, 0.85, 0.9, 1.0));
        
        // Draw hex map with fog of war
        self.renderer.draw_map_with_fog(&self.map, &visible);

        // Draw nest farm zones, nests, and food bars
        for nest in &self.nests {
            self.renderer.draw_nest_farm_zone(nest, &visible);
        }
        for nest in &self.nests {
            if !visible.contains(&nest.position) {
                continue;
            }
            self.renderer.draw_nest(nest);
            if nest.has_siege_damage() {
                self.renderer.draw_nest_siege_bar(nest);
            } else {
                self.renderer.draw_nest_food_bar(nest, BASE_FOOD_CAP);
            }
        }
        
        // Group players by position
        let mut player_positions: HashMap<HexCoord, Vec<&Player>> = HashMap::new();
        for player in self.players.values() {
            player_positions.entry(player.position).or_insert_with(Vec::new).push(player);
        }
        
        // Draw players (grouped by team per position for battles)
        for (position, players_at_pos) in &player_positions {
            if !visible.contains(position) {
                continue;
            }
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
                
                let flip_x = is_battle && offset_factor < 0.0;
                self.renderer.draw_player_with_offset(position, *team_id, offset_factor, flip_x);
                
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
        }

        // Draw movement paths (current leg + queued waypoints)
        for player in self.players.values() {
            if Self::is_hazard_team(player.team) || !player.has_planned_route() {
                continue;
            }
            if !visible.contains(&player.position) {
                continue;
            }
            let route = self.movement_route_for_player(player);
            if route.len() < 2 {
                continue;
            }
            let active_progress = if player.is_moving() {
                player.get_movement_progress(current_time)
            } else {
                None
            };
            self.renderer.draw_movement_path(&route, active_progress);
        }
        
        // Draw selection box if active
        if let (Some(start), Some(current)) = (self.selection_box_start, self.selection_box_current) {
            self.renderer.draw_selection_box(start, current);
        }
        
        // Draw UI
        self.renderer.draw_ui(
            self.show_controls,
            self.current_team,
            self.team_dino_count(self.current_team),
            self.team_population_cap(self.current_team),
            self.nestless_seconds_remaining(self.current_team, current_time),
        );

        self.renderer.draw_game_over(self.winner, self.game_over_draw);
    }
} 