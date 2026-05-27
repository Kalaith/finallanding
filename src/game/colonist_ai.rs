use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, Colonist, ColonistState, JobPreference};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::grid::Grid;
use crate::data::schedule::ActivityType;
use crate::data::simulation_rng::SimulationRng;
use crate::data::types::Position;
use crate::systems::job_decision_system::calculate_refusal_chance;
use crate::systems::mood_system::update_mood;
use crate::systems::time_system::TimeSystem;
use std::collections::HashMap;

/// Movement speed for visual interpolation (pixels per frame)
const VISUAL_MOVE_SPEED: f32 = 2.0;
const REFUSAL_LOG_COOLDOWN_TICKS: u64 = 60;
const SOCIAL_STRAIN_LOG_COOLDOWN_TICKS: u64 = 120;

type PendingLog = (LogCategory, String, String);
type SocialLocation = (u32, ActivityLocation);

#[derive(Clone, Copy, Debug)]
struct BuildingTarget {
    building_id: u32,
    building_type: BuildingType,
    entrance: Position,
}

pub fn update_colonists(state: &mut GameState, elapsed_ticks: u64) {
    if elapsed_ticks == 0 {
        for colonist in &mut state.colonists {
            colonist.update_visual_position(VISUAL_MOVE_SPEED);
        }
        return;
    }

    let (_, hour, _) = TimeSystem::get_time_of_day(state.tick);

    let occupied: HashMap<Position, u32> = state
        .colonists
        .iter()
        .filter(|c| !c.is_on_mission())
        .map(|c| (c.position, c.id))
        .collect();
    let colonist_names: HashMap<u32, String> = state
        .colonists
        .iter()
        .map(|colonist| (colonist.id, colonist.name.clone()))
        .collect();
    let social_locations: Vec<SocialLocation> = state
        .colonists
        .iter()
        .map(|colonist| (colonist.id, colonist.activity_location.clone()))
        .collect();

    let mut building_occupancy: HashMap<u32, u32> = HashMap::new();
    for c in &state.colonists {
        if c.is_on_mission() {
            continue;
        }

        if let Some(bid) = c.assigned_habitat {
            *building_occupancy.entry(bid).or_default() += 1;
        }
    }

    let buildings: Vec<(u32, BuildingType, Position, (u32, u32))> = state
        .building_system
        .buildings()
        .iter()
        .map(|b| (b.id, b.building_type, b.position, b.size()))
        .collect();
    let habitat_capacity = 2 + state.technology.habitat_capacity_bonus();
    let priority = state.priority.active;

    let mut pending_logs: Vec<PendingLog> = Vec::new();

    for i in 0..state.colonists.len() {
        let scheduled_activity = state.colonists[i].schedule.get_activity_for_hour(hour);

        update_colonist_ai(
            &mut state.colonists[i],
            &scheduled_activity,
            &occupied,
            &colonist_names,
            &social_locations,
            &state.grid,
            &mut state.rng,
            &buildings,
            &mut building_occupancy,
            habitat_capacity,
            state.tick,
            &mut pending_logs,
        );

        state.colonists[i].update_visual_position(VISUAL_MOVE_SPEED);
        update_mood(&mut state.colonists[i], elapsed_ticks, priority);
    }

    for (category, title, detail) in pending_logs {
        state.push_log(category, title, detail);
    }
}

