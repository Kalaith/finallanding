use super::*;

impl GameplayState {
    /// Handle building selection UI (keyboard)
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

    pub(super) fn toggle_building(&mut self, building_type: BuildingType) {
        if self.selected_building == Some(building_type) {
            self.selected_building = None;
        } else {
            self.selected_building = Some(building_type);
        }
    }

    pub(super) fn undo_last_building(&mut self) {
        let refund = self
            .data
            .building_system
            .last_placed_building()
            .map(|building| {
                (
                    building.id,
                    building.building_type,
                    building.building_type.salvage_cost(),
                )
            });

        if let Some(building_id) = self
            .data
            .building_system
            .undo_last_placement(&mut self.data.grid)
        {
            let cleared_assignments = self.clear_building_assignments(building_id);
            let assignment_note = if cleared_assignments.is_empty() {
                String::new()
            } else {
                format!(
                    " Cleared room pins for {}.",
                    truncate_text(&cleared_assignments.join(", "), 46)
                )
            };

            if let Some((refund_id, building_type, salvage_cost)) = refund {
                if refund_id == building_id {
                    self.data.resources.refund_salvage(salvage_cost);
                    self.data.push_log(
                        LogCategory::System,
                        "Building plan undone",
                        format!(
                            "Removed {} #{} and refunded {} salvage.{}",
                            building_type.name(),
                            building_id,
                            salvage_cost,
                            assignment_note
                        ),
                    );
                    return;
                }
            }

            self.data.push_log(
                LogCategory::System,
                "Building plan undone",
                format!(
                    "Removed building #{} from the settlement plan.{}",
                    building_id, assignment_note
                ),
            );
        }
    }

    pub(super) fn launch_recommended_mission(&mut self) {
        let mission_type = MissionSystem::recommended_mission_type(&self.data);
        self.launch_mission(mission_type);
    }

    pub(super) fn launch_mission(&mut self, mission_type: MissionType) {
        if let Err(error) = MissionSystem::launch_mission(&mut self.data, mission_type) {
            let definition = mission_type.definition();
            let (title, detail) = match error {
                crate::systems::mission_system::LaunchMissionError::NoExplorationGate => (
                    "No Exploration Gate",
                    format!(
                        "Build an Exploration Gate before sending {}.",
                        definition.name
                    ),
                ),
                crate::systems::mission_system::LaunchMissionError::NoAvailableColonist => (
                    "No available mission crew",
                    format!(
                        "{} needs a colonist who is not away or hurt.",
                        definition.name
                    ),
                ),
                crate::systems::mission_system::LaunchMissionError::MissionCooldown {
                    remaining_ticks,
                } => (
                    "Mission crew regrouping",
                    format!(
                        "Wait {} more minutes before launching another mission.",
                        remaining_ticks
                    ),
                ),
            };

            self.data.push_log(LogCategory::Mission, title, detail);
        }
    }

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

