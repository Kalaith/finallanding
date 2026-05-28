use super::*;

impl GameplayState {
    pub(super) fn update_pointer_ui_input(&mut self, input: &InputState) {
        let assign_room_filter_click =
            self.toolbar_mode == ToolbarMode::Assign && input.right_pressed;
        if !input.left_pressed && !assign_room_filter_click {
            return;
        }

        let mouse_x = input.mouse_pos.x;
        let mouse_y = input.mouse_pos.y;

        if assign_room_filter_click {
            if input.hovered_rect(self.layout.game_area()) {
                self.update_assign_building_filter_click();
                return;
            }
            return;
        }

        if mouse_y <= self.layout.top_bar_height {
            self.update_top_bar_click(mouse_x, mouse_y);
            return;
        }

        if self.update_toolbar_click(mouse_x, mouse_y) {
            return;
        }

        let right_panel = self.layout.right_panel();
        if mouse_x >= right_panel.x
            && mouse_x <= right_panel.x + right_panel.w
            && mouse_y >= right_panel.y
            && mouse_y <= right_panel.y + right_panel.h
        {
            return;
        }
    }

    pub(super) fn update_toolbar_click(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
        let toolbar = self.layout.bottom_toolbar();
        if let Some(mode) = toolbar_mode_at(toolbar, mouse_x, mouse_y) {
            self.toolbar_mode = mode;
            if !mode.uses_building_choices() {
                self.selected_building = None;
            } else if self
                .selected_building
                .is_some_and(|building| !toolbar_buildings_for_mode(mode).contains(&building))
            {
                self.selected_building = None;
            }
            return true;
        }

        let context = toolbar_context_rect(toolbar);
        if !context.contains(Vec2::new(mouse_x, mouse_y)) {
            return false;
        }

        match self.toolbar_mode {
            ToolbarMode::Build | ToolbarMode::Rooms | ToolbarMode::Objects => {
                self.handle_build_toolbar_click(context, mouse_x, mouse_y);
            }
            ToolbarMode::Colony => {
                self.handle_colony_toolbar_click(context, mouse_x, mouse_y);
            }
            ToolbarMode::Research => {
                self.handle_research_toolbar_click(context, mouse_x, mouse_y);
            }
            ToolbarMode::Assign => {
                self.handle_assign_toolbar_click(context, mouse_x, mouse_y);
            }
            ToolbarMode::Log => {
                self.handle_log_toolbar_click(context, mouse_x, mouse_y);
            }
        }

        true
    }
}