fn update_colonist_ai(
    colonist: &mut Colonist,
    scheduled_activity: &ActivityType,
    occupied: &HashMap<Position, u32>,
    colonist_names: &HashMap<u32, String>,
    social_locations: &[SocialLocation],
    grid: &Grid,
    rng: &mut SimulationRng,
    buildings: &[(u32, BuildingType, Position, (u32, u32))],
    building_occupancy: &mut HashMap<u32, u32>,
    habitat_capacity: u32,
    current_tick: u64,
    pending_logs: &mut Vec<PendingLog>,
) {
    match colonist.state {
        ColonistState::OnMission { .. } => {
            colonist.activity_location = ActivityLocation::None;
            colonist.current_activity = ActivityType::Work;
        }
        ColonistState::Idle => {
            colonist.current_activity = scheduled_activity.clone();

            let target_building_type = match scheduled_activity {
                ActivityType::Sleep => find_or_assign_habitat(
                    colonist,
                    buildings,
                    building_occupancy,
                    habitat_capacity,
                    social_locations,
                    pending_logs,
                    current_tick,
                ),
                ActivityType::Work => {
                    let refusal_chance =
                        calculate_refusal_chance(colonist, colonist.job_preference);
                    if rng.range_f32(0.0, 100.0) < refusal_chance {
                        log_work_refusal(colonist, refusal_chance, current_tick, pending_logs);
                        None
                    } else {
                        Some(colonist.job_preference.work_building_type())
                    }
                }
                ActivityType::Eat => Some(BuildingType::MessHall),
                ActivityType::Relax => None,
            };

            if let Some(building_type) = target_building_type {
                let specific_target = specific_target_for_activity(
                    colonist,
                    scheduled_activity,
                    building_type,
                    buildings,
                );

                if let Some(target) = find_building_entrance(
                    colonist.position,
                    building_type,
                    buildings,
                    specific_target,
                    colonist,
                    social_locations,
                    grid,
                ) {
                    if is_adjacent_to_building(
                        colonist.position,
                        target.building_type,
                        buildings,
                        Some(target.building_id),
                    ) {
                        set_activity_at_building(
                            colonist,
                            scheduled_activity,
                            target.building_id,
                            target.building_type,
                        );
                    } else {
                        set_moving_to_activity(colonist, scheduled_activity, target.entrance);
                    }
                } else if rng.range_i32(0, 100) < 5 {
                    let target = find_wander_target(colonist.position, occupied, grid, rng);
                    set_moving_to_activity(colonist, &ActivityType::Relax, target);
                }
            } else if matches!(scheduled_activity, ActivityType::Relax) && rng.range_i32(0, 100) < 5
            {
                let target = find_wander_target(colonist.position, occupied, grid, rng);
                set_moving_to_activity(colonist, scheduled_activity, target);
            }
        }
        ColonistState::Moving { target } => {
            let next_pos = get_next_move_position(
                colonist,
                target,
                occupied,
                grid,
                colonist_names,
                current_tick,
                pending_logs,
            );
            colonist.position = next_pos;

            if colonist.position == target {
                if let Some(building_type) =
                    building_type_for_activity(scheduled_activity, colonist.job_preference)
                {
                    let specific_target = specific_target_for_activity(
                        colonist,
                        scheduled_activity,
                        building_type,
                        buildings,
                    );
                    if let Some((building_id, building_type)) = find_adjacent_building(
                        colonist.position,
                        building_type,
                        buildings,
                        specific_target,
                    ) {
                        set_activity_at_building(
                            colonist,
                            scheduled_activity,
                            building_id,
                            building_type,
                        );
                    } else {
                        set_idle(colonist);
                    }
                } else {
                    set_idle(colonist);
                }
            }
        }
        ColonistState::Working | ColonistState::Eating | ColonistState::Sleeping => {
            match (colonist.state, scheduled_activity) {
                (ColonistState::Working, ActivityType::Work) => {
                    colonist.current_activity = ActivityType::Work;
                }
                (ColonistState::Eating, ActivityType::Eat) => {
                    colonist.current_activity = ActivityType::Eat;
                }
                (ColonistState::Sleeping, ActivityType::Sleep) => {
                    colonist.current_activity = ActivityType::Sleep;
                }
                _ => set_idle(colonist),
            }
        }
    }
}

fn find_or_assign_habitat(
    colonist: &mut Colonist,
    buildings: &[(u32, BuildingType, Position, (u32, u32))],
    building_occupancy: &mut HashMap<u32, u32>,
    habitat_capacity: u32,
    social_locations: &[SocialLocation],
    pending_logs: &mut Vec<PendingLog>,
    current_tick: u64,
) -> Option<BuildingType> {
    if let Some(bid) = colonist.assigned_habitat {
        if building_matches_type(buildings, bid, BuildingType::Habitat) {
            return Some(BuildingType::Habitat);
        }

        colonist.assigned_habitat = None;
    }

    let mut best_habitat: Option<(u32, i32, u32)> = None;
    for (id, b_type, _, _) in buildings {
        if *b_type != BuildingType::Habitat {
            continue;
        }

        let count = *building_occupancy.get(id).unwrap_or(&0);
        if count >= habitat_capacity {
            continue;
        }

        let social_score = social_score_for_building(colonist, *id, social_locations);
        let candidate = (*id, social_score, count);
        if best_habitat
            .map(|best| better_habitat_candidate(candidate, best))
            .unwrap_or(true)
        {
            best_habitat = Some(candidate);
        }
    }

    if let Some((building_id, _, _)) = best_habitat {
        colonist.assigned_habitat = Some(building_id);
        *building_occupancy.entry(building_id).or_default() += 1;
        return Some(BuildingType::Habitat);
    }

    colonist.state = ColonistState::Sleeping;
    colonist.current_activity = ActivityType::Sleep;
    colonist.activity_location = ActivityLocation::Ground(colonist.position);

    if current_tick.saturating_sub(colonist.last_refusal_tick) >= REFUSAL_LOG_COOLDOWN_TICKS {
        colonist.last_refusal_tick = current_tick;
        pending_logs.push((
            LogCategory::Mood,
            format!("{} slept without a habitat", colonist.name),
            "Recovery is weaker when there are not enough usable habitats.".to_string(),
        ));
    }

    None
}

