use super::*;

impl GameplayState {
    pub(super) fn apply_assign_batch_action(&mut self, action: AssignBatchAction) {
        let Some(selected_id) = self.selected_colonist_id else {
            return;
        };
        let target_indices = if action.targets_all() {
            (0..self.data.colonists.len()).collect::<Vec<_>>()
        } else {
            assign_visible_colonist_indices(
                &self.data.colonists,
                self.selected_colonist_id,
                self.assign_roster_page,
                self.assign_roster_filter,
                self.assign_roster_sort,
                self.assign_role_filter,
                self.assign_building_filter,
            )
        };
        let scope = if action.targets_all() {
            BatchAssignmentScope::All
        } else {
            BatchAssignmentScope::Page
        };

        let Some(selected) = self.colonist_by_id(selected_id).cloned() else {
            return;
        };

        let (title, detail) = match action {
            AssignBatchAction::PageHome | AssignBatchAction::AllHome => {
                let Some(habitat_id) = selected.assigned_habitat else {
                    self.log_batch_assignment_unavailable("home", &selected.name);
                    return;
                };
                let capacity = 2 + self.data.technology.habitat_capacity_bonus();
                let assigned = apply_batch_home_pin(
                    &mut self.data.colonists,
                    selected_id,
                    habitat_id,
                    &target_indices,
                    capacity,
                );
                batch_assignment_log(
                    "Batch recovery pins",
                    &selected.name,
                    "H",
                    habitat_id,
                    scope,
                    assigned,
                )
            }
            AssignBatchAction::PageWork | AssignBatchAction::AllWork => {
                let Some(workplace_id) = selected.assigned_workplace else {
                    self.log_batch_assignment_unavailable("work", &selected.name);
                    return;
                };
                let Some(building_type) = self
                    .data
                    .building_system
                    .get_building(workplace_id)
                    .map(|building| building.building_type)
                else {
                    self.log_batch_assignment_unavailable("work", &selected.name);
                    return;
                };
                let assigned = apply_batch_work_pin(
                    &mut self.data.colonists,
                    selected_id,
                    workplace_id,
                    building_type,
                    &target_indices,
                );
                batch_assignment_log(
                    "Batch work pins",
                    &selected.name,
                    "W",
                    workplace_id,
                    scope,
                    assigned,
                )
            }
        };

        self.data.push_log(LogCategory::Social, title, detail);
    }

    pub(super) fn log_batch_assignment_unavailable(&mut self, pin_kind: &str, selected_name: &str) {
        self.data.push_log(
            LogCategory::Social,
            "Batch assignment unavailable",
            format!(
                "{} needs a pinned {} space before that pin can be copied.",
                selected_name, pin_kind
            ),
        );
    }
}
