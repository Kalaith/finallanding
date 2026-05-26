use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, Colonist, ColonistState, JobPreference};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::schedule::ActivityType;
use crate::data::types::Position;
use crate::systems::job_decision_system::calculate_refusal_chance;
use crate::systems::mood_system::update_mood;
use crate::systems::time_system::TimeSystem;
use macroquad::rand::gen_range;
use std::collections::HashMap;

/// Movement speed for visual interpolation (pixels per frame)
const VISUAL_MOVE_SPEED: f32 = 2.0;
const REFUSAL_LOG_COOLDOWN_TICKS: u64 = 60;

type PendingLog = (LogCategory, String, String);

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

    let occupied: HashMap<Position, u32> =
        state.colonists.iter().map(|c| (c.position, c.id)).collect();

    let mut building_occupancy: HashMap<u32, u32> = HashMap::new();
    for c in &state.colonists {
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

    let mut pending_logs: Vec<PendingLog> = Vec::new();

    for i in 0..state.colonists.len() {
        let scheduled_activity = state.colonists[i].schedule.get_activity_for_hour(hour);

        update_colonist_ai(
            &mut state.colonists[i],
            &scheduled_activity,
            &occupied,
            &buildings,
            &mut building_occupancy,
            state.tick,
            &mut pending_logs,
        );

        state.colonists[i].update_visual_position(VISUAL_MOVE_SPEED);
        update_mood(&mut state.colonists[i], elapsed_ticks);
    }

    for (category, title, detail) in pending_logs {
        state.push_log(category, title, detail);
    }
}

fn update_colonist_ai(
    colonist: &mut Colonist,
    scheduled_activity: &ActivityType,
    occupied: &HashMap<Position, u32>,
    buildings: &[(u32, BuildingType, Position, (u32, u32))],
    building_occupancy: &mut HashMap<u32, u32>,
    current_tick: u64,
    pending_logs: &mut Vec<PendingLog>,
) {
    match colonist.state {
        ColonistState::Idle => {
            colonist.current_activity = scheduled_activity.clone();

            let target_building_type = match scheduled_activity {
                ActivityType::Sleep => find_or_assign_habitat(
                    colonist,
                    buildings,
                    building_occupancy,
                    pending_logs,
                    current_tick,
                ),
                ActivityType::Work => {
                    let refusal_chance =
                        calculate_refusal_chance(colonist, colonist.job_preference);
                    if gen_range(0.0, 100.0) < refusal_chance {
                        log_work_refusal(colonist, refusal_chance, current_tick, pending_logs);
                        None
                    } else {
                        Some(work_building_for_job(colonist.job_preference))
                    }
                }
                ActivityType::Eat => Some(BuildingType::MessHall),
                ActivityType::Relax => None,
            };

            if let Some(building_type) = target_building_type {
                let specific_target = if building_type == BuildingType::Habitat {
                    colonist.assigned_habitat
                } else {
                    None
                };

                if let Some(target) = find_building_entrance(
                    colonist.position,
                    building_type,
                    buildings,
                    specific_target,
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
                } else if gen_range(0, 100) < 5 {
                    let target = find_wander_target(colonist.position, occupied);
                    set_moving_to_activity(colonist, &ActivityType::Relax, target);
                }
            } else if matches!(scheduled_activity, ActivityType::Relax) && gen_range(0, 100) < 5 {
                let target = find_wander_target(colonist.position, occupied);
                set_moving_to_activity(colonist, scheduled_activity, target);
            }
        }
        ColonistState::Moving { target } => {
            let next_pos = get_next_move_position(colonist, target, occupied);
            colonist.position = next_pos;

            if colonist.position == target {
                let specific_target = if matches!(scheduled_activity, ActivityType::Sleep) {
                    colonist.assigned_habitat
                } else {
                    None
                };

                if let Some(building_type) =
                    building_type_for_activity(scheduled_activity, colonist.job_preference)
                {
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
    pending_logs: &mut Vec<PendingLog>,
    current_tick: u64,
) -> Option<BuildingType> {
    if let Some(bid) = colonist.assigned_habitat {
        if buildings.iter().any(|(id, _, _, _)| *id == bid) {
            return Some(BuildingType::Habitat);
        }

        colonist.assigned_habitat = None;
    }

    for (id, b_type, _, _) in buildings {
        if *b_type == BuildingType::Habitat {
            let count = building_occupancy.get(id).unwrap_or(&0);
            if *count < 2 {
                colonist.assigned_habitat = Some(*id);
                *building_occupancy.entry(*id).or_default() += 1;
                return Some(BuildingType::Habitat);
            }
        }
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

fn work_building_for_job(job_preference: JobPreference) -> BuildingType {
    match job_preference {
        JobPreference::Explorer => BuildingType::ExplorationGate,
        JobPreference::Builder => BuildingType::Workshop,
        JobPreference::Cook => BuildingType::MessHall,
        JobPreference::Hauler => BuildingType::Storage,
        JobPreference::None => BuildingType::Workshop,
    }
}

fn building_type_for_activity(
    activity: &ActivityType,
    job_preference: JobPreference,
) -> Option<BuildingType> {
    match activity {
        ActivityType::Sleep => Some(BuildingType::Habitat),
        ActivityType::Work => Some(work_building_for_job(job_preference)),
        ActivityType::Eat => Some(BuildingType::MessHall),
        ActivityType::Relax => None,
    }
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
) -> Option<BuildingTarget> {
    let mut candidates: Vec<usize> = Vec::new();

    for (i, (id, bt, _, _)) in buildings.iter().enumerate() {
        if *bt == building_type {
            if let Some(target_id) = specific_target {
                if *id == target_id {
                    candidates.push(i);
                }
            } else {
                candidates.push(i);
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    let idx = if candidates.len() > 1 {
        gen_range(0, candidates.len())
    } else {
        0
    };

    let (building_id, building_type, pos, (width, height)) = buildings[candidates[idx]];
    let mut best_target: Option<Position> = None;
    let mut best_distance = i32::MAX;

    for dx in -1..=(width as i32) {
        for dy in -1..=(height as i32) {
            let check_x = pos.x + dx;
            let check_y = pos.y + dy;

            if dx >= 0 && dx < width as i32 && dy >= 0 && dy < height as i32 {
                continue;
            }

            if check_x < 0 || check_y < 0 || check_x >= 20 || check_y >= 20 {
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

    best_target.map(|entrance| BuildingTarget {
        building_id,
        building_type,
        entrance,
    })
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
fn find_wander_target(current: Position, occupied: &HashMap<Position, u32>) -> Position {
    for _ in 0..10 {
        let target_x = gen_range(0, 20);
        let target_y = gen_range(0, 20);
        let target = Position::new(target_x, target_y);
        if !occupied.contains_key(&target) {
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
) -> Position {
    let mut next = colonist.position;
    let current = colonist.position;

    if current.x < target.x {
        next.x += 1;
    } else if current.x > target.x {
        next.x -= 1;
    }

    if current.y < target.y {
        next.y += 1;
    } else if current.y > target.y {
        next.y -= 1;
    }

    if next != current {
        if let Some(other_id) = occupied.get(&next) {
            if let Some(rel) = colonist.relationships.get(other_id) {
                if *rel < -20 {
                    colonist.mood -= 5.0;
                }
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
            if horiz != current && !occupied.contains_key(&horiz) {
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
            if vert != current && !occupied.contains_key(&vert) {
                return vert;
            }

            return current;
        }
    }

    next
}