fn log_work_refusal(
    colonist: &mut Colonist,
    refusal_chance: f32,
    current_tick: u64,
    pending_logs: &mut Vec<PendingLog>,
) {
    colonist.current_activity = ActivityType::Work;
    colonist.activity_location = ActivityLocation::None;

    if current_tick.saturating_sub(colonist.last_refusal_tick) < REFUSAL_LOG_COOLDOWN_TICKS {
        return;
    }

    colonist.last_refusal_tick = current_tick;
    pending_logs.push((
        LogCategory::Work,
        format!("{} refused work", colonist.name),
        format!(
            "Mood {:.0} and social pressure created a {:.0}% refusal chance.",
            colonist.mood, refusal_chance
        ),
    ));
}

fn building_type_for_activity(
    activity: &ActivityType,
    job_preference: JobPreference,
) -> Option<BuildingType> {
    match activity {
        ActivityType::Sleep => Some(BuildingType::Habitat),
        ActivityType::Work => Some(job_preference.work_building_type()),
        ActivityType::Eat => Some(BuildingType::MessHall),
        ActivityType::Relax => None,
    }
}

fn specific_target_for_activity(
    colonist: &Colonist,
    activity: &ActivityType,
    building_type: BuildingType,
    buildings: &[(u32, BuildingType, Position, (u32, u32))],
) -> Option<u32> {
    match activity {
        ActivityType::Sleep => colonist.assigned_habitat.filter(|building_id| {
            building_matches_type(buildings, *building_id, BuildingType::Habitat)
        }),
        ActivityType::Work => colonist
            .assigned_workplace
            .filter(|building_id| building_matches_type(buildings, *building_id, building_type)),
        ActivityType::Eat | ActivityType::Relax => None,
    }
}

fn building_matches_type(
    buildings: &[(u32, BuildingType, Position, (u32, u32))],
    building_id: u32,
    building_type: BuildingType,
) -> bool {
    buildings
        .iter()
        .any(|(id, candidate_type, _, _)| *id == building_id && *candidate_type == building_type)
}

fn set_activity_at_building(
    colonist: &mut Colonist,
    activity: &ActivityType,
    building_id: u32,
    building_type: BuildingType,
) {
    colonist.current_activity = activity.clone();
    colonist.activity_location = ActivityLocation::Building {
        building_id,
        building_type,
    };
    colonist.state = activity_to_state(activity);
}

fn set_moving_to_activity(colonist: &mut Colonist, activity: &ActivityType, target: Position) {
    colonist.current_activity = activity.clone();
    colonist.activity_location = ActivityLocation::None;
    colonist.state = ColonistState::Moving { target };
}

fn set_idle(colonist: &mut Colonist) {
    colonist.current_activity = ActivityType::Relax;
    colonist.activity_location = ActivityLocation::None;
    colonist.state = ColonistState::Idle;
}

/// Convert activity type to colonist state
fn activity_to_state(activity: &ActivityType) -> ColonistState {
    match activity {
        ActivityType::Sleep => ColonistState::Sleeping,
        ActivityType::Work => ColonistState::Working,
        ActivityType::Eat => ColonistState::Eating,
        ActivityType::Relax => ColonistState::Idle,
    }
}

