use super::*;

impl GameplayState {
    pub(super) fn update_assign_building_filter_click(&mut self) {
        let clicked = self
            .building_at_mouse()
            .map(|building| (building.id, building.building_type.name().to_string()));
        self.assign_roster_page = 0;

        if let Some((building_id, name)) = clicked {
            self.assign_building_filter = Some(building_id);
            self.data.push_log(
                LogCategory::Social,
                "Assignment room filter set",
                format!(
                    "Assign roster now shows survivors pinned to {} #{}.",
                    name, building_id
                ),
            );
        } else if self.assign_building_filter.take().is_some() {
            self.data.push_log(
                LogCategory::Social,
                "Assignment room filter cleared",
                "Assign roster now shows survivors from every pinned room.".to_string(),
            );
        }
    }
}
