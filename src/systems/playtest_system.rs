use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, ColonistState, JobPreference};
use crate::data::game_state::GameState;
use crate::data::priority::ColonyPriority;
use crate::data::resources::ColonyCondition;
use crate::data::scenario::ScenarioOutcome;
use crate::data::schedule::ActivityType;
use crate::data::types::Position;
use crate::game::building_system::PlacementResult;
use crate::systems::incident_system::IncidentSystem;
use crate::systems::mission_system::MissionSystem;
use crate::systems::mood_system::update_mood;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::scenario_system::ScenarioSystem;
use crate::systems::social_system::SocialSystem;
use crate::systems::summary_system::SummarySystem;
use crate::systems::time_system::TimeSystem;
use crate::systems::work_system::WorkSystem;

pub const REFERENCE_START_TICK: u64 = 420;
pub const NORMAL_SECONDS_PER_TICK: f32 = 0.25;

#[derive(Clone, Debug)]
pub struct PlaytestReport {
    pub start_tick: u64,
    pub end_tick: u64,
    pub estimated_normal_minutes: f32,
    pub outcome: ScenarioOutcome,
    pub condition: ColonyCondition,
    pub supplies: i32,
    pub salvage: i32,
    pub daily_supply_need: i32,
    pub technologies_unlocked: usize,
    pub required_technologies: usize,
    pub missions_completed: u32,
    pub buildings_placed: usize,
    pub incidents_triggered: usize,
}

impl PlaytestReport {
    pub fn proves_reference_run(&self) -> bool {
        self.outcome == ScenarioOutcome::Victory
            && (30.0..=40.0).contains(&self.estimated_normal_minutes)
            && self.condition != ColonyCondition::Critical
            && self.condition != ColonyCondition::Collapsed
            && self.supplies >= self.daily_supply_need.max(1)
            && self.technologies_unlocked >= self.required_technologies
            && self.missions_completed >= self.required_technologies as u32
            && self.buildings_placed >= 5
            && self.incidents_triggered >= 1
    }
}

#[derive(Clone, Debug, Default)]
struct ReferenceStrategy {
    missions_completed: u32,
    next_mission_tick: u64,
}

pub struct PlaytestSystem;

impl PlaytestSystem {
    pub fn run_reference_playthrough() -> PlaytestReport {
        let mut state = GameState::new();
        state.tick = REFERENCE_START_TICK;
        crate::game::colonist_spawner::spawn_initial_colonists(&mut state);

        let mut strategy = ReferenceStrategy::default();
        let target_tick =
            state.scenario.target_day.saturating_sub(1) as u64 * TimeSystem::TICKS_PER_DAY;

        Self::manage_build_plan(&mut state);

        while state.tick < target_tick && !state.scenario.is_finished() {
            state.tick += 1;

            let before_active_missions = state.missions.active_count();
            MissionSystem::process_completed_missions(&mut state);
            let after_active_missions = state.missions.active_count();
            strategy.missions_completed +=
                before_active_missions.saturating_sub(after_active_missions) as u32;
            MissionSystem::recover_injured_colonists(&mut state);

            Self::update_priority(&mut state);
            Self::maybe_launch_mission(&mut state, &mut strategy);

            if state.tick % TimeSystem::TICKS_PER_DAY == 0 {
                let (day, _, _) = TimeSystem::get_time_of_day(state.tick);
                SummarySystem::summarize_previous_day(&mut state, day);
                ResourceSystem::handle_new_day(&mut state);
                Self::manage_build_plan(&mut state);
            }

            if state.tick % TimeSystem::TICKS_PER_HOUR == 0 {
                IncidentSystem::process_hourly_incidents(&mut state);
                Self::process_hour(&mut state);
                Self::manage_build_plan(&mut state);
            }

            ScenarioSystem::evaluate(&mut state);
        }

        if !state.scenario.is_finished() {
            ScenarioSystem::evaluate(&mut state);
        }

        let estimated_normal_minutes =
            (state.tick - REFERENCE_START_TICK) as f32 * NORMAL_SECONDS_PER_TICK / 60.0;

        PlaytestReport {
            start_tick: REFERENCE_START_TICK,
            end_tick: state.tick,
            estimated_normal_minutes,
            outcome: state.scenario.outcome,
            condition: state.resources.condition,
            supplies: state.resources.supplies,
            salvage: state.resources.salvage,
            daily_supply_need: ResourceSystem::daily_supply_need(&state),
            technologies_unlocked: state.technology.unlocked_count(),
            required_technologies: state.scenario.required_tech_unlocks,
            missions_completed: strategy.missions_completed,
            buildings_placed: state.building_system.building_count(),
            incidents_triggered: state.incidents.triggered.len(),
        }
    }

    fn update_priority(state: &mut GameState) {
        let desired = if state.technology.unlocked_count() < state.scenario.required_tech_unlocks {
            ColonyPriority::Survey
        } else if state.resources.supplies < ResourceSystem::daily_supply_need(state).max(1) * 3 {
            ColonyPriority::Stockpile
        } else {
            ColonyPriority::Recovery
        };

        state.priority.active = desired;
    }

    fn maybe_launch_mission(state: &mut GameState, strategy: &mut ReferenceStrategy) {
        if state.tick < strategy.next_mission_tick
            || state.technology.unlocked_count() >= state.scenario.required_tech_unlocks
            || state.missions.active_count() > 0
        {
            return;
        }

        let mission_type = MissionSystem::recommended_mission_type(state);
        if MissionSystem::launch_mission(state, mission_type).is_ok() {
            strategy.next_mission_tick = state.tick + mission_type.definition().cooldown_minutes;
        } else {
            strategy.next_mission_tick = state.tick + 60;
        }
    }

