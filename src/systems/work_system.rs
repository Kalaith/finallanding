use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, ColonistState, JobPreference, Trait};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::systems::resource_system::ResourceSystem;

const EXPLORATION_THRESHOLD: u32 = 8;
const WORKSHOP_THRESHOLD: u32 = 6;
const KITCHEN_THRESHOLD: u32 = 4;
const HAULING_THRESHOLD: u32 = 5;

pub struct WorkSystem;

impl WorkSystem {
    pub fn process_hourly_work(state: &mut GameState) {
        let mut exploration_output = 0;
        let mut workshop_output = 0;
        let mut kitchen_output = 0;
        let mut hauling_output = 0;
        let priority = state.priority.active;

        for colonist in &state.colonists {
            if colonist.state != ColonistState::Working {
                continue;
            }

            let ActivityLocation::Building { building_type, .. } = colonist.activity_location
            else {
                continue;
            };

            let output = priority.adjust_work_output(
                building_type,
                Self::colonist_output(colonist.mood, colonist.trait_data),
            );
            match (building_type, colonist.job_preference) {
                (BuildingType::ExplorationGate, JobPreference::Explorer) => {
                    exploration_output += output;
                }
                (BuildingType::Workshop, JobPreference::Builder) => {
                    workshop_output += output;
                }
                (BuildingType::MessHall, JobPreference::Cook) => {
                    kitchen_output += output;
                }
                (BuildingType::Storage, JobPreference::Hauler) => {
                    hauling_output += output;
                }
                _ => {}
            }
        }

        Self::apply_exploration_output(state, exploration_output);
        Self::apply_workshop_output(state, workshop_output);
        Self::apply_kitchen_output(state, kitchen_output);
        Self::apply_hauling_output(state, hauling_output);
    }

    fn colonist_output(mood: f32, trait_data: Trait) -> u32 {
        let mut output: u32 = if mood >= 70.0 {
            3
        } else if mood >= 35.0 {
            2
        } else {
            1
        };

        match trait_data {
            Trait::HardWorker => output += 1,
            Trait::Lazy => output = output.saturating_sub(1).max(1),
            _ => {}
        }

        output
    }

    fn apply_exploration_output(state: &mut GameState, output: u32) {
        if output == 0 {
            return;
        }

        state.resources.exploration_progress += output;
        let completed = state.resources.exploration_progress / EXPLORATION_THRESHOLD;
        state.resources.exploration_progress %= EXPLORATION_THRESHOLD;

        if completed == 0 {
            return;
        }

        let supplies_found = completed as i32 * 3;
        let salvage_found = completed as i32 * 2;
        let wasted = ResourceSystem::add_supplies_from_work(state, supplies_found);
        state.resources.add_salvage(salvage_found);

        let detail = if wasted > 0 {
            format!(
                "Explorers returned with {} supplies and {} salvage, but {} supplies could not be stored.",
                supplies_found, salvage_found, wasted
            )
        } else {
            format!(
                "Explorers returned with {} supplies and {} salvage.",
                supplies_found, salvage_found
            )
        };

        state.push_log(LogCategory::Resource, "Exploration team returned", detail);
        ResourceSystem::update_condition(state);
    }

    fn apply_workshop_output(state: &mut GameState, output: u32) {
        if output == 0 {
            return;
        }

        state.resources.workshop_progress += output;
        let completed = state.resources.workshop_progress / WORKSHOP_THRESHOLD;
        state.resources.workshop_progress %= WORKSHOP_THRESHOLD;

        if completed == 0 {
            return;
        }

        let salvage = completed as i32;
        state.resources.add_salvage(salvage);
        state.push_log(
            LogCategory::Resource,
            "Workshop recovered salvage",
            format!("Builders recovered {} usable salvage.", salvage),
        );
    }

