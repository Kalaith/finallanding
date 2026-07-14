use super::*;

impl State for GameplayState {
    fn update(&mut self) -> StateTransition {
        let input = InputState::capture();

        // Debug toggle
        self.debug_overlay.record_frame(get_frame_time());
        if is_key_pressed(KeyCode::F3) {
            self.debug_overlay.toggle();
        }
        let keyboard_captured = self.update_social_history_search_input();

        if let Some(transition) = self.scenario_restart_transition(&input) {
            return transition;
        }

        // Time speed and priority controls (keyboard)
        if !keyboard_captured && input.space_pressed {
            self.data.time.speed = if self.data.time.speed == TimeSpeed::Paused {
                TimeSpeed::Normal
            } else {
                TimeSpeed::Paused
            };
        }
        if !keyboard_captured && is_key_pressed(KeyCode::Key1) {
            self.set_priority(ColonyPriority::Recovery);
        }
        if !keyboard_captured && is_key_pressed(KeyCode::Key2) {
            self.set_priority(ColonyPriority::Stockpile);
        }
        if !keyboard_captured && is_key_pressed(KeyCode::Key3) {
            self.set_priority(ColonyPriority::Survey);
        }

        self.update_pointer_ui_input(&input);
        self.update_colonist_selection(&input);

        let elapsed_ticks = self.advance_time();
        if elapsed_ticks > 0 {
            MissionSystem::process_completed_missions(&mut self.data);
            MissionSystem::recover_injured_colonists(&mut self.data);
            self.process_time_events();
            crate::game::colonist_ai::update_colonists(&mut self.data, elapsed_ticks);
            ScenarioSystem::evaluate(&mut self.data);
        } else {
            crate::game::colonist_ai::update_colonists(&mut self.data, 0);
        }

        // Update hovered cell based on mouse position (account for UI offset)
        let mouse = input.mouse_pos;
        let game_area = self.layout.game_area();
        let grid_pos = self.iso_view().screen_to_grid(mouse);
        if game_area.contains(mouse) && self.data.grid.is_in_bounds(grid_pos.x, grid_pos.y) {
            self.hovered_cell = Some(grid_pos);
        } else {
            self.hovered_cell = None;
        }

        // Building system updates (keyboard)
        if !keyboard_captured {
            self.update_building_selection();
        }
        self.update_building_placement(&input);

        StateTransition::None
    }

    fn draw(&self) {
        let hovered_colonist_id = self.colonist_id_at_mouse();

        // Draw game area (grid, buildings, colonists)
        self.draw_grid_with_offset();
        self.draw_buildings();
        self.draw_ghost_preview();
        self.draw_colonists_with_offset(hovered_colonist_id);
        self.draw_hover_colonist_card(hovered_colonist_id);
        let advisor_plan = AdvisorSystem::plan(&self.data);
        let objectives = ObjectiveSystem::active_cards(&self.data);
        draw_advisor_overlay(&self.layout, &objectives, &advisor_plan);
        draw_colonist_inspector(
            &self.layout,
            self.inspected_colonist(hovered_colonist_id),
            &self.data.colonists,
            self.data.tick,
            &self.art,
        );

        // Draw UI components (on top)
        draw_top_bar(
            &self.layout,
            self.data.tick,
            self.data.time.speed,
            self.data.colonists.len(),
            self.average_mood(),
            &self.data.resources,
            self.data.priority.active,
        );

        let colony_summary = SummarySystem::colony_pressure_summary(&self.data);
        let mission_plans = MissionSystem::mission_plans(&self.data);
        draw_right_rail(
            &self.layout,
            &self.data,
            ResourceSystem::storage_capacity(&self.data),
            ResourceSystem::daily_supply_need(&self.data),
            &colony_summary,
            &self.art,
        );
        draw_toolbar_context_panel(
            &self.layout,
            ToolbarPanelData {
                mode: self.toolbar_mode,
                selected_building: self.selected_building,
                resources: &self.data.resources,
                active_priority: self.data.priority.active,
                research: ToolbarResearchData {
                    mission_plans: &mission_plans,
                    technology: &self.data.technology,
                    active_mission_count: self.data.missions.active_count(),
                    required_unlocks: self.data.scenario.required_tech_unlocks,
                },
                assign: ToolbarAssignData {
                    colonists: &self.data.colonists,
                    selected_colonist_id: self.selected_colonist_id,
                    roster_page: self.assign_roster_page,
                    roster_filter: self.assign_roster_filter,
                    roster_sort: self.assign_roster_sort,
                    role_filter: self.assign_role_filter,
                    building_filter: self.assign_building_filter,
                    technology: &self.data.technology,
                },
                log: ToolbarLogData {
                    logs: &self.data.event_log,
                    social_history: &self.data.social_history,
                    page: self.social_history_page,
                    filter: self.social_history_filter,
                    query: &self.social_history_query,
                    search_active: self.social_history_search_active,
                    selected_day: self.selected_social_history_day,
                    colony_summary: &colony_summary,
                },
            },
        );
        draw_bottom_toolbar(&self.layout, self.toolbar_mode, self.selected_building);

        // Debug overlay
        if self.debug_overlay.visible {
            draw_debug_overlay(
                &self.debug_overlay,
                self.data.tick,
                &self.data.colonists,
                self.hovered_cell,
                self.data.building_system.building_count(),
                &self.data.resources,
                ResourceSystem::storage_capacity(&self.data),
                ResourceSystem::daily_supply_need(&self.data),
                &ScenarioSystem::objective_line(&self.data),
                self.data.scenario.outcome,
                self.data.missions.active_count(),
                &self.data.technology,
                self.data.priority.active,
            );
        }

        self.draw_scenario_overlay();
    }
}
