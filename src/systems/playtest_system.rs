use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, ColonistState, JobPreference};
use crate::data::game_state::GameState;
use crate::data::mission::MissionType;
use crate::data::priority::ColonyPriority;
use crate::data::schedule::ActivityType;
use crate::data::types::Position;
use crate::game::building_system::PlacementResult;
use crate::systems::incident_system::IncidentSystem;
use crate::systems::mission_system::MissionSystem;
use crate::systems::mood_system::update_mood;
use crate::systems::playtest_strategy::{activity_for_hour, ReferenceStrategy};
use crate::systems::resource_system::ResourceSystem;
use crate::systems::scenario_system::ScenarioSystem;
use crate::systems::social_system::SocialSystem;
use crate::systems::summary_system::SummarySystem;
use crate::systems::time_system::TimeSystem;
use crate::systems::work_system::WorkSystem;

pub const REFERENCE_START_TICK: u64 = 420;
pub const NORMAL_SECONDS_PER_TICK: f32 = 0.25;

pub use crate::systems::playtest_report::PlaytestReport;
pub use crate::systems::playtest_strategy::PlaytestStrategyKind;

pub struct PlaytestSystem;

impl PlaytestSystem {
    pub fn run_reference_playthrough() -> PlaytestReport {
        Self::run_strategy_playthrough(PlaytestStrategyKind::Reference)
    }

