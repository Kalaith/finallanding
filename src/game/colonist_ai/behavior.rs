use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, Colonist, ColonistState};
use crate::data::event_log::LogCategory;
use crate::data::grid::Grid;
use crate::data::schedule::ActivityType;
use crate::data::simulation_rng::SimulationRng;
use crate::data::types::Position;
use crate::game::colonist_ai::assignment::{
    building_type_for_activity, find_or_assign_habitat, specific_target_for_activity,
};
use crate::game::colonist_ai::movement::{find_wander_target, get_next_move_position};
use crate::game::colonist_ai::targeting::{
    find_adjacent_building, find_building_entrance, is_adjacent_to_building,
};
use crate::game::colonist_ai::types::{BuildingSnapshot, PendingLog, SocialLocation};
use crate::game::colonist_ai::REFUSAL_LOG_COOLDOWN_TICKS;
use crate::systems::job_decision_system::calculate_refusal_chance;
use std::collections::HashMap;

pub(super) fn update_colonist_ai(
    colonist: &mut Colonist,
    scheduled_activity: &ActivityType,
    occupied: &HashMap<Position, u32>,
    colonist_names: &HashMap<u32, String>,
    social_locations: &[SocialLocation],
    grid: &Grid,
    rng: &mut SimulationRng,
    buildings: &[BuildingSnapshot],
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

fn activity_to_state(activity: &ActivityType) -> ColonistState {
    match activity {
        ActivityType::Sleep => ColonistState::Sleeping,
        ActivityType::Work => ColonistState::Working,
        ActivityType::Eat => ColonistState::Eating,
        ActivityType::Relax => ColonistState::Idle,
    }
}
