use crate::data::event_log::LogCategory;
use crate::data::game_state::{GameState, TimeSpeed};
use crate::data::resources::ColonyCondition;
use crate::data::scenario::ScenarioOutcome;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::time_system::TimeSystem;

pub struct ScenarioSystem;

impl ScenarioSystem {
    pub fn evaluate(state: &mut GameState) {
        if state.scenario.is_finished() {
            return;
        }

        let (day, _, _) = TimeSystem::get_time_of_day(state.tick);

        if state.resources.condition == ColonyCondition::Collapsed {
            Self::finish(
                state,
                ScenarioOutcome::Failure,
                "Colony failed",
                "The settlement collapsed before a stable landing site could form.",
            );
            return;
        }

        if day >= state.scenario.target_day && Self::meets_victory_requirements(state) {
            Self::finish(
                state,
                ScenarioOutcome::Victory,
                "Stable landing secured",
                "The colony survived the first week with enough infrastructure, knowledge, and supplies to continue.",
            );
        }
    }

    pub fn days_remaining(state: &GameState) -> u32 {
        let (day, _, _) = TimeSystem::get_time_of_day(state.tick);
        state.scenario.target_day.saturating_sub(day)
    }

    pub fn meets_victory_requirements(state: &GameState) -> bool {
        state.resources.condition != ColonyCondition::Collapsed
            && state.resources.condition != ColonyCondition::Critical
            && state.resources.supplies >= ResourceSystem::daily_supply_need(state).max(1)
            && state.technology.unlocked_count() >= state.scenario.required_tech_unlocks
    }

    pub fn objective_line(state: &GameState) -> String {
        let tech_count = state.technology.unlocked_count();
        let tech_required = state.scenario.required_tech_unlocks;
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);
        format!(
            "Survive to Day {} | Tech {}/{} | Supplies {}/{} | {}",
            state.scenario.target_day,
            tech_count,
            tech_required,
            state.resources.supplies,
            daily_need,
            state.resources.condition.label()
        )
    }

    fn finish(
        state: &mut GameState,
        outcome: ScenarioOutcome,
        title: &'static str,
        detail: &'static str,
    ) {
        state.scenario.outcome = outcome;
        state.scenario.outcome_tick = Some(state.tick);
        state.time.speed = TimeSpeed::Paused;
        state.push_log(LogCategory::Colony, title, detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::mission::MissionItem;
    use crate::data::resources::ColonyCondition;

    #[test]
    fn test_victory_requires_day_7_resources_and_tech() {
        let mut state = GameState::new();
        state.tick = TimeSystem::TICKS_PER_DAY * 6;
        state.resources.condition = ColonyCondition::Stable;
        state.resources.supplies = 20;
        state.technology.add_item(MissionItem::MedicinalGel);
        state.technology.add_item(MissionItem::AlienCircuit);
        state.technology.add_item(MissionItem::NutrientPods);

        ScenarioSystem::evaluate(&mut state);

        assert_eq!(state.scenario.outcome, ScenarioOutcome::Victory);
        assert_eq!(state.time.speed, TimeSpeed::Paused);
    }

    #[test]
    fn test_critical_colony_does_not_win_on_target_day() {
        let mut state = GameState::new();
        state.tick = TimeSystem::TICKS_PER_DAY * 6;
        state.resources.condition = ColonyCondition::Critical;
        state.resources.supplies = 20;
        state.technology.add_item(MissionItem::MedicinalGel);
        state.technology.add_item(MissionItem::AlienCircuit);
        state.technology.add_item(MissionItem::NutrientPods);

        ScenarioSystem::evaluate(&mut state);

        assert_eq!(state.scenario.outcome, ScenarioOutcome::InProgress);
    }

    #[test]
    fn test_collapsed_colony_fails() {
        let mut state = GameState::new();
        state.resources.condition = ColonyCondition::Collapsed;

        ScenarioSystem::evaluate(&mut state);

        assert_eq!(state.scenario.outcome, ScenarioOutcome::Failure);
        assert_eq!(state.time.speed, TimeSpeed::Paused);
    }
}
