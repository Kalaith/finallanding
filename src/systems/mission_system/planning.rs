use crate::data::game_state::GameState;
use crate::data::mission::{MissionDefinition, MissionType};
use crate::data::priority::ColonyPriority;
use crate::systems::resource_system::ResourceSystem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionPlan {
    pub mission_type: MissionType,
    pub definition: MissionDefinition,
    pub danger_percent: u32,
    pub recommended: bool,
    pub recommendation_reason: String,
    pub cooldown_remaining: u64,
}

pub(super) struct MissionPlanning;

impl MissionPlanning {
    pub(super) fn mission_plans(state: &GameState) -> Vec<MissionPlan> {
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

    pub(super) fn recommended_mission_type(state: &GameState) -> MissionType {
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

    pub(super) fn recommendation_reason(state: &GameState, mission_type: MissionType) -> String {
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

    pub(super) fn mission_danger_percent(state: &GameState, mission_type: MissionType) -> u32 {
        let definition = mission_type.definition();
        let technology_adjusted = definition
            .danger_percent
            .saturating_sub(state.technology.mission_danger_reduction());
        state
            .priority
            .active
            .adjust_mission_danger(technology_adjusted)
    }
}

#[cfg(test)]
mod tests {
    use super::super::MissionSystem;
    use super::*;

    #[test]
    fn test_priority_adjusts_mission_danger() {
        let mut state = GameState::new();
        state.priority.active = ColonyPriority::Recovery;
        assert_eq!(
            MissionPlanning::mission_danger_percent(&state, MissionType::PerimeterScan),
            12
        );

        state.priority.active = ColonyPriority::Survey;
        assert_eq!(
            MissionPlanning::mission_danger_percent(&state, MissionType::PerimeterScan),
            27
        );
        assert!(MissionPlanning::mission_danger_percent(&state, MissionType::DeepSurvey) > 27);
    }

    #[test]
    fn test_priority_changes_visible_mission_recommendation() {
        let mut state = GameState::new();
        state.resources.supplies = 1;
        assert_eq!(
            MissionSystem::recommended_mission_type(&state),
            MissionType::SupplyRun
        );
        assert!(
            MissionPlanning::recommendation_reason(&state, MissionType::SupplyRun)
                .contains("safety buffer")
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
}
