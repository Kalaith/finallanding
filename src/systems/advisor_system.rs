use crate::data::building::BuildingType;
use crate::data::game_state::GameState;
use crate::data::priority::ColonyPriority;
use crate::data::resources::ColonyCondition;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::summary_system::SummarySystem;
use crate::systems::time_system::TimeSystem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvisorSeverity {
    Stable,
    Action,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvisorLine {
    pub title: String,
    pub detail: String,
    pub severity: AdvisorSeverity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvisorPlan {
    pub headline: String,
    pub lines: Vec<AdvisorLine>,
}

pub struct AdvisorSystem;

impl AdvisorSystem {
    pub fn plan(state: &GameState) -> AdvisorPlan {
        let mut lines = Vec::new();
        let (day, _, _) = TimeSystem::get_time_of_day(state.tick);

        Self::add_incident_guidance(state, &mut lines);
        Self::add_pressure_warnings(state, &mut lines);
        Self::add_building_guidance(state, &mut lines);
        Self::add_progress_guidance(state, &mut lines);

        if lines.is_empty() {
            lines.push(AdvisorLine {
                title: "Hold the landing site".to_string(),
                detail: format!(
                    "Day {} of {}; keep supplies above daily need and relationships stable.",
                    day, state.scenario.target_day
                ),
                severity: AdvisorSeverity::Stable,
            });
        }

        AdvisorPlan {
            headline: Self::headline(state, day),
            lines,
        }
    }

    fn headline(state: &GameState, day: u32) -> String {
        format!(
            "Advisor | Day {}/{} | {}",
            day,
            state.scenario.target_day,
            state.priority.active.label()
        )
    }

    fn add_pressure_warnings(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        let summary = SummarySystem::colony_pressure_summary(state);

        if state.resources.condition == ColonyCondition::Critical
            || state.resources.condition == ColonyCondition::Collapsed
        {
            lines.push(AdvisorLine {
                title: "Stabilize the colony".to_string(),
                detail: "Prioritize food, recovery space, and safer mission timing.".to_string(),
                severity: AdvisorSeverity::Warning,
            });
        } else if state.resources.supplies < daily_need * 2 {
            lines.push(AdvisorLine {
                title: "Raise the supply buffer".to_string(),
                detail: format!(
                    "{} supplies against {} daily need is a thin reserve.",
                    state.resources.supplies, daily_need
                ),
                severity: AdvisorSeverity::Warning,
            });
        }

        if summary.average_mood < 35.0 {
            lines.push(AdvisorLine {
                title: "Give people recovery time".to_string(),
                detail: "Low mood increases refusals and can push the colony critical.".to_string(),
                severity: AdvisorSeverity::Warning,
            });
        } else if summary.strained_pairs > 1 {
            lines.push(AdvisorLine {
                title: "Ease social strain".to_string(),
                detail: format!(
                    "{} tense pairs are adding pressure to daily work.",
                    summary.strained_pairs
                ),
                severity: AdvisorSeverity::Action,
            });
        }
    }

    fn add_incident_guidance(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        if let Some(incident_type) = state.incidents.active_incident(state.tick) {
            lines.push(AdvisorLine {
                title: incident_type.advisor_title().to_string(),
                detail: incident_type.advisor_detail().to_string(),
                severity: AdvisorSeverity::Warning,
            });
        }
    }

    fn add_building_guidance(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        let habitat_capacity = ResourceSystem::habitat_capacity(state);
        if habitat_capacity < state.colonists.len() as u32 {
            lines.push(AdvisorLine {
                title: "Shelter every survivor".to_string(),
                detail: format!(
                    "{} sleeper slots for {} colonists.",
                    habitat_capacity,
                    state.colonists.len()
                ),
                severity: AdvisorSeverity::Action,
            });
        }

        for (building_type, title, detail) in [
            (
                BuildingType::MessHall,
                "Open a meal point",
                "Cook labor creates meals that reduce the daily supply draw.",
            ),
            (
                BuildingType::Workshop,
                "Recover repair stock",
                "Builder labor turns wreckage into usable salvage.",
            ),
            (
                BuildingType::Storage,
                "Secure storage",
                "Storage raises the supply cap before survey finds overflow.",
            ),
            (
                BuildingType::ExplorationGate,
                "Mark a survey route",
                "Survey missions bring tech items and emergency resources.",
            ),
        ] {
            if Self::building_count(state, building_type) == 0 {
                lines.push(AdvisorLine {
                    title: title.to_string(),
                    detail: detail.to_string(),
                    severity: AdvisorSeverity::Action,
                });
            }
        }
    }

    fn add_progress_guidance(state: &GameState, lines: &mut Vec<AdvisorLine>) {
        if state.technology.unlocked_count() < state.scenario.required_tech_unlocks {
            let detail = if state.missions.active_count() > 0 {
                format!(
                    "Survey team away; tech progress {}/{}.",
                    state.technology.unlocked_count(),
                    state.scenario.required_tech_unlocks
                )
            } else if Self::building_count(state, BuildingType::ExplorationGate) > 0 {
                format!(
                    "Launch scans until tech reaches {}/{}.",
                    state.technology.unlocked_count(),
                    state.scenario.required_tech_unlocks
                )
            } else {
                "Build an Exploration Gate before Day 7 tech falls behind.".to_string()
            };

            lines.push(AdvisorLine {
                title: "Push toward field tech".to_string(),
                detail,
                severity: AdvisorSeverity::Action,
            });
        }

        if state.priority.active != ColonyPriority::Survey
            && state.technology.unlocked_count() < state.scenario.required_tech_unlocks
        {
            lines.push(AdvisorLine {
                title: "Use Survey priority".to_string(),
                detail: "Survey boosts exploration output and research-item returns.".to_string(),
                severity: AdvisorSeverity::Action,
            });
        }
    }

    fn building_count(state: &GameState, building_type: BuildingType) -> usize {
        state
            .building_system
            .buildings()
            .iter()
            .filter(|building| building.building_type == building_type)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{Colonist, JobPreference, Trait};
    use crate::data::incident::IncidentType;
    use crate::data::mission::MissionItem;
    use crate::data::types::Position;

    fn add_colonist(state: &mut GameState, id: u32) {
        state.colonists.push(Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));
    }

    fn place(state: &mut GameState, building_type: BuildingType, position: Position) {
        state
            .building_system
            .try_place_building(&mut state.grid, building_type, position);
    }

    #[test]
    fn test_advisor_starts_with_shelter_and_core_buildings() {
        let mut state = GameState::new();
        add_colonist(&mut state, 1);
        add_colonist(&mut state, 2);
        add_colonist(&mut state, 3);

        let plan = AdvisorSystem::plan(&state);

        assert!(plan
            .lines
            .iter()
            .any(|line| line.title == "Shelter every survivor"));
        assert!(plan
            .lines
            .iter()
            .any(|line| line.title == "Open a meal point"));
        assert!(plan
            .lines
            .iter()
            .any(|line| line.title == "Mark a survey route"));
    }

    #[test]
    fn test_advisor_prompts_missions_when_core_is_built() {
        let mut state = GameState::new();
        add_colonist(&mut state, 1);
        add_colonist(&mut state, 2);
        place(&mut state, BuildingType::Habitat, Position::new(0, 0));
        place(&mut state, BuildingType::MessHall, Position::new(3, 0));
        place(&mut state, BuildingType::Workshop, Position::new(7, 0));
        place(&mut state, BuildingType::Storage, Position::new(10, 0));
        place(
            &mut state,
            BuildingType::ExplorationGate,
            Position::new(13, 0),
        );
        state.priority.active = ColonyPriority::Survey;

        let plan = AdvisorSystem::plan(&state);

        assert!(plan
            .lines
            .iter()
            .any(|line| line.title == "Push toward field tech"));
    }

    #[test]
    fn test_advisor_settles_after_victory_requirements_are_met() {
        let mut state = GameState::new();
        add_colonist(&mut state, 1);
        place(&mut state, BuildingType::Habitat, Position::new(0, 0));
        place(&mut state, BuildingType::MessHall, Position::new(3, 0));
        place(&mut state, BuildingType::Workshop, Position::new(7, 0));
        place(&mut state, BuildingType::Storage, Position::new(10, 0));
        place(
            &mut state,
            BuildingType::ExplorationGate,
            Position::new(13, 0),
        );
        state.resources.supplies = 20;
        state.technology.add_item(MissionItem::MedicinalGel);
        state.technology.add_item(MissionItem::AlienCircuit);
        state.technology.add_item(MissionItem::NutrientPods);

        let plan = AdvisorSystem::plan(&state);

        assert!(plan
            .lines
            .iter()
            .any(|line| line.title == "Hold the landing site"));
    }

    #[test]
    fn test_active_incident_creates_advisor_priority() {
        let mut state = GameState::new();
        state
            .incidents
            .activate(IncidentType::ToolBreakage, state.tick, 60);

        let plan = AdvisorSystem::plan(&state);

        assert_eq!(plan.lines[0].title, "Recover repair stock");
        assert_eq!(plan.lines[0].severity, AdvisorSeverity::Warning);
    }
}
