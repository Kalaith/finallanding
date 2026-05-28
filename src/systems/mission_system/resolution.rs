use crate::data::colonist::{ActivityLocation, ColonistState};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::mission::{ActiveMission, MissionItem, MissionType};
use crate::data::priority::ColonyPriority;
use crate::systems::resource_system::ResourceSystem;

pub(super) struct MissionResolution;

impl MissionResolution {
    pub(super) fn process_completed_missions(state: &mut GameState) {
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

    pub(super) fn recover_injured_colonists(state: &mut GameState) {
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
    use crate::data::colonist::{Colonist, JobPreference, Trait};
    use crate::data::technology::TechId;
    use crate::data::types::Position;

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

        MissionResolution::process_completed_missions(&mut state);

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

        MissionResolution::process_completed_missions(&mut state);

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

        MissionResolution::process_completed_missions(&mut state);

        assert!(state.colonists[0].is_hurt(state.tick));
        assert!(!state.colonists[0].can_start_mission(state.tick));
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
            MissionResolution::item_for_mission(&mission),
            MissionItem::SalvageCache
        );
    }
}
