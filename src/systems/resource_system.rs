use crate::data::building::BuildingType;
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::resources::{ColonyCondition, BASE_STORAGE_CAPACITY, STORAGE_CAPACITY_BONUS};

pub struct ResourceSystem;

impl ResourceSystem {
    pub fn daily_supply_need(state: &GameState) -> i32 {
        state.colonists.len() as i32
    }

    pub fn storage_capacity(state: &GameState) -> i32 {
        let storage_count = state
            .building_system
            .buildings()
            .iter()
            .filter(|building| building.building_type == BuildingType::Storage)
            .count() as i32;

        BASE_STORAGE_CAPACITY + storage_count * STORAGE_CAPACITY_BONUS
    }

    pub fn can_afford_building(state: &GameState, building_type: BuildingType) -> bool {
        state.resources.salvage >= building_type.salvage_cost()
    }

    pub fn handle_new_day(state: &mut GameState) {
        let need = Self::daily_supply_need(state);
        let shortage = state.resources.consume_supplies(need);

        if shortage == 0 {
            state.push_log(
                LogCategory::Resource,
                "Daily supplies consumed",
                format!(
                    "{} supplies used. {} remain.",
                    need, state.resources.supplies
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
        let daily_need = Self::daily_supply_need(state).max(1);

        let next = if state.resources.supplies <= 0 && average_mood < 15.0 {
            ColonyCondition::Collapsed
        } else if state.resources.supplies <= 0 || average_mood < 25.0 {
            ColonyCondition::Critical
        } else if state.resources.supplies < daily_need * 2 || average_mood < 45.0 {
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
                    "Supplies {}, salvage {}, average mood {:.0}.",
                    state.resources.supplies, state.resources.salvage, average_mood
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
}
