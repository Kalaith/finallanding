use super::*;

impl GameplayState {
    pub(super) fn toggle_building(&mut self, building_type: BuildingType) {
        if self.selected_building == Some(building_type) {
            self.selected_building = None;
        } else {
            self.selected_building = Some(building_type);
        }
    }

    pub(super) fn undo_last_building(&mut self) {
        let refund = self
            .data
            .building_system
            .last_placed_building()
            .map(|building| {
                (
                    building.id,
                    building.building_type,
                    building.building_type.salvage_cost(),
                )
            });

        if let Some(building_id) = self
            .data
            .building_system
            .undo_last_placement(&mut self.data.grid)
        {
            let cleared_assignments = self.clear_building_assignments(building_id);
            let assignment_note = if cleared_assignments.is_empty() {
                String::new()
            } else {
                format!(
                    " Cleared room pins for {}.",
                    truncate_text(&cleared_assignments.join(", "), 46)
                )
            };

            if let Some((refund_id, building_type, salvage_cost)) = refund {
                if refund_id == building_id {
                    self.data.resources.refund_salvage(salvage_cost);
                    self.data.push_log(
                        LogCategory::System,
                        "Building plan undone",
                        format!(
                            "Removed {} #{} and refunded {} salvage.{}",
                            building_type.name(),
                            building_id,
                            salvage_cost,
                            assignment_note
                        ),
                    );
                    return;
                }
            }

            self.data.push_log(
                LogCategory::System,
                "Building plan undone",
                format!(
                    "Removed building #{} from the settlement plan.{}",
                    building_id, assignment_note
                ),
            );
        }
    }

    pub(super) fn update_building_placement(&mut self, input: &InputState) {
        // Only allow placement in the game area (not over UI)
        let mouse_pos = input.mouse_pos;
        let mouse_x = mouse_pos.x;
        let mouse_y = mouse_pos.y;
        let game_area = self.layout.game_area();
        let toolbar = self.layout.bottom_toolbar();

        if input.hovered_rect(toolbar)
            || input.hovered_rect(toolbar_context_rect(toolbar))
            || input.hovered_rect(self.layout.left_panel())
            || input.hovered_rect(self.layout.right_panel())
            || mouse_y <= self.layout.top_bar_height
        {
            return;
        }

        if mouse_x < game_area.x || mouse_x > game_area.x + game_area.w {
            return;
        }
        if mouse_y < game_area.y || mouse_y > game_area.y + game_area.h {
            return;
        }

        let Some(building_type) = self.selected_building else {
            return;
        };

        if input.left_pressed {
            let pos = self.iso_view().screen_to_grid(vec2(mouse_x, mouse_y));
            let feedback = PlanningSystem::building_feedback(&self.data, building_type, pos);
            if let Some(reason) = feedback.invalid_reason.as_ref() {
                self.data.push_log(
                    LogCategory::System,
                    format!("Cannot place {}", building_type.name()),
                    format!(
                        "{} {} helps {}: {}",
                        reason,
                        building_type.name(),
                        feedback.helps,
                        feedback.purpose
                    ),
                );
                return;
            }

            let result = self.data.building_system.try_place_building(
                &mut self.data.grid,
                building_type,
                pos,
            );
            let result_reason = placement_result_reason(&result);

            match result {
                PlacementResult::Success(building_id) => {
                    self.data
                        .resources
                        .spend_salvage(building_type.salvage_cost());
                    self.data.push_log(
                        LogCategory::System,
                        format!("{} placed", building_type.name()),
                        PlanningSystem::placement_log_detail(
                            &feedback,
                            building_id,
                            self.data.resources.salvage,
                        ),
                    );
                }
                PlacementResult::OutOfBounds
                | PlacementResult::AreaOccupied
                | PlacementResult::InvalidBuilding => {
                    self.data.push_log(
                        LogCategory::System,
                        format!("Cannot place {}", building_type.name()),
                        result_reason.to_string(),
                    );
                }
            }
        }
    }

    pub(super) fn handle_build_toolbar_click(&mut self, context: Rect, mouse_x: f32, mouse_y: f32) {
        if let Some(building_type) =
            toolbar_building_at_for_mode(context, self.toolbar_mode, mouse_x, mouse_y)
        {
            self.toggle_building(building_type);
        }
    }
}
