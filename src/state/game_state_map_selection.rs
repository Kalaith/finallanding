use super::*;

impl GameplayState {
    pub(super) fn update_colonist_selection(&mut self, input: &InputState) {
        if self.selected_building.is_some() || !input.left_pressed {
            return;
        }

        if !self.pointer_inside_playable_map(input) {
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
