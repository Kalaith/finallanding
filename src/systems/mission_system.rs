use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, ColonistState, JobPreference};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::mission::{ActiveMission, MissionItem, MissionType};
use crate::systems::resource_system::ResourceSystem;

pub struct MissionSystem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchMissionError {
    NoExplorationGate,
    NoAvailableColonist,
}

impl MissionSystem {
    pub fn perimeter_scan_danger_percent(state: &GameState) -> u32 {
        let definition = MissionType::PerimeterScan.definition();
        let technology_adjusted = definition
            .danger_percent
            .saturating_sub(state.technology.mission_danger_reduction());
        state
            .priority
            .active
            .adjust_mission_danger(technology_adjusted)
    }

    pub fn launch_perimeter_scan(state: &mut GameState) -> Result<u32, LaunchMissionError> {
        if !state
            .building_system
            .buildings()
            .iter()
            .any(|building| building.building_type == BuildingType::ExplorationGate)
        {
            return Err(LaunchMissionError::NoExplorationGate);
        }

        let Some(colonist_index) = Self::find_available_mission_colonist(state) else {
            return Err(LaunchMissionError::NoAvailableColonist);
        };

        let definition = MissionType::PerimeterScan.definition();
        let mission_id = state.missions.next_id;
        state.missions.next_id += 1;
        let danger_percent = Self::perimeter_scan_danger_percent(state);
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
            mission_type: MissionType::PerimeterScan,
            started_tick: state.tick,
            completes_at_tick,
            danger_percent,
            priority,
        });

        state.push_log(
            LogCategory::Mission,
            format!("{} started {}", colonist_name, definition.name),
            format!(
                "Mission duration {} minute, danger {}%. Priority: {}.",
                definition.duration_minutes,
                danger_percent,
                priority.label()
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

        let (supplies, salvage) = Self::resources_for_item(item);
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
            format!("{} returned from mission", colonist_name),
            Self::mission_detail(item, supplies, salvage, wasted_supplies),
        );

        if injured {
            state.push_log(
                LogCategory::Mission,
                format!("{} was hurt", colonist_name),
                format!(
                    "Mission danger caused an injury. Recovery takes {} minutes.",
                    injury_duration
                ),
            );
        }

        for tech_id in unlocked {
            state.push_log(
                LogCategory::Technology,
                format!("Technology unlocked: {}", tech_id.name()),
                tech_id.effect_text().to_string(),
            );
        }

        ResourceSystem::update_condition(state);
    }

    fn item_for_mission(mission: &ActiveMission) -> MissionItem {
        let roll = (mission.id + mission.colonist_id + mission.started_tick as u32) % 5;
        match mission.priority {
            crate::data::priority::ColonyPriority::Recovery => match roll {
                0 | 1 => MissionItem::MedicinalGel,
                2 => MissionItem::NutrientPods,
                3 => MissionItem::StructuralAlloy,
                _ => MissionItem::SalvageCache,
            },
            crate::data::priority::ColonyPriority::Stockpile => match roll {
                0 => MissionItem::StructuralAlloy,
                1 | 4 => MissionItem::SalvageCache,
                2 | 3 => MissionItem::NutrientPods,
                _ => MissionItem::SalvageCache,
            },
            crate::data::priority::ColonyPriority::Survey => match roll % 4 {
                0 => MissionItem::StructuralAlloy,
                1 => MissionItem::AlienCircuit,
                2 => MissionItem::MedicinalGel,
                _ => MissionItem::NutrientPods,
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

    fn mission_detail(
        item: MissionItem,
        supplies: i32,
        salvage: i32,
        wasted_supplies: i32,
    ) -> String {
        let mut detail = format!("Recovered {}.", item.name());

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
            detail.push_str(" The item was added to technology research.");
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
    fn test_launch_creates_one_minute_mission() {
        let mut state = GameState::new();
        add_gate(&mut state);
        state.colonists.push(Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        ));

        let mission_id = MissionSystem::launch_perimeter_scan(&mut state).unwrap();

        assert_eq!(mission_id, 1);
        assert_eq!(
            state.missions.active_missions[0].remaining_ticks(state.tick),
            1
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
        assert_eq!(MissionSystem::perimeter_scan_danger_percent(&state), 15);

        state.priority.active = ColonyPriority::Survey;
        assert_eq!(MissionSystem::perimeter_scan_danger_percent(&state), 30);
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
