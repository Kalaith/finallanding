use crate::data::building::BuildingType;
use crate::data::game_state::GameState;
use crate::data::resources::ColonyCondition;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::scenario_system::ScenarioSystem;
use crate::systems::time_system::TimeSystem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveStatus {
    Complete,
    Active,
    AtRisk,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectiveCard {
    pub title: String,
    pub detail: String,
    pub progress: f32,
    pub status: ObjectiveStatus,
}

pub struct ObjectiveSystem;

impl ObjectiveSystem {
    pub fn active_cards(state: &GameState) -> Vec<ObjectiveCard> {
        let mut cards = vec![
            Self::landing_card(state),
            Self::shelter_card(state),
            Self::food_card(state),
            Self::core_rooms_card(state),
            Self::technology_card(state),
        ];

        cards.sort_by_key(|card| match card.status {
            ObjectiveStatus::AtRisk => 0,
            ObjectiveStatus::Active => 1,
            ObjectiveStatus::Complete => 2,
        });
        cards.truncate(4);
        cards
    }

    fn landing_card(state: &GameState) -> ObjectiveCard {
        let (day, _, _) = TimeSystem::get_time_of_day(state.tick);
        let target_day = state.scenario.target_day.max(1);
        let progress = day as f32 / target_day as f32;
        let status = if ScenarioSystem::meets_victory_requirements(state) {
            ObjectiveStatus::Complete
        } else if state.resources.condition == ColonyCondition::Critical
            || state.resources.condition == ColonyCondition::Collapsed
        {
            ObjectiveStatus::AtRisk
        } else {
            ObjectiveStatus::Active
        };

        ObjectiveCard {
            title: "Secure stable landing".to_string(),
            detail: format!(
                "Day {} of {} | {}",
                day,
                target_day,
                state.resources.condition.label()
            ),
            progress,
            status,
        }
    }

    fn shelter_card(state: &GameState) -> ObjectiveCard {
        let needed = state.colonists.len().max(1) as u32;
        let capacity = ResourceSystem::habitat_capacity(state);
        let progress = capacity as f32 / needed as f32;
        let complete = capacity >= state.colonists.len() as u32;

        ObjectiveCard {
            title: "Shelter every survivor".to_string(),
            detail: format!("{} beds for {} colonists", capacity, state.colonists.len()),
            progress,
            status: if complete {
                ObjectiveStatus::Complete
            } else {
                ObjectiveStatus::AtRisk
            },
        }
    }

    fn food_card(state: &GameState) -> ObjectiveCard {
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        let target_buffer = (daily_need * 2).max(1);
        let progress = state.resources.supplies as f32 / target_buffer as f32;
        let status = if state.resources.supplies >= target_buffer {
            ObjectiveStatus::Complete
        } else if state.resources.supplies < daily_need {
            ObjectiveStatus::AtRisk
        } else {
            ObjectiveStatus::Active
        };

        ObjectiveCard {
            title: "Hold a food buffer".to_string(),
            detail: format!(
                "{} food vs {} daily need",
                state.resources.supplies, daily_need
            ),
            progress,
            status,
        }
    }

    fn core_rooms_card(state: &GameState) -> ObjectiveCard {
        let placed = BuildingType::all()
            .iter()
            .filter(|building_type| Self::has_building(state, **building_type))
            .count();
        let total = BuildingType::all().len();

        ObjectiveCard {
            title: "Establish core rooms".to_string(),
            detail: format!("{} of {} room types placed", placed, total),
            progress: placed as f32 / total as f32,
            status: if placed == total {
                ObjectiveStatus::Complete
            } else {
                ObjectiveStatus::Active
            },
        }
    }

    fn technology_card(state: &GameState) -> ObjectiveCard {
        let required = state.scenario.required_tech_unlocks.max(1);
        let unlocked = state.technology.unlocked_count();
        let has_gate = Self::has_building(state, BuildingType::ExplorationGate);

        ObjectiveCard {
            title: "Recover field technology".to_string(),
            detail: format!("{} of {} tech unlocked", unlocked, required),
            progress: unlocked as f32 / required as f32,
            status: if unlocked >= required {
                ObjectiveStatus::Complete
            } else if !has_gate {
                ObjectiveStatus::AtRisk
            } else {
                ObjectiveStatus::Active
            },
        }
    }

    fn has_building(state: &GameState, building_type: BuildingType) -> bool {
        state
            .building_system
            .buildings()
            .iter()
            .any(|building| building.building_type == building_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{Colonist, JobPreference, Trait};
    use crate::data::mission::MissionItem;
    use crate::data::types::Position;

    fn add_colonists(state: &mut GameState, count: u32) {
        for id in 0..count {
            state.colonists.push(Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            ));
        }
    }

    fn place(state: &mut GameState, building_type: BuildingType, x: i32) {
        state.building_system.try_place_building(
            &mut state.grid,
            building_type,
            Position::new(x, 0),
        );
    }

    #[test]
    fn test_objectives_flag_missing_shelter_as_risk() {
        let mut state = GameState::new();
        add_colonists(&mut state, 4);

        let cards = ObjectiveSystem::active_cards(&state);
        let shelter = cards
            .iter()
            .find(|card| card.title == "Shelter every survivor")
            .expect("shelter card should be visible when at risk");

        assert_eq!(shelter.status, ObjectiveStatus::AtRisk);
        assert!(shelter.detail.contains("0 beds"));
    }

    #[test]
    fn test_objectives_track_core_rooms_and_technology() {
        let mut state = GameState::new();
        add_colonists(&mut state, 2);
        place(&mut state, BuildingType::Habitat, 0);
        place(&mut state, BuildingType::MessHall, 3);
        place(&mut state, BuildingType::Workshop, 7);
        place(&mut state, BuildingType::Storage, 10);
        place(&mut state, BuildingType::ExplorationGate, 13);
        state.technology.add_item(MissionItem::MedicinalGel);
        state.technology.add_item(MissionItem::AlienCircuit);
        state.technology.add_item(MissionItem::NutrientPods);

        let core = ObjectiveSystem::core_rooms_card(&state);
        let tech = ObjectiveSystem::technology_card(&state);

        assert_eq!(core.status, ObjectiveStatus::Complete);
        assert_eq!(tech.status, ObjectiveStatus::Complete);
    }
}
