use crate::data::building::BuildingType;
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::resources::{ColonyCondition, BASE_STORAGE_CAPACITY, STORAGE_CAPACITY_BONUS};

pub struct ResourceSystem;

impl ResourceSystem {
    pub fn daily_supply_need(state: &GameState) -> i32 {
        let missing_food_penalty = if state.colonists.len() >= 4
            && Self::building_count(state, BuildingType::MessHall) == 0
        {
            state.colonists.len() as i32 * 2
        } else {
            0
        };

        (state.colonists.len() as i32 + missing_food_penalty
            - state.resources.prepared_meals
            - state.technology.daily_supply_reduction())
        .max(0)
    }

    pub fn storage_capacity(state: &GameState) -> i32 {
        let storage_count = Self::building_count(state, BuildingType::Storage) as i32;

        BASE_STORAGE_CAPACITY
            + storage_count * STORAGE_CAPACITY_BONUS
            + state.technology.storage_capacity_bonus()
    }

    fn building_count(state: &GameState, building_type: BuildingType) -> usize {
        state
            .building_system
            .buildings()
            .iter()
            .filter(|building| building.building_type == building_type)
            .count()
    }

    pub fn habitat_capacity(state: &GameState) -> u32 {
        let habitat_count = state
            .building_system
            .buildings()
            .iter()
            .filter(|building| building.building_type == BuildingType::Habitat)
            .count() as u32;

        habitat_count * (2 + state.technology.habitat_capacity_bonus())
    }

    pub fn can_afford_building(state: &GameState, building_type: BuildingType) -> bool {
        state.resources.salvage >= building_type.salvage_cost()
    }

    pub fn handle_new_day(state: &mut GameState) {
        let need = Self::daily_supply_need(state);
        let prepared_meals = state.resources.prepared_meals;
        state.resources.prepared_meals = 0;

        let shortage = state.resources.consume_supplies(need);

        if shortage == 0 {
            state.push_log(
                LogCategory::Resource,
                "Daily supplies consumed",
                format!(
                    "{} supplies used after {} prepared meals. {} remain.",
                    need, prepared_meals, state.resources.supplies
                ),
            );
        } else {
            for colonist in &mut state.colonists {
                colonist.mood = (colonist.mood - 12.0 - shortage as f32 * 2.0).clamp(0.0, 100.0);
            }

            state.push_log(
                LogCategory::Resource,
                "Ration crisis",
                format!(
                    "The colony was short {} supplies. Mood dropped across the settlement.",
                    shortage
                ),
            );
        }

        Self::clamp_supplies_to_storage(state);
        Self::update_condition(state);
        Self::log_low_supplies(state);
    }

    pub fn update_condition(state: &mut GameState) {
        let previous = state.resources.condition;
        let average_mood = if state.colonists.is_empty() {
            0.0
        } else {
            state.colonists.iter().map(|c| c.mood).sum::<f32>() / state.colonists.len() as f32
        };
        let shelter_deficit =
            (state.colonists.len() as u32).saturating_sub(Self::habitat_capacity(state));
        let shelter_pressure = shelter_deficit as f32 * 6.0;
        let effective_mood = (average_mood - shelter_pressure).max(0.0);
        let daily_need = Self::daily_supply_need(state).max(1);

        let next = if state.resources.supplies <= 0 && effective_mood < 15.0 {
            ColonyCondition::Collapsed
        } else if state.resources.supplies <= 0 || effective_mood < 25.0 {
            ColonyCondition::Critical
        } else if state.resources.supplies < daily_need * 2
            || effective_mood < 45.0
            || shelter_deficit > 0
        {
            ColonyCondition::Strained
        } else {
            ColonyCondition::Stable
        };

        if next == ColonyCondition::Stable {
            state.resources.stable_days += 1;
        } else {
            state.resources.stable_days = 0;
        }

        state.resources.condition = next;

        if previous != next {
            state.push_log(
                LogCategory::Colony,
                format!("Colony status: {}", next.label()),
                format!(
                    "Supplies {}, salvage {}, average mood {:.0}, shelter deficit {}.",
                    state.resources.supplies,
                    state.resources.salvage,
                    average_mood,
                    shelter_deficit
                ),
            );
        }
    }

    pub fn clamp_supplies_to_storage(state: &mut GameState) {
        let capacity = Self::storage_capacity(state);
        if state.resources.supplies > capacity {
            state.resources.supplies = capacity;
        }
    }

    pub fn add_supplies_from_work(state: &mut GameState, amount: i32) -> i32 {
        let capacity = Self::storage_capacity(state);
        state.resources.add_supplies(amount, capacity)
    }

    fn log_low_supplies(state: &mut GameState) {
        let daily_need = Self::daily_supply_need(state).max(1);
        if state.resources.supplies > daily_need * 2 {
            return;
        }

        state.push_log(
            LogCategory::Resource,
            "Supplies are low",
            format!(
                "{} supplies remain against a daily need of {}.",
                state.resources.supplies, daily_need
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{Colonist, JobPreference, Trait};
    use crate::data::mission::MissionItem;
    use crate::data::types::Position;

    #[test]
    fn test_daily_supply_need_matches_colonists() {
        let mut state = GameState::new();
        state.colonists.push(Colonist::new(
            1,
            "Test".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));

        assert_eq!(ResourceSystem::daily_supply_need(&state), 1);
    }

    #[test]
    fn test_prepared_meals_reduce_daily_need() {
        let mut state = GameState::new();
        state.resources.prepared_meals = 1;
        state.colonists.push(Colonist::new(
            1,
            "Test".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));

        assert_eq!(ResourceSystem::daily_supply_need(&state), 0);
    }

    #[test]
    fn test_hydroponic_technology_reduces_daily_need() {
        let mut state = GameState::new();
        state
            .technology
            .add_item(crate::data::mission::MissionItem::NutrientPods);
        state.colonists.push(Colonist::new(
            1,
            "Test".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));

        assert_eq!(ResourceSystem::daily_supply_need(&state), 0);
    }

    #[test]
    fn test_nutrient_culture_reduces_daily_need_further() {
        let mut state = GameState::new();
        state.technology.add_item(MissionItem::NutrientPods);
        state.technology.add_item(MissionItem::NutrientPods);
        state.technology.add_item(MissionItem::MedicinalGel);
        for id in 0..4 {
            state.colonists.push(Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            ));
        }

        assert_eq!(state.technology.daily_supply_reduction(), 2);
        assert_eq!(ResourceSystem::daily_supply_need(&state), 10);
    }

    #[test]
    fn test_ration_shortage_reduces_mood() {
        let mut state = GameState::new();
        state.resources.supplies = 0;
        state.colonists.push(Colonist::new(
            1,
            "Test".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));

        ResourceSystem::handle_new_day(&mut state);

        assert!(state.colonists[0].mood < 50.0);
        assert_eq!(state.resources.condition, ColonyCondition::Critical);
    }

    #[test]
    fn test_missing_habitat_capacity_strains_condition() {
        let mut state = GameState::new();
        state.resources.supplies = 20;
        for id in 0..6 {
            state.colonists.push(Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            ));
        }

        ResourceSystem::update_condition(&mut state);

        assert_eq!(ResourceSystem::habitat_capacity(&state), 0);
        assert_eq!(state.resources.condition, ColonyCondition::Critical);
    }
}
