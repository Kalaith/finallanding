use super::*;

impl GameplayState {
    pub(super) fn update_colonist_selection(&mut self, input: &InputState) {
        if self.selected_building.is_some() || !input.left_pressed {
            return;
        }

        let game_area = self.layout.game_area();
        let mouse_pos = input.mouse_pos;
        let mouse_x = mouse_pos.x;
        let mouse_y = mouse_pos.y;
        let toolbar = self.layout.bottom_toolbar();
        if input.hovered_rect(toolbar)
            || input.hovered_rect(toolbar_context_rect(toolbar))
            || input.hovered_rect(self.layout.left_panel())
            || input.hovered_rect(self.layout.right_panel())
            || mouse_y <= self.layout.top_bar_height
        {
            return;
        }

        if mouse_x < game_area.x
            || mouse_x > game_area.x + game_area.w
            || mouse_y < game_area.y
            || mouse_y > game_area.y + game_area.h
        {
            return;
        }

        if let Some(colonist_id) = self.colonist_id_at_mouse() {
            self.selected_colonist_id = Some(colonist_id);
            return;
        }

        if self.toolbar_mode == ToolbarMode::Assign {
            self.update_assign_space_click();
            return;
        }

        self.selected_colonist_id = None;
    }
}