    /// Handle building placement via mouse click
    pub(super) fn update_building_placement(&mut self, input: &InputState) {
        // Only allow placement in the game area (not over UI)
        let mouse_pos = input.mouse_pos;
        let mouse_x = mouse_pos.x;
        let mouse_y = mouse_pos.y;
        let game_area = self.layout.game_area();
        let toolbar = self.layout.bottom_toolbar();

        if input.hovered_rect(toolbar)
            || input.hovered_rect(toolbar_context_rect(toolbar))
            || input.hovered_rect(self.layout.left_panel())
            || input.hovered_rect(self.layout.right_panel())
            || mouse_y <= self.layout.top_bar_height
        {
            return;
        }

        if mouse_x < game_area.x || mouse_x > game_area.x + game_area.w {
            return;
        }
        if mouse_y < game_area.y || mouse_y > game_area.y + game_area.h {
            return;
        }

        let Some(building_type) = self.selected_building else {
            return;
        };

        if input.left_pressed {
            let pos = self.iso_view().screen_to_grid(vec2(mouse_x, mouse_y));
            let feedback = PlanningSystem::building_feedback(&self.data, building_type, pos);
            if let Some(reason) = feedback.invalid_reason.as_ref() {
                self.data.push_log(
                    LogCategory::System,
                    format!("Cannot place {}", building_type.name()),
                    format!(
                        "{} {} helps {}: {}",
                        reason,
                        building_type.name(),
                        feedback.helps,
                        feedback.purpose
                    ),
                );
                return;
            }

            let result = self.data.building_system.try_place_building(
                &mut self.data.grid,
                building_type,
                pos,
            );
            let result_reason = placement_result_reason(&result);

            match result {
                PlacementResult::Success(building_id) => {
                    self.data
                        .resources
                        .spend_salvage(building_type.salvage_cost());
                    self.data.push_log(
                        LogCategory::System,
                        format!("{} placed", building_type.name()),
                        PlanningSystem::placement_log_detail(
                            &feedback,
                            building_id,
                            self.data.resources.salvage,
                        ),
                    );
                }
                PlacementResult::OutOfBounds
                | PlacementResult::AreaOccupied
                | PlacementResult::InvalidBuilding => {
                    self.data.push_log(
                        LogCategory::System,
                        format!("Cannot place {}", building_type.name()),
                        result_reason.to_string(),
                    );
                }
            }
        }
    }

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
                if let Some(building_type) =
                    toolbar_building_at_for_mode(context, self.toolbar_mode, mouse_x, mouse_y)
                {
                    self.toggle_building(building_type);
                }
            }
            ToolbarMode::Colony => {
                if let Some(priority) = toolbar_priority_at(context, mouse_x, mouse_y) {
                    self.set_priority(priority);
                }
            }
            ToolbarMode::Research => {
                if let Some(mission_type) = toolbar_mission_at(context, mouse_x, mouse_y) {
                    self.launch_mission(mission_type);
                }
            }
            ToolbarMode::Assign => {
                if let Some(filter) = assign_filter_at(context, mouse_x, mouse_y) {
                    self.assign_roster_filter = filter;
                    self.assign_roster_page = 0;
                    return true;
                }

                if let Some(sort) = assign_sort_at(context, mouse_x, mouse_y) {
                    self.assign_roster_sort = sort;
                    self.assign_roster_page = 0;
                    return true;
                }

                if assign_role_filter_at(context, mouse_x, mouse_y) {
                    self.assign_role_filter = next_assign_role_filter(self.assign_role_filter);
                    self.assign_roster_page = 0;
                    return true;
                }

                if let Some(action) = assign_batch_action_at(context, mouse_x, mouse_y) {
                    self.apply_assign_batch_action(action);
                    return true;
                }

                if let Some(action) = assign_page_action_at(context, mouse_x, mouse_y) {
                    self.update_assign_roster_page(action);
                    return true;
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
                if let Some(slot) =
                    toolbar_colonist_index_at(context, visible_count, mouse_x, mouse_y)
                {
                    if let Some(index) = self.assign_colonist_index_for_slot(slot) {
                        self.update_assign_click(index);
                    }
                }
            }
            ToolbarMode::Log => {
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
                    return true;
                }

                self.social_history_search_active = false;

                if let Some(filter) = log_filter_at(context, mouse_x, mouse_y) {
                    self.social_history_filter = filter;
                    self.social_history_page = 0;
                    self.selected_social_history_day = None;
                    return true;
                }
                if let Some(action) = log_page_action_at(context, mouse_x, mouse_y) {
                    self.update_log_page(action);
                    self.selected_social_history_day = None;
                    return true;
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
        }

        true
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

    pub(super) fn toggle_relationship_directive(&mut self, first_id: u32, second_id: u32) {
        let first_name = self
            .colonist_by_id(first_id)
            .map(|colonist| colonist.name.clone())
            .unwrap_or_else(|| format!("Colonist {}", first_id));
        let second_name = self
            .colonist_by_id(second_id)
            .map(|colonist| colonist.name.clone())
            .unwrap_or_else(|| format!("Colonist {}", second_id));

        let change = RelationshipDirectiveSystem::toggle_pair_directive(
            &mut self.data.colonists,
            first_id,
            second_id,
        );

        match change {
            Ok(DirectiveChange::Set(directive)) => {
                self.data.push_log(
                    LogCategory::Social,
                    directive.log_title(),
                    directive_log_detail(directive, &first_name, &second_name),
                );
            }
            Ok(DirectiveChange::Cleared(directive)) => {
                self.data.push_log(
                    LogCategory::Social,
                    "Relationship directive cleared",
                    format!(
                        "{} and {} no longer have a forced {} directive.",
                        first_name,
                        second_name,
                        directive.label().to_lowercase()
                    ),
                );
            }
            Err(_) => {
                self.data.push_log(
                    LogCategory::Social,
                    "Directive failed",
                    format!(
                        "Could not update a directive between {} and {}.",
                        first_name, second_name
                    ),
                );
            }
        }
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

    pub(super) fn update_assign_space_click(&mut self) {
        let Some(building) = self.building_at_mouse().cloned() else {
            return;
        };

        let Some(colonist_id) = self.selected_colonist_id else {
            return;
        };

        self.assign_selected_colonist_to_building(colonist_id, &building);
    }

    pub(super) fn assign_selected_colonist_to_building(
        &mut self,
        colonist_id: u32,
        building: &Building,
    ) {
        let Some(colonist_index) = self
            .data
            .colonists
            .iter()
            .position(|colonist| colonist.id == colonist_id)
        else {
            return;
        };

        let name = self.data.colonists[colonist_index].name.clone();
        let job = self.data.colonists[colonist_index].job_preference;
        let Some(kind) = space_assignment_kind(job, building.building_type) else {
            self.data.push_log(
                LogCategory::Social,
                "Room assignment blocked",
                format!(
                    "{} cannot pin {} #{} while assigned {}. Retask first or choose a compatible space.",
                    name,
                    building.building_type.name(),
                    building.id,
                    job.label()
                ),
            );
            return;
        };

        let colonist = &mut self.data.colonists[colonist_index];
        let (title, detail) = match kind {
            SpaceAssignmentKind::Recovery => {
                if colonist.assigned_habitat == Some(building.id) {
                    colonist.assigned_habitat = None;
                    (
                        "Recovery room pin cleared".to_string(),
                        format!("{} can choose any available Habitat again.", name),
                    )
                } else {
                    colonist.assigned_habitat = Some(building.id);
                    (
                        "Recovery room pinned".to_string(),
                        format!(
                            "{} will prefer Habitat #{} for sleep and recovery.",
                            name, building.id
                        ),
                    )
                }
            }
            SpaceAssignmentKind::Work => {
                if colonist.assigned_workplace == Some(building.id) {
                    colonist.assigned_workplace = None;
                    (
                        "Work space pin cleared".to_string(),
                        format!(
                            "{} can choose any compatible {} space again.",
                            name,
                            job.label()
                        ),
                    )
                } else {
                    colonist.assigned_workplace = Some(building.id);
                    if matches!(
                        colonist.state,
                        ColonistState::Working | ColonistState::Moving { .. }
                    ) {
                        colonist.state = ColonistState::Idle;
                        colonist.activity_location = ActivityLocation::None;
                    }
                    (
                        "Work space pinned".to_string(),
                        format!(
                            "{} will prefer {} #{} while assigned {}.",
                            name,
                            building.building_type.name(),
                            building.id,
                            job.label()
                        ),
                    )
                }
            }
        };

        self.data.push_log(LogCategory::Social, title, detail);
    }

    pub(super) fn clear_building_assignments(&mut self, building_id: u32) -> Vec<String> {
        let mut cleared = Vec::new();
        if self.assign_building_filter == Some(building_id) {
            self.assign_building_filter = None;
        }
        for colonist in &mut self.data.colonists {
            let mut changed = false;
            if colonist.assigned_habitat == Some(building_id) {
                colonist.assigned_habitat = None;
                changed = true;
            }
            if colonist.assigned_workplace == Some(building_id) {
                colonist.assigned_workplace = None;
                changed = true;
            }
            if changed {
                cleared.push(colonist.name.clone());
            }
        }

        cleared
    }

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