    pub fn run_strategy_playthrough(kind: PlaytestStrategyKind) -> PlaytestReport {
        let mut state = GameState::new();
        state.tick = REFERENCE_START_TICK;
        crate::game::colonist_spawner::spawn_initial_colonists(&mut state);

        let mut strategy = ReferenceStrategy::new(kind);
        let target_tick =
            state.scenario.target_day.saturating_sub(1) as u64 * TimeSystem::TICKS_PER_DAY;

        Self::manage_build_plan(&mut state, kind);

        while state.tick < target_tick && !state.scenario.is_finished() {
            state.tick += 1;

            let before_active_missions = state.missions.active_count();
            MissionSystem::process_completed_missions(&mut state);
            let after_active_missions = state.missions.active_count();
            strategy.missions_completed +=
                before_active_missions.saturating_sub(after_active_missions) as u32;
            MissionSystem::recover_injured_colonists(&mut state);

            Self::update_priority(&mut state, kind);
            Self::maybe_launch_mission(&mut state, &mut strategy);

            if state.tick % TimeSystem::TICKS_PER_DAY == 0 {
                let (day, _, _) = TimeSystem::get_time_of_day(state.tick);
                SummarySystem::summarize_previous_day(&mut state, day);
                ResourceSystem::handle_new_day(&mut state);
                Self::manage_build_plan(&mut state, kind);
            }

            if state.tick % TimeSystem::TICKS_PER_HOUR == 0 {
                IncidentSystem::process_hourly_incidents(&mut state);
                Self::process_hour(&mut state);
                Self::manage_build_plan(&mut state, kind);
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
            strategy: kind,
            end_tick: state.tick,
            estimated_normal_minutes,
            outcome: state.scenario.outcome,
            condition: state.resources.condition,
            average_mood: Self::average_mood(&state),
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

    pub fn capture_report_set() -> Vec<PlaytestReport> {
        PlaytestStrategyKind::report_set()
            .iter()
            .map(|kind| {
                if *kind == PlaytestStrategyKind::Reference {
                    Self::run_reference_playthrough()
                } else {
                    Self::run_strategy_playthrough(*kind)
                }
            })
            .collect()
    }

    pub fn playthrough_report_markdown(reports: &[PlaytestReport]) -> String {
        let mut output = String::from("# The Final Landing Playthrough Capture\n\n");
        output.push_str(
            "Headless strategy runs from the live simulation. Minutes are estimated at normal speed.\n\n",
        );
        output.push_str("| Strategy | Band | Outcome | Condition | Start | Minutes | Mood | Supplies | Salvage | Tech | Missions | Buildings | Incidents | Proves |\n");
        output.push_str(
            "| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |\n",
        );

        for report in reports {
            output.push_str(&format!(
                "| {} | {} | {:?} | {:?} | {} | {:.1} | {:.0} | {} | {} | {}/{} | {} | {} | {} | {} |\n",
                report.strategy.label(),
                report.outcome_band().label(),
                report.outcome,
                report.condition,
                report.start_tick,
                report.estimated_normal_minutes,
                report.average_mood,
                report.supplies,
                report.salvage,
                report.technologies_unlocked,
                report.required_technologies,
                report.missions_completed,
                report.buildings_placed,
                report.incidents_triggered,
                if report.proves_reference_run() { "yes" } else { "no" }
            ));
        }

        output
    }

    fn update_priority(state: &mut GameState, kind: PlaytestStrategyKind) {
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        let desired = match kind {
            PlaytestStrategyKind::Reference => {
                if state.technology.unlocked_count() < state.scenario.required_tech_unlocks {
                    ColonyPriority::Survey
                } else if state.resources.supplies < daily_need * 3 {
                    ColonyPriority::Stockpile
                } else {
                    ColonyPriority::Recovery
                }
            }
            PlaytestStrategyKind::Conservative => {
                if state.resources.supplies < daily_need * 4 {
                    ColonyPriority::Stockpile
                } else if Self::average_mood(state) < 55.0 {
                    ColonyPriority::Recovery
                } else {
                    ColonyPriority::Survey
                }
            }
            PlaytestStrategyKind::SurveyHeavy => {
                if state.technology.unlocked_count() < state.scenario.required_tech_unlocks + 1 {
                    ColonyPriority::Survey
                } else {
                    ColonyPriority::Stockpile
                }
            }
            PlaytestStrategyKind::RecoveryHeavy => {
                if state.resources.supplies < daily_need * 2 {
                    ColonyPriority::Stockpile
                } else {
                    ColonyPriority::Recovery
                }
            }
            PlaytestStrategyKind::NoFood => ColonyPriority::Recovery,
            PlaytestStrategyKind::NoMissions => ColonyPriority::Survey,
            PlaytestStrategyKind::NoHabitats => {
                if state.technology.unlocked_count() < state.scenario.required_tech_unlocks {
                    ColonyPriority::Survey
                } else {
                    ColonyPriority::Stockpile
                }
            }
        };

        state.priority.active = desired;
    }

    fn maybe_launch_mission(state: &mut GameState, strategy: &mut ReferenceStrategy) {
        if strategy.kind == PlaytestStrategyKind::NoMissions {
            return;
        }

        if state.tick < strategy.next_mission_tick
            || state.technology.unlocked_count() >= state.scenario.required_tech_unlocks
            || state.missions.active_count() > 0
        {
            return;
        }

        let mission_type = Self::mission_for_strategy(state, strategy.kind);
        if MissionSystem::launch_mission(state, mission_type).is_ok() {
            strategy.next_mission_tick = state.tick + mission_type.definition().cooldown_minutes;
        } else {
            strategy.next_mission_tick = state.tick + 60;
        }
    }

    fn mission_for_strategy(state: &GameState, kind: PlaytestStrategyKind) -> MissionType {
        match kind {
            PlaytestStrategyKind::Reference => MissionSystem::recommended_mission_type(state),
            PlaytestStrategyKind::Conservative => {
                let daily_need = ResourceSystem::daily_supply_need(state).max(1);
                if state.resources.supplies < daily_need * 3 {
                    MissionType::SupplyRun
                } else {
                    MissionType::PerimeterScan
                }
            }
            PlaytestStrategyKind::SurveyHeavy => MissionType::DeepSurvey,
            PlaytestStrategyKind::RecoveryHeavy => {
                let daily_need = ResourceSystem::daily_supply_need(state).max(1);
                if state.resources.supplies < daily_need * 2 {
                    MissionType::SupplyRun
                } else {
                    MissionType::PerimeterScan
                }
            }
            PlaytestStrategyKind::NoFood => MissionType::DeepSurvey,
            PlaytestStrategyKind::NoHabitats => MissionSystem::recommended_mission_type(state),
            PlaytestStrategyKind::NoMissions => MissionType::PerimeterScan,
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

    fn manage_build_plan(state: &mut GameState, kind: PlaytestStrategyKind) {
        if kind != PlaytestStrategyKind::NoHabitats {
            Self::ensure_building_count(
                state,
                BuildingType::Habitat,
                &[
                    Position::new(0, 0),
                    Position::new(0, 4),
                    Position::new(3, 4),
                ],
            );
        }

        if kind != PlaytestStrategyKind::NoFood {
            Self::ensure_building_count(state, BuildingType::MessHall, &[Position::new(3, 0)]);
        }

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
                PlacementResult::OutOfBounds | PlacementResult::AreaOccupied => return,
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

    fn average_mood(state: &GameState) -> f32 {
        if state.colonists.is_empty() {
            return 0.0;
        }

        state
            .colonists
            .iter()
            .map(|colonist| colonist.mood)
            .sum::<f32>()
            / state.colonists.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::scenario::ScenarioOutcome;
    use crate::systems::playtest_report::PlaytestOutcomeBand;

    #[test]
    fn test_reference_playthrough_reaches_day_7_victory_window() {
        let report = PlaytestSystem::run_reference_playthrough();

        assert_eq!(report.start_tick, REFERENCE_START_TICK);
        assert_eq!(report.strategy, PlaytestStrategyKind::Reference);
        assert_eq!(report.end_tick, TimeSystem::TICKS_PER_DAY * 6);
        assert!(
            report.proves_reference_run(),
            "reference playthrough did not prove a full run: {:?}",
            report
        );
        assert!(report.technologies_unlocked >= report.required_technologies);
    }

    #[test]
    fn test_strategy_variants_track_win_limp_or_fail() {
        let conservative =
            PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::Conservative);
        let survey = PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::SurveyHeavy);
        let recovery =
            PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::RecoveryHeavy);

        assert_ne!(conservative.outcome_band(), PlaytestOutcomeBand::Fail);
        assert_ne!(survey.outcome_band(), PlaytestOutcomeBand::Fail);
        assert_ne!(recovery.outcome_band(), PlaytestOutcomeBand::Fail);
        assert_eq!(conservative.outcome, ScenarioOutcome::Victory);
        assert_eq!(survey.outcome, ScenarioOutcome::Victory);
        assert_eq!(recovery.outcome, ScenarioOutcome::Victory);
    }

    #[test]
    fn test_poor_planning_variants_can_fail_day_7() {
        let no_food = PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::NoFood);
        assert_eq!(no_food.outcome_band(), PlaytestOutcomeBand::Limp);
        assert_ne!(no_food.outcome, ScenarioOutcome::Victory);

        let no_habitats =
            PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::NoHabitats);
        assert_eq!(no_habitats.outcome_band(), PlaytestOutcomeBand::Limp);
        assert_ne!(no_habitats.outcome, ScenarioOutcome::Victory);

        for kind in [PlaytestStrategyKind::NoMissions] {
            let report = PlaytestSystem::run_strategy_playthrough(kind);
            assert_eq!(
                report.outcome_band(),
                PlaytestOutcomeBand::Fail,
                "{kind:?} should fail: {report:?}"
            );
        }
    }

    #[test]
    fn test_playthrough_report_markdown_includes_strategy_matrix() {
        let reports = vec![
            PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::Reference),
            PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::NoMissions),
        ];

        let markdown = PlaytestSystem::playthrough_report_markdown(&reports);

        assert!(markdown.contains("# The Final Landing Playthrough Capture"));
        assert!(markdown.contains("| Reference | Win | Victory |"));
        assert!(markdown.contains("| No missions | Fail |"));
        assert!(markdown.contains("Tech"));
        assert!(markdown.contains("Incidents"));
    }
}
