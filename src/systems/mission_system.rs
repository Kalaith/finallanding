use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, ColonistState, JobPreference};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::mission::{ActiveMission, MissionDefinition, MissionItem, MissionType};
use crate::data::priority::ColonyPriority;
use crate::systems::resource_system::ResourceSystem;

pub struct MissionSystem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchMissionError {
    NoExplorationGate,
    NoAvailableColonist,
    MissionCooldown { remaining_ticks: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionPlan {
    pub mission_type: MissionType,
    pub definition: MissionDefinition,
    pub danger_percent: u32,
    pub recommended: bool,
    pub recommendation_reason: String,
    pub cooldown_remaining: u64,
}

impl MissionSystem {
    pub fn mission_plans(state: &GameState) -> Vec<MissionPlan> {
        let recommended_type = Self::recommended_mission_type(state);
        let recommendation_reason = Self::recommendation_reason(state, recommended_type);
        let cooldown_remaining = state.missions.cooldown_remaining(state.tick);

        MissionType::all()
            .iter()
            .map(|mission_type| MissionPlan {
                mission_type: *mission_type,
                definition: mission_type.definition(),
                danger_percent: Self::mission_danger_percent(state, *mission_type),
                recommended: *mission_type == recommended_type,
                recommendation_reason: recommendation_reason.clone(),
                cooldown_remaining,
            })
            .collect()
    }

    pub fn recommended_mission_type(state: &GameState) -> MissionType {
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        if state.resources.supplies < daily_need * 2 {
            return MissionType::SupplyRun;
        }

        if state.priority.active == ColonyPriority::Recovery {
            return MissionType::PerimeterScan;
        }

        if state.priority.active == ColonyPriority::Survey
            || state.technology.unlocked_count() < state.scenario.required_tech_unlocks
        {
            return MissionType::DeepSurvey;
        }

        MissionType::PerimeterScan
    }

    pub fn recommendation_reason(state: &GameState, mission_type: MissionType) -> String {
        match mission_type {
            MissionType::SupplyRun => {
                let daily_need = ResourceSystem::daily_supply_need(state).max(1);
                format!(
                    "Supplies {} are under a {}-supply safety buffer.",
                    state.resources.supplies,
                    daily_need * 2
                )
            }
            MissionType::PerimeterScan => format!(
                "{} priority favors a safer balanced scout.",
                state.priority.active.label()
            ),
            MissionType::DeepSurvey => format!(
                "{} priority pushes tech progress {}/{}.",
                state.priority.active.label(),
                state.technology.unlocked_count(),
                state.scenario.required_tech_unlocks
            ),
        }
    }

    pub fn mission_danger_percent(state: &GameState, mission_type: MissionType) -> u32 {
        let definition = mission_type.definition();
        let technology_adjusted = definition
            .danger_percent
            .saturating_sub(state.technology.mission_danger_reduction());
        state
            .priority
            .active
            .adjust_mission_danger(technology_adjusted)
    }

    pub fn perimeter_scan_danger_percent(state: &GameState) -> u32 {
        Self::mission_danger_percent(state, MissionType::PerimeterScan)
    }

    pub fn launch_perimeter_scan(state: &mut GameState) -> Result<u32, LaunchMissionError> {
        Self::launch_mission(state, MissionType::PerimeterScan)
    }

    pub fn launch_mission(
        state: &mut GameState,
        mission_type: MissionType,
    ) -> Result<u32, LaunchMissionError> {
        if !state
            .building_system
            .buildings()
            .iter()
            .any(|building| building.building_type == BuildingType::ExplorationGate)
        {
            return Err(LaunchMissionError::NoExplorationGate);
        }

        let cooldown_remaining = state.missions.cooldown_remaining(state.tick);
        if cooldown_remaining > 0 {
            return Err(LaunchMissionError::MissionCooldown {
                remaining_ticks: cooldown_remaining,
            });
        }

        let Some(colonist_index) = Self::find_available_mission_colonist(state) else {
            return Err(LaunchMissionError::NoAvailableColonist);
        };

        let definition = mission_type.definition();
        let mission_id = state.missions.next_id;
        state.missions.next_id += 1;
        let danger_percent = Self::mission_danger_percent(state, mission_type);
        let completes_at_tick = state.tick + definition.duration_minutes;
        let priority = state.priority.active;

        let colonist_name = state.colonists[colonist_index].name.clone();
        state.colonists[colonist_index].state = ColonistState::OnMission { mission_id };
        state.colonists[colonist_index].current_activity =
            crate::data::schedule::ActivityType::Work;
        state.colonists[colonist_index].activity_location = ActivityLocation::None;
        state.colonists[colonist_index].active_mission_id = Some(mission_id);

        state.missions.active_missions.push(ActiveMission {
            id: mission_id,
            colonist_id: state.colonists[colonist_index].id,
            mission_type,
            started_tick: state.tick,
            completes_at_tick,
            danger_percent,
            priority,
        });
        state.missions.next_launch_tick = state.tick + definition.cooldown_minutes;

        state.push_log(
            LogCategory::Mission,
            format!("{} started {}", colonist_name, definition.name),
            format!(
                "{} Duration {}m, danger {}% after {} priority. Crew regroups for {}m.",
                definition.reward_profile,
                definition.duration_minutes,
                danger_percent,
                priority.label(),
                definition.cooldown_minutes
            ),
        );

        Ok(mission_id)
    }

    pub fn process_completed_missions(state: &mut GameState) {
        if state.missions.active_missions.is_empty() {
            return;
        }

        let mut completed = Vec::new();
        state.missions.active_missions.retain(|mission| {
            if mission.completes_at_tick <= state.tick {
                completed.push(mission.clone());
                false
            } else {
                true
            }
        });

        for mission in completed {
            Self::complete_mission(state, mission);
        }
    }

    pub fn recover_injured_colonists(state: &mut GameState) {
        let recovered_names = state
            .colonists
            .iter_mut()
            .filter_map(|colonist| {
                let recovered = colonist
                    .injured_until_tick
                    .is_some_and(|recovery_tick| recovery_tick <= state.tick);
                if recovered {
                    colonist.injured_until_tick = None;
                    Some(colonist.name.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for name in recovered_names {
            state.push_log(
                LogCategory::Mission,
                format!("{} recovered", name),
                "They can be assigned to missions again.".to_string(),
            );
        }
    }

    fn find_available_mission_colonist(state: &GameState) -> Option<usize> {
        state
            .colonists
            .iter()
            .position(|colonist| {
                colonist.job_preference == JobPreference::Explorer
                    && colonist.can_start_mission(state.tick)
            })
            .or_else(|| {
                state
                    .colonists
                    .iter()
                    .position(|colonist| colonist.can_start_mission(state.tick))
            })
    }

    fn complete_mission(state: &mut GameState, mission: ActiveMission) {
        let item = Self::item_for_mission(&mission);
        let injured = Self::mission_caused_injury(&mission);
        let injury_duration = state.technology.injury_duration_ticks();
        let definition = mission.mission_type.definition();

        let colonist_name = if let Some(colonist) = state
            .colonists
            .iter_mut()
            .find(|colonist| colonist.id == mission.colonist_id)
        {
            colonist.active_mission_id = None;
            colonist.activity_location = ActivityLocation::None;
            colonist.state = ColonistState::Idle;

            if injured {
                colonist.injured_until_tick = Some(state.tick + injury_duration);
                colonist.mood = (colonist.mood - 10.0).clamp(0.0, 100.0);
            }

            colonist.name.clone()
        } else {
            format!("Colonist {}", mission.colonist_id)
        };

        let (base_supplies, base_salvage) = Self::base_resources_for_mission(&mission);
        let (item_supplies, item_salvage) = Self::resources_for_item(item);
        let supplies = base_supplies + item_supplies;
        let salvage = base_salvage + item_salvage;
        let wasted_supplies = if supplies > 0 {
            ResourceSystem::add_supplies_from_work(state, supplies)
        } else {
            0
        };
        if salvage > 0 {
            state.resources.add_salvage(salvage);
        }

        let unlocked = state.technology.add_item(item);

        state.push_log(
            LogCategory::Mission,
            format!("{} returned from {}", colonist_name, definition.name),
            Self::mission_detail(&mission, item, supplies, salvage, wasted_supplies),
        );

        if injured {
            state.push_log(
                LogCategory::Mission,
                format!("{} was hurt", colonist_name),
                format!(
                    "{} risk caught up with the crew at {}% danger. Recovery takes {} minutes.",
                    definition.name, mission.danger_percent, injury_duration
                ),
            );
        }

        for tech_id in unlocked {
            state.push_log(
                LogCategory::Technology,
                format!("Technology unlocked: {}", tech_id.name()),
                format!(
                    "{} from {} completed research. {}",
                    item.name(),
                    definition.name,
                    tech_id.effect_text()
                ),
            );
        }

        ResourceSystem::update_condition(state);
    }

    fn item_for_mission(mission: &ActiveMission) -> MissionItem {
        let roll = (mission.id + mission.colonist_id + mission.started_tick as u32) % 6;
        match mission.mission_type {
            MissionType::SupplyRun => match mission.priority {
                ColonyPriority::Recovery => match roll {
                    0 | 1 => MissionItem::MedicinalGel,
                    2 | 3 => MissionItem::NutrientPods,
                    _ => MissionItem::SalvageCache,
                },
                ColonyPriority::Stockpile => match roll {
                    0 | 1 | 2 => MissionItem::SalvageCache,
                    3 | 4 => MissionItem::NutrientPods,
                    _ => MissionItem::StructuralAlloy,
                },
                ColonyPriority::Survey => match roll {
                    0 => MissionItem::AlienCircuit,
                    1 | 2 => MissionItem::NutrientPods,
                    3 => MissionItem::StructuralAlloy,
                    _ => MissionItem::SalvageCache,
                },
            },
            MissionType::PerimeterScan => match mission.priority {
                ColonyPriority::Recovery => match roll {
                    0 | 1 => MissionItem::MedicinalGel,
                    2 => MissionItem::NutrientPods,
                    3 => MissionItem::StructuralAlloy,
                    _ => MissionItem::SalvageCache,
                },
                ColonyPriority::Stockpile => match roll {
                    0 => MissionItem::StructuralAlloy,
                    1 | 4 | 5 => MissionItem::SalvageCache,
                    2 | 3 => MissionItem::NutrientPods,
                    _ => MissionItem::SalvageCache,
                },
                ColonyPriority::Survey => match roll {
                    0 => MissionItem::StructuralAlloy,
                    1 => MissionItem::AlienCircuit,
                    2 | 5 => MissionItem::MedicinalGel,
                    _ => MissionItem::NutrientPods,
                },
            },
            MissionType::DeepSurvey => match mission.priority {
                ColonyPriority::Recovery => match roll {
                    0 | 1 => MissionItem::MedicinalGel,
                    2 => MissionItem::AlienCircuit,
                    3 => MissionItem::StructuralAlloy,
                    _ => MissionItem::NutrientPods,
                },
                ColonyPriority::Stockpile => match roll {
                    0 | 1 => MissionItem::StructuralAlloy,
                    2 => MissionItem::AlienCircuit,
                    3 => MissionItem::MedicinalGel,
                    4 => MissionItem::NutrientPods,
                    _ => MissionItem::SalvageCache,
                },
                ColonyPriority::Survey => match roll {
                    0 | 1 => MissionItem::AlienCircuit,
                    2 => MissionItem::StructuralAlloy,
                    3 => MissionItem::MedicinalGel,
                    _ => MissionItem::NutrientPods,
                },
            },
        }
    }

    fn mission_caused_injury(mission: &ActiveMission) -> bool {
        let roll = (mission.id * 37 + mission.colonist_id * 11 + mission.started_tick as u32) % 100;
        roll < mission.danger_percent
    }

    fn resources_for_item(item: MissionItem) -> (i32, i32) {
        match item {
            MissionItem::StructuralAlloy => (0, 1),
            MissionItem::AlienCircuit => (0, 0),
            MissionItem::MedicinalGel => (0, 0),
            MissionItem::NutrientPods => (3, 0),
            MissionItem::SalvageCache => (0, 4),
        }
    }

    fn base_resources_for_mission(mission: &ActiveMission) -> (i32, i32) {
        match mission.mission_type {
            MissionType::SupplyRun => (6, 1),
            MissionType::PerimeterScan => (2, 1),
            MissionType::DeepSurvey => (0, 2),
        }
    }

    fn mission_detail(
        mission: &ActiveMission,
        item: MissionItem,
        supplies: i32,
        salvage: i32,
        wasted_supplies: i32,
    ) -> String {
        let definition = mission.mission_type.definition();
        let mut detail = format!(
            "{} completed under {} priority. Found {}.",
            definition.name,
            mission.priority.label(),
            item.name()
        );

        if supplies > 0 {
            detail.push_str(&format!(" Stored {} supplies.", supplies - wasted_supplies));
            if wasted_supplies > 0 {
                detail.push_str(&format!(" {} supplies were wasted.", wasted_supplies));
            }
        }

        if salvage > 0 {
            detail.push_str(&format!(" Added {} salvage.", salvage));
        }

        if item.contributes_to_technology() {
            detail.push_str(" Added to technology research.");
        } else {
            detail.push_str(" This was a resource-focused return.");
        }

        detail
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::building::BuildingType;
    use crate::data::colonist::{Colonist, JobPreference, Trait};
    use crate::data::priority::ColonyPriority;
    use crate::data::technology::TechId;
    use crate::data::types::Position;

    fn add_gate(state: &mut GameState) {
        state.data_place_test_building(BuildingType::ExplorationGate);
    }

    trait TestBuildingPlacement {
        fn data_place_test_building(&mut self, building_type: BuildingType);
    }

    impl TestBuildingPlacement for GameState {
        fn data_place_test_building(&mut self, building_type: BuildingType) {
            self.building_system.try_place_building(
                &mut self.grid,
                building_type,
                Position::new(0, 0),
            );
        }
    }

    #[test]
    fn test_launch_creates_typed_mission_and_starts_cooldown() {
        let mut state = GameState::new();
        add_gate(&mut state);
        state.colonists.push(Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        ));

        let mission_id =
            MissionSystem::launch_mission(&mut state, MissionType::DeepSurvey).unwrap();

        assert_eq!(mission_id, 1);
        assert_eq!(
            state.missions.active_missions[0].mission_type,
            MissionType::DeepSurvey
        );
        assert_eq!(
            state.missions.active_missions[0].remaining_ticks(state.tick),
            MissionType::DeepSurvey.definition().duration_minutes
        );
        assert_eq!(
            state.missions.cooldown_remaining(state.tick),
            MissionType::DeepSurvey.definition().cooldown_minutes
        );
        assert!(state.colonists[0].is_on_mission());
    }

    #[test]
    fn test_hurt_colonist_cannot_launch() {
        let mut state = GameState::new();
        add_gate(&mut state);
        let mut colonist = Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        );
        colonist.injured_until_tick = Some(10);
        state.colonists.push(colonist);

        assert_eq!(
            MissionSystem::launch_perimeter_scan(&mut state),
            Err(LaunchMissionError::NoAvailableColonist)
        );
    }

    #[test]
    fn test_mission_cooldown_blocks_rapid_relaunch() {
        let mut state = GameState::new();
        add_gate(&mut state);
        state.colonists.push(Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        ));
        state.colonists.push(Colonist::new(
            2,
            "Backup".to_string(),
            Position::new(4, 3),
            Trait::HardWorker,
            JobPreference::Builder,
        ));

        MissionSystem::launch_mission(&mut state, MissionType::SupplyRun).unwrap();

        assert_eq!(
            MissionSystem::launch_mission(&mut state, MissionType::PerimeterScan),
            Err(LaunchMissionError::MissionCooldown {
                remaining_ticks: MissionType::SupplyRun.definition().cooldown_minutes
            })
        );
    }

    #[test]
    fn test_mission_item_unlocks_technology() {
        let mut state = GameState::new();
        let mission = ActiveMission {
            id: 1,
            colonist_id: 1,
            mission_type: MissionType::PerimeterScan,
            started_tick: 0,
            completes_at_tick: 1,
            danger_percent: 0,
            priority: ColonyPriority::Survey,
        };
        state.colonists.push(Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        ));
        state.missions.active_missions.push(mission);
        state.tick = 1;

        MissionSystem::process_completed_missions(&mut state);

        assert!(state.technology.has(TechId::FieldMedicine));
    }

    #[test]
    fn test_mission_completion_log_names_mission_rewards_and_priority() {
        let mut state = GameState::new();
        let mission = ActiveMission {
            id: 1,
            colonist_id: 1,
            mission_type: MissionType::SupplyRun,
            started_tick: 0,
            completes_at_tick: 1,
            danger_percent: 0,
            priority: ColonyPriority::Stockpile,
        };
        state.colonists.push(Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        ));
        state.missions.active_missions.push(mission);
        state.tick = 1;

        MissionSystem::process_completed_missions(&mut state);

        let log = state
            .event_log
            .iter()
            .find(|entry| entry.title == "Scout returned from Supply Run")
            .expect("mission completion should log the mission name");
        assert!(log.detail.contains("Stockpile priority"));
        assert!(log.detail.contains("Found Salvage Cache"));
        assert!(log.detail.contains("Stored"));
        assert!(log.detail.contains("Added"));
        assert!(log.detail.contains("resource-focused return"));
    }

    #[test]
    fn test_dangerous_mission_can_hurt_colonist() {
        let mut state = GameState::new();
        let mission = ActiveMission {
            id: 1,
            colonist_id: 1,
            mission_type: MissionType::PerimeterScan,
            started_tick: 0,
            completes_at_tick: 1,
            danger_percent: 100,
            priority: ColonyPriority::Stockpile,
        };
        state.colonists.push(Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        ));
        state.missions.active_missions.push(mission);
        state.tick = 1;

        MissionSystem::process_completed_missions(&mut state);

        assert!(state.colonists[0].is_hurt(state.tick));
        assert!(!state.colonists[0].can_start_mission(state.tick));
    }

    #[test]
    fn test_priority_adjusts_mission_danger() {
        let mut state = GameState::new();
        state.priority.active = ColonyPriority::Recovery;
        assert_eq!(MissionSystem::perimeter_scan_danger_percent(&state), 12);

        state.priority.active = ColonyPriority::Survey;
        assert_eq!(MissionSystem::perimeter_scan_danger_percent(&state), 27);
        assert!(MissionSystem::mission_danger_percent(&state, MissionType::DeepSurvey) > 27);
    }

    #[test]
    fn test_priority_changes_visible_mission_recommendation() {
        let mut state = GameState::new();
        state.resources.supplies = 1;
        assert_eq!(
            MissionSystem::recommended_mission_type(&state),
            MissionType::SupplyRun
        );

        state.resources.supplies = 30;
        state.priority.active = ColonyPriority::Recovery;
        assert_eq!(
            MissionSystem::recommended_mission_type(&state),
            MissionType::PerimeterScan
        );

        state.priority.active = ColonyPriority::Survey;
        assert_eq!(
            MissionSystem::recommended_mission_type(&state),
            MissionType::DeepSurvey
        );

        let plans = MissionSystem::mission_plans(&state);
        assert!(plans
            .iter()
            .any(|plan| plan.mission_type == MissionType::DeepSurvey && plan.recommended));
    }

    #[test]
    fn test_survey_priority_favors_research_items() {
        let mission = ActiveMission {
            id: 4,
            colonist_id: 1,
            mission_type: MissionType::PerimeterScan,
            started_tick: 0,
            completes_at_tick: 1,
            danger_percent: 0,
            priority: ColonyPriority::Survey,
        };

        assert_ne!(
            MissionSystem::item_for_mission(&mission),
            MissionItem::SalvageCache
        );
    }
}
