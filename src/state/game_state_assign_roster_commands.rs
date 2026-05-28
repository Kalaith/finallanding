use super::*;

impl GameplayState {
    pub(super) fn handle_assign_toolbar_click(
        &mut self,
        context: Rect,
        mouse_x: f32,
        mouse_y: f32,
    ) {
        if let Some(filter) = assign_filter_at(context, mouse_x, mouse_y) {
            self.assign_roster_filter = filter;
            self.assign_roster_page = 0;
            return;
        }

        if let Some(sort) = assign_sort_at(context, mouse_x, mouse_y) {
            self.assign_roster_sort = sort;
            self.assign_roster_page = 0;
            return;
        }

        if assign_role_filter_at(context, mouse_x, mouse_y) {
            self.assign_role_filter = next_assign_role_filter(self.assign_role_filter);
            self.assign_roster_page = 0;
            return;
        }

        if let Some(action) = assign_batch_action_at(context, mouse_x, mouse_y) {
            self.apply_assign_batch_action(action);
            return;
        }

        if let Some(action) = assign_page_action_at(context, mouse_x, mouse_y) {
            self.update_assign_roster_page(action);
            return;
        }

        let visible_count = assign_visible_colonist_indices(
            &self.data.colonists,
            self.selected_colonist_id,
            self.assign_roster_page,
            self.assign_roster_filter,
            self.assign_roster_sort,
            self.assign_role_filter,
            self.assign_building_filter,
        )
        .len();
        if let Some(slot) = toolbar_colonist_index_at(context, visible_count, mouse_x, mouse_y) {
            if let Some(index) = self.assign_colonist_index_for_slot(slot) {
                self.update_assign_click(index);
            }
        }
    }

    pub(super) fn update_assign_roster_page(&mut self, action: PageAction) {
        let page_count = assign_roster_page_count(
            &self.data.colonists,
            self.selected_colonist_id,
            self.assign_roster_filter,
            self.assign_role_filter,
            self.assign_building_filter,
        );
        match action {
            PageAction::Previous => {
                self.assign_roster_page = self.assign_roster_page.saturating_sub(1);
            }
            PageAction::Next => {
                if self.assign_roster_page + 1 < page_count {
                    self.assign_roster_page += 1;
                }
            }
        }

        self.assign_roster_page = self.assign_roster_page.min(page_count.saturating_sub(1));
    }

    pub(super) fn assign_colonist_index_for_slot(&self, slot: usize) -> Option<usize> {
        assign_visible_colonist_indices(
            &self.data.colonists,
            self.selected_colonist_id,
            self.assign_roster_page,
            self.assign_roster_filter,
            self.assign_roster_sort,
            self.assign_role_filter,
            self.assign_building_filter,
        )
        .get(slot)
        .copied()
    }

    pub(super) fn update_assign_click(&mut self, colonist_index: usize) {
        let Some(clicked_id) = self
            .data
            .colonists
            .get(colonist_index)
            .map(|colonist| colonist.id)
        else {
            return;
        };

        if let Some(selected_id) = self.selected_colonist_id {
            if selected_id != clicked_id {
                self.toggle_relationship_directive(selected_id, clicked_id);
                return;
            }
        }

        self.cycle_colonist_job(colonist_index);
    }

    pub(super) fn cycle_colonist_job(&mut self, colonist_index: usize) {
        let Some(snapshot) = self.data.colonists.get(colonist_index) else {
            return;
        };

        let colonist_id = snapshot.id;
        let name = snapshot.name.clone();
        let previous = snapshot.job_preference;
        let next = previous.next_assignable();
        let forecast =
            AssignmentSystem::forecast_role_change(&self.data.colonists, colonist_id, next);

        let Some(colonist) = self.data.colonists.get_mut(colonist_index) else {
            return;
        };

        colonist.job_preference = next;
        let cleared_workplace = colonist.assigned_workplace.take();
        if matches!(
            colonist.state,
            ColonistState::Working | ColonistState::Moving { .. }
        ) {
            colonist.state = ColonistState::Idle;
            colonist.activity_location = crate::data::colonist::ActivityLocation::None;
        }
        self.selected_colonist_id = Some(colonist_id);

        self.data.push_log(
            LogCategory::System,
            format!("Role assigned: {}", name),
            format!(
                "{} -> {}. {}{}",
                previous.label(),
                next.label(),
                forecast.detail,
                if cleared_workplace.is_some() {
                    " Work space pin cleared for the new role."
                } else {
                    ""
                }
            ),
        );
    }
}
