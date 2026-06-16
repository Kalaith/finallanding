use super::*;

impl GameplayState {
    pub(super) fn set_priority(&mut self, priority: ColonyPriority) {
        if self.data.priority.active == priority {
            return;
        }

        self.data.priority.active = priority;
        self.data.push_log(
            LogCategory::System,
            format!("Priority set: {}", priority.label()),
            priority.description(),
        );
    }

    // Handle building placement via mouse click

    pub(super) fn handle_colony_toolbar_click(
        &mut self,
        context: Rect,
        mouse_x: f32,
        mouse_y: f32,
    ) {
        if let Some(priority) = toolbar_priority_at(context, mouse_x, mouse_y) {
            self.set_priority(priority);
        }
    }

    pub(super) fn update_top_bar_click(&mut self, mouse_x: f32, mouse_y: f32) {
        if let Some(speed) = top_bar_speed_at(mouse_x, mouse_y) {
            self.data.time.speed = speed;
            return;
        }

        if let Some(priority) = top_bar_priority_at(mouse_x, mouse_y) {
            self.set_priority(priority);
        }
    }
}
