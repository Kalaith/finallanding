use crate::data::building::BuildingType;
use crate::data::game_state::GameState;
use crate::data::types::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildingPlacementFeedback {
    pub building_type: BuildingType,
    pub footprint: (u32, u32),
    pub cost: i32,
    pub purpose: &'static str,
    pub helps: &'static str,
    pub impact: &'static str,
    pub invalid_reason: Option<String>,
}

impl BuildingPlacementFeedback {
    pub fn can_place(&self) -> bool {
        self.invalid_reason.is_none()
    }
}

pub struct PlanningSystem;

impl PlanningSystem {
    pub fn building_feedback(
        state: &GameState,
        building_type: BuildingType,
        position: Position,
    ) -> BuildingPlacementFeedback {
        let footprint = building_type.size();
        let invalid_reason = Self::invalid_reason(state, building_type, position);

        BuildingPlacementFeedback {
            building_type,
            footprint,
            cost: building_type.salvage_cost(),
            purpose: building_type.purpose(),
            helps: building_type.planning_role(),
            impact: building_type.placement_impact(),
            invalid_reason,
        }
    }

    pub fn invalid_reason(
        state: &GameState,
        building_type: BuildingType,
        position: Position,
    ) -> Option<String> {
        if state.resources.salvage < building_type.salvage_cost() {
            return Some(format!(
                "Need {} salvage; {} available.",
                building_type.salvage_cost(),
                state.resources.salvage
            ));
        }

        let (width, height) = building_type.size();
        if position.x < 0 || position.y < 0 {
            return Some("Footprint starts outside the map.".to_string());
        }

        if position.x as u32 + width > state.grid.width as u32
            || position.y as u32 + height > state.grid.height as u32
        {
            return Some("Footprint leaves the map.".to_string());
        }

        for dx in 0..width as i32 {
            for dy in 0..height as i32 {
                let check = Position::new(position.x + dx, position.y + dy);
                let Some(cell) = state.grid.get_cell(check.x, check.y) else {
                    return Some("Footprint leaves the map.".to_string());
                };

                if cell.building_id.is_some() {
                    return Some("Footprint overlaps another building.".to_string());
                }

                if !cell.is_walkable() {
                    return Some("Footprint overlaps blocked terrain.".to_string());
                }
            }
        }

        None
    }

    pub fn placement_log_detail(
        feedback: &BuildingPlacementFeedback,
        building_id: u32,
        remaining_salvage: i32,
    ) -> String {
        format!(
            "Added {} support: {} Building #{} cost {} salvage; {} remain.",
            feedback.helps, feedback.impact, building_id, feedback.cost, remaining_salvage
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::grid::CellType;

    #[test]
    fn test_building_feedback_includes_planning_facts() {
        let state = GameState::new();

        let feedback =
            PlanningSystem::building_feedback(&state, BuildingType::MessHall, Position::new(1, 1));

        assert!(feedback.can_place());
        assert_eq!(feedback.footprint, (3, 2));
        assert_eq!(feedback.cost, 12);
        assert_eq!(feedback.helps, "Food");
        assert!(feedback.purpose.contains("Meal"));
        assert!(feedback.impact.contains("cooks"));
    }

    #[test]
    fn test_invalid_reason_reports_insufficient_salvage() {
        let mut state = GameState::new();
        state.resources.salvage = 0;

        let feedback = PlanningSystem::building_feedback(
            &state,
            BuildingType::ExplorationGate,
            Position::new(1, 1),
        );

        assert!(!feedback.can_place());
        assert_eq!(
            feedback.invalid_reason.as_deref(),
            Some("Need 14 salvage; 0 available.")
        );
    }

    #[test]
    fn test_invalid_reason_reports_map_and_overlap_blocks() {
        let mut state = GameState::new();

        assert_eq!(
            PlanningSystem::invalid_reason(
                &state,
                BuildingType::MessHall,
                Position::new(state.grid.width as i32 - 1, state.grid.height as i32 - 1)
            )
            .as_deref(),
            Some("Footprint leaves the map.")
        );

        state.building_system.try_place_building(
            &mut state.grid,
            BuildingType::Habitat,
            Position::new(2, 2),
        );

        assert_eq!(
            PlanningSystem::invalid_reason(&state, BuildingType::Storage, Position::new(2, 2))
                .as_deref(),
            Some("Footprint overlaps another building.")
        );

        state.grid.set_cell_type(8, 8, CellType::Wall);
        assert_eq!(
            PlanningSystem::invalid_reason(&state, BuildingType::Storage, Position::new(8, 8))
                .as_deref(),
            Some("Footprint overlaps blocked terrain.")
        );
    }

    #[test]
    fn test_placement_log_detail_explains_mechanical_change() {
        let state = GameState::new();
        let feedback =
            PlanningSystem::building_feedback(&state, BuildingType::Storage, Position::new(1, 1));

        let detail = PlanningSystem::placement_log_detail(&feedback, 4, 28);

        assert!(detail.contains("Added Storage support"));
        assert!(detail.contains("Adds supply capacity"));
        assert!(detail.contains("Building #4 cost 6 salvage"));
        assert!(detail.contains("28 remain"));
    }
}
