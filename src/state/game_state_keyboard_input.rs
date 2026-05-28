use super::*;

impl GameplayState {
    pub(super) fn update_building_selection(&mut self) {
        // Number keys select buildings (Q, W, E, R, T for 5 buildings)
        if is_key_pressed(KeyCode::Q) {
            self.toggle_building(BuildingType::Habitat);
            self.toolbar_mode = ToolbarMode::Build;
        }
        if is_key_pressed(KeyCode::W) {
            self.toggle_building(BuildingType::MessHall);
            self.toolbar_mode = ToolbarMode::Build;
        }
        if is_key_pressed(KeyCode::E) {
            self.toggle_building(BuildingType::Workshop);
            self.toolbar_mode = ToolbarMode::Build;
        }
        if is_key_pressed(KeyCode::R) {
            self.toggle_building(BuildingType::Storage);
            self.toolbar_mode = ToolbarMode::Build;
        }
        if is_key_pressed(KeyCode::T) {
            self.toggle_building(BuildingType::ExplorationGate);
            self.toolbar_mode = ToolbarMode::Build;
        }
        if is_key_pressed(KeyCode::M) {
            self.toolbar_mode = ToolbarMode::Research;
            self.launch_recommended_mission();
        }

        // Escape to cancel building mode
        if is_key_pressed(KeyCode::Escape) {
            self.selected_building = None;
        }

        // Z for undo
        if is_key_pressed(KeyCode::Z) {
            self.undo_last_building();
        }
    }
}