/// Find the best entrance position for a building of the given type
fn find_building_entrance(
    from: Position,
    building_type: BuildingType,
    buildings: &[(u32, BuildingType, Position, (u32, u32))],
    specific_target: Option<u32>,
    colonist: &Colonist,
    social_locations: &[SocialLocation],
    grid: &Grid,
) -> Option<BuildingTarget> {
    let mut best_target: Option<(BuildingTarget, i32, i32)> = None;

    for (id, bt, pos, (width, height)) in buildings.iter() {
        if *bt != building_type {
            continue;
        }

        if let Some(target_id) = specific_target {
            if *id != target_id {
                continue;
            }
        }

        let Some((entrance, distance)) = best_building_entrance(from, *pos, *width, *height, grid)
        else {
            continue;
        };

        let target = BuildingTarget {
            building_id: *id,
            building_type: *bt,
            entrance,
        };
        let social_score = social_score_for_building(colonist, *id, social_locations);
        let candidate = (target, social_score, distance);

        if best_target
            .as_ref()
            .map(|best| better_target_candidate(&candidate, best))
            .unwrap_or(true)
        {
            best_target = Some(candidate);
        }
    }

    best_target.map(|(target, _, _)| target)
}

fn best_building_entrance(
    from: Position,
    pos: Position,
    width: u32,
    height: u32,
    grid: &Grid,
) -> Option<(Position, i32)> {
    let mut best_target: Option<Position> = None;
    let mut best_distance = i32::MAX;

    for dx in -1..=(width as i32) {
        for dy in -1..=(height as i32) {
            let check_x = pos.x + dx;
            let check_y = pos.y + dy;

            if dx >= 0 && dx < width as i32 && dy >= 0 && dy < height as i32 {
                continue;
            }

            if !grid
                .get_cell(check_x, check_y)
                .is_some_and(|cell| cell.is_walkable())
            {
                continue;
            }

            let candidate = Position::new(check_x, check_y);
            let dist = (from.x - check_x).abs() + (from.y - check_y).abs();

            if dist < best_distance {
                best_distance = dist;
                best_target = Some(candidate);
            }
        }
    }

    best_target.map(|entrance| (entrance, best_distance))
}

fn social_score_for_building(
    colonist: &Colonist,
    building_id: u32,
    social_locations: &[SocialLocation],
) -> i32 {
    social_locations
        .iter()
        .filter(|(other_id, location)| {
            *other_id != colonist.id && location.building_id() == Some(building_id)
        })
        .map(|(other_id, _)| {
            let relationship = colonist.relationships.get(other_id).copied().unwrap_or(0);
            let directive = if colonist.preferred_partner_id == Some(*other_id) {
                80
            } else if colonist.avoided_partner_id == Some(*other_id) {
                -80
            } else {
                0
            };

            relationship + directive
        })
        .sum()
}

fn better_habitat_candidate(candidate: (u32, i32, u32), best: (u32, i32, u32)) -> bool {
    let (candidate_id, candidate_score, candidate_count) = candidate;
    let (best_id, best_score, best_count) = best;

    candidate_score > best_score
        || (candidate_score == best_score && candidate_count < best_count)
        || (candidate_score == best_score
            && candidate_count == best_count
            && candidate_id < best_id)
}

fn better_target_candidate(
    candidate: &(BuildingTarget, i32, i32),
    best: &(BuildingTarget, i32, i32),
) -> bool {
    let (candidate_target, candidate_score, candidate_distance) = candidate;
    let (best_target, best_score, best_distance) = best;

    candidate_score > best_score
        || (candidate_score == best_score && candidate_distance < best_distance)
        || (candidate_score == best_score
            && candidate_distance == best_distance
            && candidate_target.building_id < best_target.building_id)
}

fn find_adjacent_building(
    pos: Position,
    building_type: BuildingType,
    buildings: &[(u32, BuildingType, Position, (u32, u32))],
    specific_target: Option<u32>,
) -> Option<(u32, BuildingType)> {
    for (id, bt, bpos, (width, height)) in buildings {
        if *bt != building_type {
            continue;
        }

        if let Some(target_id) = specific_target {
            if *id != target_id {
                continue;
            }
        }

        if is_position_adjacent_to_building(pos, *bpos, *width, *height) {
            return Some((*id, *bt));
        }
    }

    None
}

/// Check if colonist is adjacent to any building of the given type
fn is_adjacent_to_building(
    pos: Position,
    building_type: BuildingType,
    buildings: &[(u32, BuildingType, Position, (u32, u32))],
    specific_target: Option<u32>,
) -> bool {
    find_adjacent_building(pos, building_type, buildings, specific_target).is_some()
}