    fn process_hour(state: &mut GameState) {
        let (_, hour, _) = TimeSystem::get_time_of_day(state.tick);
        Self::assign_colonists_for_hour(state, hour);

        for colonist in &mut state.colonists {
            update_mood(colonist, TimeSystem::TICKS_PER_HOUR, state.priority.active);
        }

        WorkSystem::process_hourly_work(state);
        SocialSystem::check_working_together(state);
        SocialSystem::check_eating_together(state);
    }

    fn assign_colonists_for_hour(state: &mut GameState, hour: u32) {
        let activity = activity_for_hour(hour);
        let mess_hall = Self::first_building_id(state, BuildingType::MessHall);
        let habitat = Self::first_building_id(state, BuildingType::Habitat);
        let workshop = Self::first_building_id(state, BuildingType::Workshop);
        let storage = Self::first_building_id(state, BuildingType::Storage);
        let exploration_gate = Self::first_building_id(state, BuildingType::ExplorationGate);

        for colonist in &mut state.colonists {
            if colonist.is_on_mission() {
                continue;
            }

            colonist.current_activity = activity.clone();
            match activity {
                ActivityType::Work => {
                    let target = match colonist.job_preference {
                        JobPreference::Explorer => {
                            exploration_gate.map(|id| (id, BuildingType::ExplorationGate))
                        }
                        JobPreference::Builder => workshop.map(|id| (id, BuildingType::Workshop)),
                        JobPreference::Cook => mess_hall.map(|id| (id, BuildingType::MessHall)),
                        JobPreference::Hauler => storage.map(|id| (id, BuildingType::Storage)),
                        JobPreference::None => workshop.map(|id| (id, BuildingType::Workshop)),
                    };

                    if let Some((building_id, building_type)) = target {
                        colonist.state = ColonistState::Working;
                        colonist.activity_location = ActivityLocation::Building {
                            building_id,
                            building_type,
                        };
                    } else {
                        colonist.state = ColonistState::Idle;
                        colonist.activity_location = ActivityLocation::None;
                    }
                }
                ActivityType::Eat => {
                    colonist.state = ColonistState::Eating;
                    colonist.activity_location = mess_hall
                        .map(|building_id| ActivityLocation::Building {
                            building_id,
                            building_type: BuildingType::MessHall,
                        })
                        .unwrap_or(ActivityLocation::None);
                }
                ActivityType::Sleep => {
                    colonist.state = ColonistState::Sleeping;
                    colonist.activity_location = habitat
                        .map(|building_id| ActivityLocation::Building {
                            building_id,
                            building_type: BuildingType::Habitat,
                        })
                        .unwrap_or(ActivityLocation::Ground(colonist.position));
                }
                ActivityType::Relax => {
                    colonist.state = ColonistState::Idle;
                    colonist.activity_location = ActivityLocation::None;
                }
            }
        }
    }

    fn manage_build_plan(state: &mut GameState) {
        Self::ensure_building_count(
            state,
            BuildingType::Habitat,
            &[
                Position::new(0, 0),
                Position::new(0, 4),
                Position::new(3, 4),
            ],
        );
        Self::ensure_building_count(state, BuildingType::MessHall, &[Position::new(3, 0)]);
        Self::ensure_building_count(state, BuildingType::Workshop, &[Position::new(7, 0)]);
        Self::ensure_building_count(state, BuildingType::Storage, &[Position::new(10, 0)]);
        Self::ensure_building_count(
            state,
            BuildingType::ExplorationGate,
            &[Position::new(13, 0)],
        );
    }

    fn ensure_building_count(
        state: &mut GameState,
        building_type: BuildingType,
        positions: &[Position],
    ) {
        let current = state
            .building_system
            .buildings()
            .iter()
            .filter(|building| building.building_type == building_type)
            .count();

        for position in positions.iter().skip(current) {
            if !ResourceSystem::can_afford_building(state, building_type) {
                return;
            }

            match state.building_system.try_place_building(
                &mut state.grid,
                building_type,
                *position,
            ) {
                PlacementResult::Success(_) => {
                    state.resources.spend_salvage(building_type.salvage_cost());
                }
                PlacementResult::OutOfBounds
                | PlacementResult::AreaOccupied
                | PlacementResult::InvalidBuilding => return,
            }
        }
    }

    fn first_building_id(state: &GameState, building_type: BuildingType) -> Option<u32> {
        state
            .building_system
            .buildings()
            .iter()
            .find(|building| building.building_type == building_type)
            .map(|building| building.id)
    }
}

fn activity_for_hour(hour: u32) -> ActivityType {
    match hour {
        6 | 20 | 21 => ActivityType::Eat,
        7..=17 => ActivityType::Work,
        22..=23 | 0..=5 => ActivityType::Sleep,
        _ => ActivityType::Relax,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_playthrough_reaches_day_7_victory_window() {
        let report = PlaytestSystem::run_reference_playthrough();

        assert_eq!(report.start_tick, REFERENCE_START_TICK);
        assert_eq!(report.end_tick, TimeSystem::TICKS_PER_DAY * 6);
        assert!(
            report.proves_reference_run(),
            "reference playthrough did not prove a full run: {:?}",
            report
        );
        assert!(report.technologies_unlocked >= report.required_technologies);
    }
}
