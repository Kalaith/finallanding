use super::*;

impl GameplayState {
    pub(super) fn update_social_history_search_input(&mut self) -> bool {
        if self.toolbar_mode != ToolbarMode::Log {
            self.social_history_search_active = false;
            return false;
        }
        if !self.social_history_search_active {
            return false;
        }

        let mut changed = false;
        while let Some(character) = get_char_pressed() {
            if character.is_ascii()
                && !character.is_control()
                && self.social_history_query.chars().count() < 28
            {
                self.social_history_query.push(character);
                changed = true;
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            changed |= self.social_history_query.pop().is_some();
        }
        if is_key_pressed(KeyCode::Delete) && !self.social_history_query.is_empty() {
            self.social_history_query.clear();
            changed = true;
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            self.social_history_search_active = false;
        }

        if changed {
            self.social_history_page = 0;
            self.selected_social_history_day = None;
        }

        true
    }

    pub(super) fn handle_log_toolbar_click(&mut self, context: Rect, mouse_x: f32, mouse_y: f32) {
        if let Some(action) = log_search_action_at(context, mouse_x, mouse_y) {
            match action {
                LogSearchAction::Focus => {
                    self.social_history_search_active = true;
                }
                LogSearchAction::Clear => {
                    self.social_history_query.clear();
                    self.social_history_search_active = false;
                    self.social_history_page = 0;
                    self.selected_social_history_day = None;
                }
                LogSearchAction::Export => {
                    self.social_history_search_active = false;
                    self.export_social_archive();
                }
            }
            return;
        }

        self.social_history_search_active = false;

        if let Some(filter) = log_filter_at(context, mouse_x, mouse_y) {
            self.social_history_filter = filter;
            self.social_history_page = 0;
            self.selected_social_history_day = None;
            return;
        }
        if let Some(action) = log_page_action_at(context, mouse_x, mouse_y) {
            self.update_log_page(action);
            self.selected_social_history_day = None;
            return;
        }
        if let Some(row) = log_timeline_row_at(context, 3, mouse_x, mouse_y) {
            if let Some(day) = social_timeline_day_at(
                &self.data.social_history,
                self.social_history_filter,
                &self.social_history_query,
                self.social_history_page,
                row,
            ) {
                self.selected_social_history_day =
                    (self.selected_social_history_day != Some(day)).then_some(day);
            }
        }
    }

    pub(super) fn update_log_page(&mut self, action: PageAction) {
        let page_count = social_history_page_count(
            &self.data.social_history,
            self.social_history_filter,
            &self.social_history_query,
        );
        match action {
            PageAction::Previous => {
                self.social_history_page = self.social_history_page.saturating_sub(1);
            }
            PageAction::Next => {
                if self.social_history_page + 1 < page_count {
                    self.social_history_page += 1;
                }
            }
        }

        self.social_history_page = self.social_history_page.min(page_count.saturating_sub(1));
    }

    pub(super) fn export_social_archive(&mut self) {
        if self.data.social_history.is_empty() {
            self.data.push_log(
                LogCategory::Social,
                "Social archive export skipped",
                "No daily social reports have been recorded yet.".to_string(),
            );
            return;
        }

        match write_social_archive_markdown(&self.data.social_history) {
            Ok(path) => self.data.push_log(
                LogCategory::Social,
                "Social archive exported",
                format!(
                    "Wrote {} daily relationship reports to {}.",
                    self.data.social_history.len(),
                    path.display()
                ),
            ),
            Err(error) => {
                self.data
                    .push_log(LogCategory::System, "Social archive export failed", error)
            }
        }
    }
}
