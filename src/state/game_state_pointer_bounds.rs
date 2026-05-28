use super::*;

impl GameplayState {
    pub(super) fn pointer_inside_playable_map(&self, input: &InputState) -> bool {
        input.hovered_rect(self.layout.game_area()) && !self.pointer_over_blocking_ui(input)
    }

    fn pointer_over_blocking_ui(&self, input: &InputState) -> bool {
        let toolbar = self.layout.bottom_toolbar();
        input.hovered_rect(toolbar)
            || input.hovered_rect(toolbar_context_rect(toolbar))
            || input.hovered_rect(self.layout.left_panel())
            || input.hovered_rect(self.layout.right_panel())
            || input.mouse_pos.y <= self.layout.top_bar_height
    }
}