fn is_position_adjacent_to_building(
    pos: Position,
    building_pos: Position,
    width: u32,
    height: u32,
) -> bool {
    let min_x = building_pos.x - 1;
    let max_x = building_pos.x + width as i32;
    let min_y = building_pos.y - 1;
    let max_y = building_pos.y + height as i32;

    let on_perimeter = pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y;
    let inside = pos.x >= building_pos.x
        && pos.x < building_pos.x + width as i32
        && pos.y >= building_pos.y
        && pos.y < building_pos.y + height as i32;

    on_perimeter && !inside
}

/// Find a random wander target that isn't occupied
fn find_wander_target(
    current: Position,
    occupied: &HashMap<Position, u32>,
    grid: &Grid,
    rng: &mut SimulationRng,
) -> Position {
    for _ in 0..10 {
        let target_x = rng.range_i32(0, grid.width as i32);
        let target_y = rng.range_i32(0, grid.height as i32);
        let target = Position::new(target_x, target_y);
        if is_step_open(target, occupied, grid) {
            return target;
        }
    }
    current
}

/// Get the next position to move to, avoiding collisions
fn get_next_move_position(
    colonist: &mut Colonist,
    target: Position,
    occupied: &HashMap<Position, u32>,
    grid: &Grid,
    colonist_names: &HashMap<u32, String>,
    current_tick: u64,
    pending_logs: &mut Vec<PendingLog>,
) -> Position {
    let current = colonist.position;
    let Some(next) = grid
        .find_path(current, target)
        .and_then(|path| path.into_iter().find(|step| *step != current))
    else {
        return current;
    };

    if next != current {
        if let Some(other_id) = occupied.get(&next) {
            let relationship = colonist.relationships.get(other_id).copied().unwrap_or(0);
            if relationship < -20 {
                colonist.mood = (colonist.mood - 5.0).clamp(0.0, 100.0);
                log_social_strain(
                    colonist,
                    *other_id,
                    colonist_names,
                    current_tick,
                    pending_logs,
                );
            }

            let horiz = Position::new(
                if current.x < target.x {
                    current.x + 1
                } else if current.x > target.x {
                    current.x - 1
                } else {
                    current.x
                },
                current.y,
            );
            if horiz != current && is_step_open(horiz, occupied, grid) {
                return horiz;
            }

            let vert = Position::new(
                current.x,
                if current.y < target.y {
                    current.y + 1
                } else if current.y > target.y {
                    current.y - 1
                } else {
                    current.y
                },
            );
            if vert != current && is_step_open(vert, occupied, grid) {
                return vert;
            }

            return current;
        }
    }

    next
}

fn is_step_open(position: Position, occupied: &HashMap<Position, u32>, grid: &Grid) -> bool {
    !occupied.contains_key(&position)
        && grid
            .get_cell(position.x, position.y)
            .is_some_and(|cell| cell.is_walkable())
}