    fn apply_kitchen_output(state: &mut GameState, output: u32) {
        if output == 0 {
            return;
        }

        state.resources.kitchen_progress += output;
        let completed = state.resources.kitchen_progress / KITCHEN_THRESHOLD;
        state.resources.kitchen_progress %= KITCHEN_THRESHOLD;

        if completed == 0 {
            return;
        }

        let meals = completed as i32;
        state.resources.prepared_meals += meals;
        state.push_log(
            LogCategory::Resource,
            "Meals prepared",
            format!(
                "{} prepared meals will reduce the next daily supply draw.",
                meals
            ),
        );
    }

    fn apply_hauling_output(state: &mut GameState, output: u32) {
        if output == 0 {
            return;
        }

        state.resources.hauling_progress += output;
        let completed = state.resources.hauling_progress / HAULING_THRESHOLD;
        state.resources.hauling_progress %= HAULING_THRESHOLD;

        if completed == 0 {
            return;
        }

        let salvage = completed as i32;
        state.resources.add_salvage(salvage);
        state.push_log(
            LogCategory::Resource,
            "Storage organized",
            format!(
                "Haulers recovered {} salvage from the crash stockpile.",
                salvage
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{ActivityLocation, Colonist, JobPreference};
    use crate::data::priority::ColonyPriority;
    use crate::data::resources::ResourceState;
    use crate::data::types::Position;

    #[test]
    fn test_workshop_generates_salvage() {
        let mut state = GameState::new();
        let mut colonist = Colonist::new(
            1,
            "Builder".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        colonist.state = ColonistState::Working;
        colonist.activity_location = ActivityLocation::Building {
            building_id: 1,
            building_type: BuildingType::Workshop,
        };
        colonist.mood = 70.0;
        state.colonists.push(colonist);

        let salvage_before = state.resources.salvage;
        WorkSystem::process_hourly_work(&mut state);
        WorkSystem::process_hourly_work(&mut state);

        assert!(state.resources.salvage > salvage_before);
    }

    #[test]
    fn test_cook_prepares_meals() {
        let mut state = GameState::new();
        let mut colonist = Colonist::new(
            1,
            "Cook".to_string(),
            Position::new(0, 0),
            Trait::Gourmet,
            JobPreference::Cook,
        );
        colonist.state = ColonistState::Working;
        colonist.activity_location = ActivityLocation::Building {
            building_id: 1,
            building_type: BuildingType::MessHall,
        };
        state.colonists.push(colonist);

        WorkSystem::process_hourly_work(&mut state);
        WorkSystem::process_hourly_work(&mut state);

        assert!(state.resources.prepared_meals > 0);
    }

    #[test]
    fn test_stockpile_priority_boosts_workshop_salvage() {
        let mut state = GameState::new();
        state.priority.active = ColonyPriority::Stockpile;
        let mut colonist = Colonist::new(
            1,
            "Builder".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        colonist.state = ColonistState::Working;
        colonist.activity_location = ActivityLocation::Building {
            building_id: 1,
            building_type: BuildingType::Workshop,
        };
        colonist.mood = 70.0;
        state.colonists.push(colonist);

        WorkSystem::process_hourly_work(&mut state);

        assert_eq!(state.resources.salvage, ResourceState::default().salvage);
        assert_eq!(state.resources.workshop_progress, 5);

        WorkSystem::process_hourly_work(&mut state);

        assert!(state.resources.salvage > ResourceState::default().salvage);
    }

    #[test]
    fn test_survey_priority_boosts_exploration_progress() {
        let mut state = GameState::new();
        state.priority.active = ColonyPriority::Survey;
        let mut colonist = Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Explorer,
        );
        colonist.state = ColonistState::Working;
        colonist.activity_location = ActivityLocation::Building {
            building_id: 1,
            building_type: BuildingType::ExplorationGate,
        };
        colonist.mood = 70.0;
        state.colonists.push(colonist);

        WorkSystem::process_hourly_work(&mut state);

        assert_eq!(state.resources.exploration_progress, 5);
    }
}