fn log_social_strain(
    colonist: &mut Colonist,
    other_id: u32,
    colonist_names: &HashMap<u32, String>,
    current_tick: u64,
    pending_logs: &mut Vec<PendingLog>,
) {
    if current_tick.saturating_sub(colonist.last_social_strain_tick)
        < SOCIAL_STRAIN_LOG_COOLDOWN_TICKS
    {
        return;
    }

    colonist.last_social_strain_tick = current_tick;
    let other_name = colonist_names
        .get(&other_id)
        .cloned()
        .unwrap_or_else(|| format!("Colonist {}", other_id));

    pending_logs.push((
        LogCategory::Social,
        format!("{} avoided {}", colonist.name, other_name),
        "A strained relationship made a crowded path stressful. Mood dropped.".to_string(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{ColonistState, JobPreference, Trait};
    use crate::data::grid::CellType;

    #[test]
    fn test_strained_proximity_logs_visible_mood_drop() {
        let mut state = GameState::new();
        state.tick = 420;

        let mut alice = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Explorer,
        );
        alice.state = ColonistState::Moving {
            target: Position::new(1, 0),
        };
        alice.relationships.insert(2, -30);
        alice.mood = 50.0;

        let mut bob = Colonist::new(
            2,
            "Bob".to_string(),
            Position::new(1, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        bob.relationships.insert(1, -30);

        state.colonists.push(alice);
        state.colonists.push(bob);

        update_colonists(&mut state, 1);

        let alice = &state.colonists[0];
        assert_eq!(alice.position, Position::new(0, 0));
        assert!(alice.mood <= 45.1);

        let log = state
            .event_log
            .iter()
            .find(|entry| entry.category == LogCategory::Social)
            .expect("strained proximity should create a social log");
        assert_eq!(log.title, "Alice avoided Bob");
        assert!(log.detail.contains("strained relationship"));
        assert!(log.detail.contains("Mood dropped"));
    }

    #[test]
    fn test_work_target_prefers_supportive_occupied_building() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.relationships.insert(2, -35);
        colonist.relationships.insert(3, 30);

        let buildings = vec![
            (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
            (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
        ];
        let social_locations = vec![
            (
                2,
                ActivityLocation::Building {
                    building_id: 10,
                    building_type: BuildingType::Workshop,
                },
            ),
            (
                3,
                ActivityLocation::Building {
                    building_id: 20,
                    building_type: BuildingType::Workshop,
                },
            ),
        ];
        let grid = Grid::default();

        let target = find_building_entrance(
            colonist.position,
            BuildingType::Workshop,
            &buildings,
            None,
            &colonist,
            &social_locations,
            &grid,
        )
        .expect("workshop target should be found");

        assert_eq!(target.building_id, 20);
    }

    #[test]
    fn test_movement_uses_grid_pathfinding_around_blocked_cells() {
        let mut grid = Grid::new(4, 3);
        grid.set_cell_type(1, 0, CellType::Wall);

        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        let occupied = HashMap::new();
        let colonist_names = HashMap::new();
        let mut pending_logs = Vec::new();

        let next = get_next_move_position(
            &mut colonist,
            Position::new(2, 0),
            &occupied,
            &grid,
            &colonist_names,
            0,
            &mut pending_logs,
        );

        assert_ne!(next, Position::new(1, 0));
        assert_ne!(next, colonist.position);
        assert!(is_step_open(next, &occupied, &grid));
    }

    #[test]
    fn test_work_target_honors_keep_apart_directive() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.relationships.insert(2, 30);
        colonist.avoided_partner_id = Some(2);

        let buildings = vec![
            (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
            (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
        ];
        let social_locations = vec![(
            2,
            ActivityLocation::Building {
                building_id: 10,
                building_type: BuildingType::Workshop,
            },
        )];
        let grid = Grid::default();

        let target = find_building_entrance(
            colonist.position,
            BuildingType::Workshop,
            &buildings,
            None,
            &colonist,
            &social_locations,
            &grid,
        )
        .expect("workshop target should be found");

        assert_eq!(target.building_id, 20);
    }

    #[test]
    fn test_work_target_honors_manual_workplace_assignment() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.assigned_workplace = Some(20);

        let buildings = vec![
            (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
            (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
        ];
        let grid = Grid::default();
        let target = find_building_entrance(
            colonist.position,
            BuildingType::Workshop,
            &buildings,
            specific_target_for_activity(
                &colonist,
                &ActivityType::Work,
                BuildingType::Workshop,
                &buildings,
            ),
            &colonist,
            &[],
            &grid,
        )
        .expect("assigned workshop target should be found");

        assert_eq!(target.building_id, 20);
    }

    #[test]
    fn test_workplace_assignment_ignores_wrong_building_type() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.assigned_workplace = Some(20);

        let buildings = vec![(20, BuildingType::MessHall, Position::new(12, 12), (3, 2))];

        assert_eq!(
            specific_target_for_activity(
                &colonist,
                &ActivityType::Work,
                BuildingType::Workshop,
                &buildings,
            ),
            None
        );
    }

    #[test]
    fn test_habitat_assignment_prefers_supportive_roommate() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.relationships.insert(2, -26);
        colonist.relationships.insert(3, 22);

        let buildings = vec![
            (10, BuildingType::Habitat, Position::new(1, 1), (2, 2)),
            (20, BuildingType::Habitat, Position::new(10, 10), (2, 2)),
        ];
        let social_locations = vec![
            (
                2,
                ActivityLocation::Building {
                    building_id: 10,
                    building_type: BuildingType::Habitat,
                },
            ),
            (
                3,
                ActivityLocation::Building {
                    building_id: 20,
                    building_type: BuildingType::Habitat,
                },
            ),
        ];
        let mut building_occupancy = HashMap::new();
        let mut pending_logs = Vec::new();

        let target = find_or_assign_habitat(
            &mut colonist,
            &buildings,
            &mut building_occupancy,
            2,
            &social_locations,
            &mut pending_logs,
            420,
        );

        assert_eq!(target, Some(BuildingType::Habitat));
        assert_eq!(colonist.assigned_habitat, Some(20));
    }
}
