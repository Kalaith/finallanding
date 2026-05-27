use crate::data::building::{Building, BuildingType};
use crate::data::colonist::{ActivityLocation, Colonist, ColonistState, JobPreference};
use crate::data::event_log::{LogCategory, SocialHistoryEntry};
use crate::data::game_state::GameState;
use crate::data::game_state::TimeSpeed;
use crate::data::grid::CellType;
use crate::data::mission::MissionType;
use crate::data::priority::ColonyPriority;
use crate::data::types::Position;
use crate::game::building_system::PlacementResult;
use crate::state::{State, StateTransition};
use crate::systems::advisor_system::AdvisorSystem;
use crate::systems::assignment_system::AssignmentSystem;
use crate::systems::incident_system::IncidentSystem;
use crate::systems::mission_system::MissionSystem;
use crate::systems::objective_system::ObjectiveSystem;
use crate::systems::planning_system::{BuildingPlacementFeedback, PlanningSystem};
use crate::systems::proximity_system::ProximitySystem;
use crate::systems::relationship_directive_system::{
    DirectiveChange, PairDirective, RelationshipDirectiveSystem,
};
use crate::systems::resource_system::ResourceSystem;
use crate::systems::scenario_system::ScenarioSystem;
use crate::systems::social_system::SocialSystem;
use crate::systems::summary_system::SummarySystem;
use crate::systems::time_events::TimeEventCollector;
use crate::systems::time_system::TimeSystem;
use crate::systems::work_system::WorkSystem;
use crate::ui::font::{draw_text, measure_text};
use crate::ui::style;
use crate::ui::{
    assign_batch_action_at, assign_filter_at, assign_page_action_at, assign_role_filter_at,
    assign_sort_at, draw_advisor_overlay, draw_bottom_toolbar, draw_colonist_inspector,
    draw_debug_overlay, draw_iso_diamond, draw_iso_diamond_lines, draw_iso_prism, draw_right_rail,
    draw_toolbar_context_panel, draw_tooltip_at, draw_top_bar, log_filter_at, log_page_action_at,
    log_search_action_at, log_timeline_row_at, restart_button_rect, social_history_page_count,
    social_timeline_day_at, toolbar_building_at_for_mode, toolbar_buildings_for_mode,
    toolbar_colonist_index_at, toolbar_context_rect, toolbar_mission_at, toolbar_mode_at,
    toolbar_priority_at, top_bar_priority_at, top_bar_speed_at, AssignBatchAction,
    AssignRosterFilter, AssignRosterSort, IsoView, Layout, LogFilter, LogSearchAction, PageAction,
    PlaceholderArt, SpritePose, ToolbarMode,
};
use macroquad::prelude::*;
use std::path::PathBuf;

const SECONDS_PER_GAME_TICK: f32 = 0.25;

pub struct GameplayState {
    pub data: GameState,
    hovered_cell: Option<Position>,
    /// Currently selected building type for placement (None = not in build mode)
    selected_building: Option<BuildingType>,
    /// Fixed preview grid position used only by screenshot verification captures.
    capture_preview_position: Option<Position>,
    /// Selected colonist for relationship inspection.
    selected_colonist_id: Option<u32>,
    /// Time event collector for processing time-based events
    time_events: TimeEventCollector,
    /// Previous tick for event detection
    prev_tick: u64,
    /// Accumulates real time before advancing the simulation by game ticks
    time_accumulator: f32,
    /// UI layout configuration
    layout: Layout,
    /// Debug overlay visible
    debug_mode: bool,
    /// Active bottom-toolbar mode.
    toolbar_mode: ToolbarMode,
    /// Current page in the Assign mode roster.
    assign_roster_page: usize,
    /// Active filter in the Assign mode roster.
    assign_roster_filter: AssignRosterFilter,
    /// Active sort in the Assign mode roster.
    assign_roster_sort: AssignRosterSort,
    /// Optional work-role filter in the Assign mode roster.
    assign_role_filter: Option<JobPreference>,
    /// Optional room/work-space instance filter in the Assign mode roster.
    assign_building_filter: Option<u32>,
    /// Current page in the Log mode social archive.
    social_history_page: usize,
    /// Active filter in the Log mode social archive.
    social_history_filter: LogFilter,
    /// Search query for the Log mode social archive.
    social_history_query: String,
    /// Whether typed keys should edit the Log mode social archive search.
    social_history_search_active: bool,
    /// Selected daily social report for persistent Log drilldown.
    selected_social_history_day: Option<u32>,
    /// Placeholder visual assets extracted from the rebuild reference.
    art: PlaceholderArt,
}

impl GameplayState {
    pub fn new() -> Self {
        let mut data = GameState::new();
        data.tick = 420; // Start at 07:00 AM (Work time)
        crate::game::colonist_spawner::spawn_initial_colonists(&mut data);
        data.push_log(
            LogCategory::System,
            "Crash survivors assembled",
            format!(
                "Starting stockpile: {} supplies, {} salvage. Objective: survive to Day {} and unlock {} technologies.",
                data.resources.supplies,
                data.resources.salvage,
                data.scenario.target_day,
                data.scenario.required_tech_unlocks
            ),
        );
        seed_assign_spaces_for_capture(&mut data);
        seed_activity_poses_for_capture(&mut data);
        seed_social_history_for_capture(&mut data);

        let toolbar_mode = initial_toolbar_mode();
        let selected_building = initial_selected_building(toolbar_mode);
        let selected_colonist_id = initial_selected_colonist_id(&data, toolbar_mode);
        let capture_preview_position = initial_capture_preview_position();
        let selected_social_history_day = initial_selected_social_history_day(&data);

        Self {
            prev_tick: data.tick,
            data,
            hovered_cell: None,
            selected_building,
            capture_preview_position,
            selected_colonist_id,
            time_events: TimeEventCollector::new(),
            time_accumulator: 0.0,
            layout: Layout::default(),
            debug_mode: false,
            toolbar_mode,
            assign_roster_page: 0,
            assign_roster_filter: AssignRosterFilter::All,
            assign_roster_sort: AssignRosterSort::Roster,
            assign_role_filter: None,
            assign_building_filter: None,
            social_history_page: 0,
            social_history_filter: LogFilter::All,
            social_history_query: String::new(),
            social_history_search_active: false,
            selected_social_history_day,
            art: PlaceholderArt::new(),
        }
    }

    /// Handle building selection UI (keyboard)
    fn update_building_selection(&mut self) {
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

    fn update_social_history_search_input(&mut self) -> bool {
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

    fn toggle_building(&mut self, building_type: BuildingType) {
        if self.selected_building == Some(building_type) {
            self.selected_building = None;
        } else {
            self.selected_building = Some(building_type);
        }
    }

    fn undo_last_building(&mut self) {
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

    fn launch_recommended_mission(&mut self) {
        let mission_type = MissionSystem::recommended_mission_type(&self.data);
        self.launch_mission(mission_type);
    }

    fn launch_mission(&mut self, mission_type: MissionType) {
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

    fn set_priority(&mut self, priority: ColonyPriority) {
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
    fn update_building_placement(&mut self) {
        // Only allow placement in the game area (not over UI)
        let (mouse_x, mouse_y) = mouse_position();
        let game_area = self.layout.game_area();
        let mouse_pos = vec2(mouse_x, mouse_y);
        let toolbar = self.layout.bottom_toolbar();

        if toolbar.contains(mouse_pos)
            || toolbar_context_rect(toolbar).contains(mouse_pos)
            || self.layout.left_panel().contains(mouse_pos)
            || self.layout.right_panel().contains(mouse_pos)
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

        if is_mouse_button_pressed(MouseButton::Left) {
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

    fn update_pointer_ui_input(&mut self) {
        let assign_room_filter_click =
            self.toolbar_mode == ToolbarMode::Assign && is_mouse_button_pressed(MouseButton::Right);
        if !is_mouse_button_pressed(MouseButton::Left) && !assign_room_filter_click {
            return;
        }

        let (mouse_x, mouse_y) = mouse_position();

        if assign_room_filter_click {
            if self.layout.game_area().contains(vec2(mouse_x, mouse_y)) {
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

    fn update_assign_building_filter_click(&mut self) {
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

    fn update_toolbar_click(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
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

    fn update_assign_roster_page(&mut self, action: PageAction) {
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

    fn update_log_page(&mut self, action: PageAction) {
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

    fn export_social_archive(&mut self) {
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

    fn apply_assign_batch_action(&mut self, action: AssignBatchAction) {
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

    fn log_batch_assignment_unavailable(&mut self, pin_kind: &str, selected_name: &str) {
        self.data.push_log(
            LogCategory::Social,
            "Batch assignment unavailable",
            format!(
                "{} needs a pinned {} space before that pin can be copied.",
                selected_name, pin_kind
            ),
        );
    }

    fn assign_colonist_index_for_slot(&self, slot: usize) -> Option<usize> {
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

    fn update_assign_click(&mut self, colonist_index: usize) {
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

    fn toggle_relationship_directive(&mut self, first_id: u32, second_id: u32) {
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

    fn cycle_colonist_job(&mut self, colonist_index: usize) {
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

    fn update_assign_space_click(&mut self) {
        let Some(building) = self.building_at_mouse().cloned() else {
            return;
        };

        let Some(colonist_id) = self.selected_colonist_id else {
            return;
        };

        self.assign_selected_colonist_to_building(colonist_id, &building);
    }

    fn assign_selected_colonist_to_building(&mut self, colonist_id: u32, building: &Building) {
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

    fn clear_building_assignments(&mut self, building_id: u32) -> Vec<String> {
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

    fn update_colonist_selection(&mut self) {
        if self.selected_building.is_some() || !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }

        let game_area = self.layout.game_area();
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_pos = vec2(mouse_x, mouse_y);
        let toolbar = self.layout.bottom_toolbar();
        if toolbar.contains(mouse_pos)
            || toolbar_context_rect(toolbar).contains(mouse_pos)
            || self.layout.left_panel().contains(mouse_pos)
            || self.layout.right_panel().contains(mouse_pos)
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

    fn update_top_bar_click(&mut self, mouse_x: f32, mouse_y: f32) {
        if let Some(speed) = top_bar_speed_at(mouse_x, mouse_y) {
            self.data.time.speed = speed;
            return;
        }

        if let Some(priority) = top_bar_priority_at(mouse_x, mouse_y) {
            self.set_priority(priority);
        }
    }

    fn iso_view(&self) -> IsoView {
        IsoView::for_area(
            self.layout.game_area(),
            self.data.grid.width as u32,
            self.data.grid.height as u32,
        )
    }

    fn scenario_restart_transition(&self) -> Option<StateTransition> {
        if !self.data.scenario.is_finished() {
            return None;
        }

        let restart_rect = restart_button_rect(screen_width(), screen_height());
        let mouse_pos: Vec2 = mouse_position().into();
        let clicked_restart =
            is_mouse_button_pressed(MouseButton::Left) && restart_rect.contains(mouse_pos);

        if clicked_restart || is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Enter) {
            Some(StateTransition::ToGameplay(GameplayState::new()))
        } else {
            None
        }
    }

    fn advance_time(&mut self) -> u64 {
        let speed_multiplier = match self.data.time.speed {
            TimeSpeed::Paused => 0.0,
            TimeSpeed::Normal => 1.0,
            TimeSpeed::Fast => 2.0,
            TimeSpeed::SuperFast => 4.0,
        };

        if speed_multiplier == 0.0 {
            self.time_accumulator = 0.0;
            return 0;
        }

        self.time_accumulator += get_frame_time() * speed_multiplier;
        let ticks_to_advance = (self.time_accumulator / SECONDS_PER_GAME_TICK).floor() as u64;

        if ticks_to_advance == 0 {
            return 0;
        }

        self.time_accumulator -= ticks_to_advance as f32 * SECONDS_PER_GAME_TICK;
        self.prev_tick = self.data.tick;
        self.data.tick += ticks_to_advance;

        self.time_events.clear();
        TimeSystem::collect_events(self.prev_tick, self.data.tick, &mut self.time_events);
        ticks_to_advance
    }

    fn process_time_events(&mut self) {
        let events = self.time_events.events.clone();

        for event in events {
            match event {
                crate::systems::time_events::TimeEvent::NewDay { day } => {
                    ProximitySystem::check_sleeping_proximity(&mut self.data);
                    SummarySystem::summarize_previous_day(&mut self.data, day);
                    ResourceSystem::handle_new_day(&mut self.data);
                }
                crate::systems::time_events::TimeEvent::DawnBreak => {
                    self.data.push_log(
                        LogCategory::Time,
                        "Dawn breaks",
                        "Colonists begin shifting toward the day's work.",
                    );
                }
                crate::systems::time_events::TimeEvent::Dusk => {
                    self.data.push_log(
                        LogCategory::Time,
                        "Dusk falls",
                        "The settlement starts moving toward meals and recovery.",
                    );
                }
                crate::systems::time_events::TimeEvent::HourChanged { hour: _ } => {
                    IncidentSystem::process_hourly_incidents(&mut self.data);
                    WorkSystem::process_hourly_work(&mut self.data);
                    SocialSystem::check_working_together(&mut self.data);
                    SocialSystem::check_eating_together(&mut self.data);
                }
            }
        }
    }

    fn average_mood(&self) -> f32 {
        if self.data.colonists.is_empty() {
            return 0.0;
        }

        self.data.colonists.iter().map(|c| c.mood).sum::<f32>() / self.data.colonists.len() as f32
    }

    fn draw_scenario_overlay(&self) {
        if !self.data.scenario.is_finished() {
            return;
        }

        let w = 520.0;
        let h = 190.0;
        let x = (screen_width() - w) * 0.5;
        let y = (screen_height() - h) * 0.5;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.55),
        );
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.08, 0.1, 0.95));
        draw_rectangle_lines(x, y, w, h, 2.0, WHITE);

        let title = self.data.scenario.outcome.label();
        let title_width = measure_text(title, None, 28, 1.0).width;
        draw_text(title, x + (w - title_width) * 0.5, y + 42.0, 28.0, WHITE);

        let line = ScenarioSystem::objective_line(&self.data);
        let line_width = measure_text(&line, None, 16, 1.0).width;
        draw_text(&line, x + (w - line_width) * 0.5, y + 82.0, 16.0, LIGHTGRAY);

        let prompt = "Scenario complete. Review the log, then restart for another plan.";
        let prompt_width = measure_text(prompt, None, 14, 1.0).width;
        draw_text(prompt, x + (w - prompt_width) * 0.5, y + 116.0, 14.0, GRAY);

        let button = restart_button_rect(screen_width(), screen_height());
        let mouse_pos: Vec2 = mouse_position().into();
        let button_color = if button.contains(mouse_pos) {
            Color::new(0.25, 0.38, 0.48, 1.0)
        } else {
            Color::new(0.16, 0.22, 0.28, 1.0)
        };
        draw_rectangle(button.x, button.y, button.w, button.h, button_color);
        draw_rectangle_lines(button.x, button.y, button.w, button.h, 1.0, WHITE);
        let button_text = "Restart Run";
        let button_width = measure_text(button_text, None, 18, 1.0).width;
        draw_text(
            button_text,
            button.x + (button.w - button_width) * 0.5,
            button.y + 25.0,
            18.0,
            WHITE,
        );
        let restart_hint = "R or Enter";
        let hint_width = measure_text(restart_hint, None, 12, 1.0).width;
        draw_text(
            restart_hint,
            x + (w - hint_width) * 0.5,
            y + 170.0,
            12.0,
            LIGHTGRAY,
        );
    }

    /// Draw buildings on the grid
    fn draw_buildings(&self) {
        let iso = self.iso_view();
        let hovered_building_id = self.building_at_mouse().map(|building| building.id);
        for building in self.data.building_system.buildings() {
            let (width, height) = building.size();
            let (r, g, b) = building.building_type.color();
            let color = Color::new(
                r as f32 / 255.0 * 0.72,
                g as f32 / 255.0 * 0.72,
                b as f32 / 255.0 * 0.72,
                1.0,
            );

            for cell in building.occupied_cells() {
                let center = iso.grid_to_screen(cell);
                draw_iso_diamond(center, iso.tile_w, iso.tile_h, color);
                draw_iso_diamond_lines(
                    center,
                    iso.tile_w,
                    iso.tile_h,
                    1.0,
                    Color::new(0.82, 0.82, 0.76, 0.55),
                );
            }

            let filter_match = self.toolbar_mode == ToolbarMode::Assign
                && self.assign_building_filter == Some(building.id);
            let assignment_marker = assignment_marker_with_filter(
                self.assignment_marker_for_building(building.id),
                filter_match,
            );
            let outline_style = building_outline_style_for_assign_filter(
                hovered_building_id == Some(building.id),
                assignment_marker.map(|(_, color)| color),
                filter_match,
            );
            self.draw_building_shell(
                building.building_type,
                building.position,
                width,
                height,
                &iso,
                outline_style,
            );
            if let Some((outline_color, thickness)) = outline_style {
                self.draw_building_footprint_outline(building, &iso, outline_color, thickness);
            }

            let name = building.building_type.name();
            let label_pos = iso.grid_to_screen(Position::new(
                building.position.x + width as i32 / 2,
                building.position.y + height as i32 / 2,
            ));
            if let Some((assignment_label, assignment_color)) = assignment_marker {
                let marker_width = measure_text(assignment_label, None, 10, 1.0).width + 10.0;
                draw_rectangle(
                    label_pos.x - marker_width * 0.5,
                    label_pos.y - 29.0,
                    marker_width,
                    14.0,
                    Color::new(0.03, 0.04, 0.04, 0.82),
                );
                draw_rectangle_lines(
                    label_pos.x - marker_width * 0.5,
                    label_pos.y - 29.0,
                    marker_width,
                    14.0,
                    1.0,
                    assignment_color,
                );
                draw_text(
                    assignment_label,
                    label_pos.x - marker_width * 0.5 + 5.0,
                    label_pos.y - 18.0,
                    10.0,
                    assignment_color,
                );
            }
            let label_width = measure_text(name, None, 12, 1.0).width;
            draw_text(
                name,
                label_pos.x - label_width * 0.5,
                label_pos.y - 8.0,
                12.0,
                WHITE,
            );
        }
    }

    fn draw_building_footprint_outline(
        &self,
        building: &Building,
        iso: &IsoView,
        color: Color,
        thickness: f32,
    ) {
        for cell in building.occupied_cells() {
            let center = iso.grid_to_screen(cell);
            draw_iso_diamond_lines(center, iso.tile_w, iso.tile_h, thickness, color);
        }
    }

    fn assignment_marker_for_building(&self, building_id: u32) -> Option<(&'static str, Color)> {
        if self.toolbar_mode != ToolbarMode::Assign {
            return None;
        }

        let colonist = self
            .selected_colonist_id
            .and_then(|id| self.colonist_by_id(id))?;

        if colonist.assigned_habitat == Some(building_id) {
            Some(("HOME", style::BAR_GREEN))
        } else if colonist.assigned_workplace == Some(building_id) {
            Some(("WORK", style::HEADING_BLUE))
        } else {
            None
        }
    }

    fn draw_building_shell(
        &self,
        building_type: BuildingType,
        position: Position,
        width: u32,
        height: u32,
        iso: &IsoView,
        outline_style: Option<(Color, f32)>,
    ) {
        let center = iso.grid_to_screen(Position::new(
            position.x + width as i32 / 2,
            position.y + height as i32 / 2,
        ));
        let shell_width = iso.tile_w * width as f32 * 0.86;
        let shell_height = iso.tile_h * height as f32 * 0.86;
        let wall_height = building_wall_height(building_type, iso.tile_h);
        let roof_center = center - vec2(0.0, wall_height + iso.tile_h * 0.12);
        let (roof, front, side) = building_shell_colors(building_type);

        draw_iso_prism(
            roof_center,
            shell_width,
            shell_height,
            wall_height,
            roof,
            front,
            side,
        );
        draw_building_shell_detail(building_type, roof_center, shell_width, shell_height);
        if let Some((outline_color, thickness)) = outline_style {
            draw_iso_diamond_lines(
                roof_center,
                shell_width + 4.0,
                shell_height + 4.0,
                thickness,
                outline_color,
            );
        }
    }

    /// Draw ghost preview of building at cursor
    fn draw_ghost_preview(&self) {
        if let Some(building_type) = self.selected_building {
            let (mouse_x, mouse_y) = mouse_position();
            let game_area = self.layout.game_area();
            let iso = self.iso_view();
            let pos = if let Some(position) = self.capture_preview_position {
                position
            } else {
                if mouse_x < game_area.x
                    || mouse_x > game_area.x + game_area.w
                    || mouse_y < game_area.y
                    || mouse_y > game_area.y + game_area.h
                {
                    return;
                }

                iso.screen_to_grid(vec2(mouse_x, mouse_y))
            };
            let (width, height) = building_type.size();
            let feedback = PlanningSystem::building_feedback(&self.data, building_type, pos);
            let can_place = feedback.can_place();

            // Green if valid, red if invalid
            let color = if can_place {
                Color::new(0.0, 1.0, 0.0, 0.4)
            } else {
                Color::new(1.0, 0.0, 0.0, 0.4)
            };

            for dx in 0..width as i32 {
                for dy in 0..height as i32 {
                    let center = iso.grid_to_screen(Position::new(pos.x + dx, pos.y + dy));
                    draw_iso_diamond(center, iso.tile_w, iso.tile_h, color);
                }
            }

            let outline_color = if can_place { GREEN } else { RED };
            let label_pos = iso.grid_to_screen(pos);

            draw_text(
                &format!(
                    "{} {}x{} | {} salvage",
                    building_type.name(),
                    width,
                    height,
                    building_type.salvage_cost()
                ),
                label_pos.x - 18.0,
                label_pos.y - 8.0,
                14.0,
                outline_color,
            );

            let panel_anchor = self
                .capture_preview_position
                .map(|_| label_pos)
                .unwrap_or_else(|| vec2(mouse_x, mouse_y));
            self.draw_placement_feedback_panel(&feedback, panel_anchor);
        }
    }

    fn draw_placement_feedback_panel(&self, feedback: &BuildingPlacementFeedback, anchor: Vec2) {
        let game_area = self.layout.game_area();
        let width = (game_area.w - 24.0).clamp(260.0, 340.0);
        let height = 124.0;
        let x = (anchor.x + 18.0)
            .min(game_area.x + game_area.w - width - 8.0)
            .max(game_area.x + 8.0);
        let y = (anchor.y + 18.0)
            .min(game_area.y + game_area.h - height - 8.0)
            .max(game_area.y + 8.0);
        let status_color = if feedback.can_place() { GREEN } else { ORANGE };

        draw_rectangle(x, y, width, height, Color::new(0.035, 0.04, 0.045, 0.94));
        draw_rectangle(x, y, 4.0, height, status_color);
        draw_rectangle_lines(x, y, width, height, 1.0, Color::new(0.45, 0.5, 0.55, 0.85));

        draw_text(
            &format!(
                "{} | {}x{} | {} salvage",
                feedback.building_type.name(),
                feedback.footprint.0,
                feedback.footprint.1,
                feedback.cost
            ),
            x + 12.0,
            y + 22.0,
            14.0,
            WHITE,
        );
        draw_text(
            &format!("Helps: {}", feedback.helps),
            x + 12.0,
            y + 43.0,
            12.0,
            LIGHTGRAY,
        );
        draw_text(
            &truncate_text(feedback.purpose, 48),
            x + 12.0,
            y + 63.0,
            11.0,
            Color::new(0.75, 0.78, 0.8, 1.0),
        );

        if let Some(reason) = feedback.invalid_reason.as_ref() {
            draw_text(
                &format!("Blocked: {}", truncate_text(reason, 39)),
                x + 12.0,
                y + 88.0,
                12.0,
                ORANGE,
            );
            draw_text(
                "Move the footprint or pick another building.",
                x + 12.0,
                y + 108.0,
                11.0,
                GRAY,
            );
        } else {
            draw_text(
                &format!("Impact: {}", truncate_text(feedback.impact, 42)),
                x + 12.0,
                y + 88.0,
                12.0,
                LIGHTGRAY,
            );
            draw_text("Click to place this plan.", x + 12.0, y + 108.0, 11.0, GRAY);
        }
    }

    /// Draw the grid with offset for top bar
    fn draw_grid_with_offset(&self) {
        let iso = self.iso_view();

        for y in 0..self.data.grid.height {
            for x in 0..self.data.grid.width {
                let cell_type = self
                    .data
                    .grid
                    .get_cell(x as i32, y as i32)
                    .map(|cell| cell.cell_type);
                let color = terrain_color(cell_type, x as i32, y as i32);

                let center = iso.grid_to_screen(Position::new(x as i32, y as i32));
                draw_iso_diamond(center, iso.tile_w, iso.tile_h, color);
                draw_terrain_detail(
                    center,
                    iso.tile_w,
                    iso.tile_h,
                    terrain_detail(cell_type, x as i32, y as i32),
                );
                draw_iso_diamond_lines(
                    center,
                    iso.tile_w,
                    iso.tile_h,
                    1.0,
                    Color::new(0.12, 0.13, 0.11, 0.45),
                );
            }
        }

        // Highlight hovered cell
        if let Some(pos) = self.hovered_cell {
            let center = iso.grid_to_screen(pos);
            draw_iso_diamond_lines(center, iso.tile_w, iso.tile_h, 2.0, YELLOW);
        }
    }

    /// Draw colonists with offset for top bar
    fn draw_colonists_with_offset(&self, hovered_colonist_id: Option<u32>) {
        let iso = self.iso_view();

        self.draw_social_links(hovered_colonist_id);

        for colonist in &self.data.colonists {
            if colonist.is_on_mission() {
                continue;
            }

            let foot = iso.grid_to_screen(colonist.position);
            let x = foot.x - 16.0;
            let y = foot.y - 28.0;
            let size = 24.0;

            // Colonist color based on state
            let color = match colonist.state {
                ColonistState::Idle => SKYBLUE,
                ColonistState::Moving { .. } => GREEN,
                ColonistState::Working => ORANGE,
                ColonistState::Eating => YELLOW,
                ColonistState::Sleeping => Color::new(0.5, 0.5, 0.8, 1.0),
                ColonistState::OnMission { .. } => PURPLE,
            };

            let center_x = x + 16.0;
            let center_y = y + 16.0;
            draw_ellipse(
                center_x,
                center_y + 12.0,
                12.0,
                4.0,
                0.0,
                Color::new(0.0, 0.0, 0.0, 0.25),
            );
            let social_signal = self.social_body_language_for(colonist);
            if let Some(sprite) = self.art.colonist_sprite_for_pose(
                colonist.id,
                sprite_pose_for_colonist_frame(colonist, social_signal, self.data.tick),
            ) {
                draw_texture_ex(
                    sprite,
                    center_x - 18.0,
                    center_y - 37.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(36.0, 70.0)),
                        ..Default::default()
                    },
                );
            } else {
                draw_rectangle(center_x - 8.0, center_y + 2.0, 16.0, 15.0, color);
                draw_rectangle_lines(center_x - 8.0, center_y + 2.0, 16.0, 15.0, 1.0, WHITE);
                draw_circle(
                    center_x,
                    center_y - 5.0,
                    8.0,
                    Color::new(0.78, 0.68, 0.56, 1.0),
                );
                draw_circle_lines(center_x, center_y - 5.0, 8.0, 1.0, WHITE);
                draw_rectangle(center_x - 5.0, center_y - 10.0, 10.0, 3.0, color);
                draw_line(
                    center_x - 5.0,
                    center_y + 17.0,
                    center_x - 9.0,
                    center_y + 24.0,
                    2.0,
                    LIGHTGRAY,
                );
                draw_line(
                    center_x + 5.0,
                    center_y + 17.0,
                    center_x + 9.0,
                    center_y + 24.0,
                    2.0,
                    LIGHTGRAY,
                );
            }
            draw_circle(
                center_x + 8.0,
                center_y + 5.0,
                3.0,
                job_color(colonist.job_preference),
            );
            if let Some(value) = strongest_relationship_value(colonist) {
                if value.abs() >= 20 {
                    let color = social_color(value, 0.95);
                    draw_circle(center_x - 10.0, center_y - 22.0, 5.0, color);
                    draw_circle_lines(center_x - 10.0, center_y - 22.0, 5.0, 1.0, BLACK);
                    draw_text(
                        if value > 0 { "+" } else { "-" },
                        center_x - 13.0,
                        center_y - 18.0,
                        9.0,
                        style::TEXT_PRIMARY,
                    );
                }
            }
            if let Some(signal) = social_signal {
                let pulse = ((self.data.tick % 90) as f32 / 90.0 * std::f32::consts::TAU)
                    .sin()
                    .abs();
                let signal_color = signal.color(0.46 + pulse * 0.22);
                draw_circle_lines(
                    center_x,
                    center_y - 12.0,
                    15.0 + pulse * 3.0,
                    2.0,
                    signal_color,
                );
                draw_text(
                    signal.symbol(),
                    center_x + 8.0,
                    center_y - 25.0,
                    12.0,
                    signal.color(1.0),
                );
            }
            let selected = Some(colonist.id) == self.selected_colonist_id;
            let hovered = Some(colonist.id) == hovered_colonist_id;
            if selected || hovered {
                let outline_color = if selected {
                    style::ACCENT_GOLD
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.86)
                };
                draw_circle_lines(center_x, center_y, size / 2.0 + 6.0, 3.0, outline_color);
                draw_circle_lines(
                    center_x,
                    center_y,
                    size / 2.0 + 10.0,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.62),
                );

                let name_width = measure_text(&colonist.name, None, 12, 1.0).width;
                draw_rectangle(
                    center_x - name_width * 0.5 - 5.0,
                    y + 28.0,
                    name_width + 10.0,
                    16.0,
                    Color::new(0.03, 0.04, 0.04, 0.76),
                );
                draw_text(
                    &colonist.name,
                    center_x - name_width / 2.0,
                    y + 40.0,
                    12.0,
                    WHITE,
                );
            }
        }
    }

    fn draw_social_links(&self, hovered_colonist_id: Option<u32>) {
        let focus_id = hovered_colonist_id.or(self.selected_colonist_id);
        let iso = self.iso_view();

        for first_index in 0..self.data.colonists.len() {
            let first = &self.data.colonists[first_index];
            if first.is_on_mission() {
                continue;
            }

            for second in self.data.colonists.iter().skip(first_index + 1) {
                if second.is_on_mission() {
                    continue;
                }

                let value = average_relationship_between(first, second);
                let focused_pair = focus_id.is_some_and(|id| id == first.id || id == second.id);
                let shared_location = shared_social_location(first, second);
                let strong_pair = value.abs() >= 25;

                if !(strong_pair || shared_location || focused_pair) || value.abs() < 10 {
                    continue;
                }

                let first_anchor = iso.grid_to_screen(first.position) + vec2(0.0, -28.0);
                let second_anchor = iso.grid_to_screen(second.position) + vec2(0.0, -28.0);
                let color = social_color(
                    value,
                    if focused_pair || shared_location {
                        0.72
                    } else {
                        0.34
                    },
                );

                draw_line(
                    first_anchor.x,
                    first_anchor.y,
                    second_anchor.x,
                    second_anchor.y,
                    if focused_pair || shared_location {
                        2.0
                    } else {
                        1.0
                    },
                    color,
                );

                if focused_pair || (shared_location && value.abs() >= 20) {
                    let mid = (first_anchor + second_anchor) * 0.5;
                    let label = format!("{:+}", value);
                    let width = measure_text(&label, None, 10, 1.0).width;
                    draw_rectangle(
                        mid.x - width * 0.5 - 4.0,
                        mid.y - 11.0,
                        width + 8.0,
                        14.0,
                        Color::new(0.03, 0.04, 0.04, 0.78),
                    );
                    draw_text(
                        &label,
                        mid.x - width * 0.5,
                        mid.y,
                        10.0,
                        social_color(value, 1.0),
                    );
                }
            }
        }
    }

    fn social_body_language_for(&self, colonist: &Colonist) -> Option<SocialBodyLanguage> {
        if matches!(
            colonist.state,
            ColonistState::Moving { .. }
                | ColonistState::Sleeping
                | ColonistState::OnMission { .. }
        ) {
            return None;
        }

        let mut best_signal = None;
        for other in &self.data.colonists {
            if other.id == colonist.id || other.is_on_mission() {
                continue;
            }

            let value = average_relationship_between(colonist, other);
            if value.abs() < 20 {
                continue;
            }

            let active_contact = shared_social_location(colonist, other)
                || shared_assignment_pin(colonist, other)
                || adjacent_positions(colonist.position, other.position);
            if !active_contact {
                continue;
            }

            let signal = if value < 0 {
                SocialBodyLanguage::Tense(value)
            } else {
                SocialBodyLanguage::Supported(value)
            };
            if best_signal
                .map(|best: SocialBodyLanguage| signal.intensity() > best.intensity())
                .unwrap_or(true)
            {
                best_signal = Some(signal);
            }
        }

        best_signal
    }

    fn draw_hover_colonist_card(&self, hovered_colonist_id: Option<u32>) {
        let Some(colonist) = hovered_colonist_id.and_then(|id| self.colonist_by_id(id)) else {
            return;
        };

        let mouse: Vec2 = mouse_position().into();
        draw_tooltip_at(
            mouse + vec2(18.0, 18.0),
            self.layout.game_area(),
            &colonist.name,
            &format!(
                "{} | Mood {:.0} | {}",
                colonist.job_preference.label(),
                colonist.mood,
                colonist_activity_summary(colonist)
            ),
        );
    }

    fn colonist_id_at_mouse(&self) -> Option<u32> {
        let game_area = self.layout.game_area();
        let (mouse_x, mouse_y) = mouse_position();
        if mouse_x < game_area.x
            || mouse_x > game_area.x + game_area.w
            || mouse_y < game_area.y
            || mouse_y > game_area.y + game_area.h
        {
            return None;
        }

        self.data
            .colonists
            .iter()
            .filter(|colonist| !colonist.is_on_mission())
            .filter_map(|colonist| {
                let foot = self.iso_view().grid_to_screen(colonist.position);
                let center_x = foot.x;
                let center_y = foot.y - 8.0;
                let dx = mouse_x - center_x;
                let dy = mouse_y - center_y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= 18.0 * 18.0 {
                    Some((colonist.id, distance_sq))
                } else {
                    None
                }
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(id, _)| id)
    }

    fn building_at_mouse(&self) -> Option<&Building> {
        let game_area = self.layout.game_area();
        let (mouse_x, mouse_y) = mouse_position();
        if !game_area.contains(vec2(mouse_x, mouse_y)) {
            return None;
        }

        let grid_pos = self.iso_view().screen_to_grid(vec2(mouse_x, mouse_y));
        self.data.building_system.get_building_at(grid_pos)
    }

    fn colonist_by_id(&self, id: u32) -> Option<&Colonist> {
        self.data
            .colonists
            .iter()
            .find(|colonist| colonist.id == id)
    }

    fn inspected_colonist(&self, hovered_colonist_id: Option<u32>) -> Option<&Colonist> {
        hovered_colonist_id
            .and_then(|id| self.colonist_by_id(id))
            .or_else(|| {
                self.selected_colonist_id
                    .and_then(|id| self.colonist_by_id(id))
            })
    }
}

fn building_wall_height(building_type: BuildingType, tile_h: f32) -> f32 {
    let multiplier = match building_type {
        BuildingType::Habitat => 0.95,
        BuildingType::MessHall => 0.78,
        BuildingType::Workshop => 1.12,
        BuildingType::Storage => 0.64,
        BuildingType::ExplorationGate => 1.25,
    };
    tile_h * multiplier
}

fn building_outline_style(hovered: bool, assignment_color: Option<Color>) -> Option<(Color, f32)> {
    if hovered {
        Some((Color::new(0.92, 0.8, 0.45, 0.96), 3.0))
    } else {
        assignment_color.map(|color| (Color::new(color.r, color.g, color.b, 0.9), 2.0))
    }
}

fn assignment_marker_with_filter(
    assignment_marker: Option<(&'static str, Color)>,
    filter_match: bool,
) -> Option<(&'static str, Color)> {
    assignment_marker.or_else(|| filter_match.then_some(("FILTER", style::ACCENT_GOLD)))
}

fn building_outline_style_for_assign_filter(
    hovered: bool,
    assignment_color: Option<Color>,
    filter_match: bool,
) -> Option<(Color, f32)> {
    if filter_match {
        Some((style::ACCENT_GOLD, 3.0))
    } else {
        building_outline_style(hovered, assignment_color)
    }
}

fn building_shell_colors(building_type: BuildingType) -> (Color, Color, Color) {
    match building_type {
        BuildingType::Habitat => (
            Color::new(0.24, 0.34, 0.42, 1.0),
            Color::new(0.13, 0.2, 0.25, 1.0),
            Color::new(0.09, 0.15, 0.19, 1.0),
        ),
        BuildingType::MessHall => (
            Color::new(0.48, 0.36, 0.18, 1.0),
            Color::new(0.29, 0.21, 0.11, 1.0),
            Color::new(0.2, 0.15, 0.09, 1.0),
        ),
        BuildingType::Workshop => (
            Color::new(0.39, 0.31, 0.25, 1.0),
            Color::new(0.24, 0.18, 0.15, 1.0),
            Color::new(0.16, 0.13, 0.12, 1.0),
        ),
        BuildingType::Storage => (
            Color::new(0.35, 0.36, 0.34, 1.0),
            Color::new(0.2, 0.22, 0.21, 1.0),
            Color::new(0.15, 0.16, 0.16, 1.0),
        ),
        BuildingType::ExplorationGate => (
            Color::new(0.32, 0.29, 0.45, 1.0),
            Color::new(0.2, 0.18, 0.3, 1.0),
            Color::new(0.13, 0.12, 0.2, 1.0),
        ),
    }
}

fn draw_building_shell_detail(building_type: BuildingType, center: Vec2, width: f32, height: f32) {
    draw_iso_diamond(
        center + vec2(0.0, height * 0.08),
        width * 0.66,
        height * 0.5,
        Color::new(0.05, 0.06, 0.055, 0.52),
    );

    match building_type {
        BuildingType::Habitat => {
            for index in 0..2 {
                let bed_x = center.x - width * 0.2 + index as f32 * width * 0.2;
                draw_rectangle(
                    bed_x,
                    center.y + height * 0.28,
                    12.0,
                    7.0,
                    Color::new(0.56, 0.64, 0.68, 1.0),
                );
                draw_rectangle(
                    bed_x + 1.5,
                    center.y + height * 0.29,
                    4.0,
                    5.0,
                    Color::new(0.18, 0.24, 0.27, 1.0),
                );
            }
            draw_line(
                center.x - width * 0.28,
                center.y + height * 0.23,
                center.x + width * 0.28,
                center.y + height * 0.23,
                1.0,
                Color::new(0.66, 0.74, 0.75, 0.7),
            );
            draw_circle(
                center.x,
                center.y + height * 0.23,
                2.4,
                Color::new(
                    style::HEADING_BLUE.r,
                    style::HEADING_BLUE.g,
                    style::HEADING_BLUE.b,
                    0.85,
                ),
            );
            draw_rectangle(
                center.x - width * 0.14,
                center.y + height * 0.56,
                6.0,
                4.0,
                style::HEADING_BLUE,
            );
            draw_rectangle(
                center.x + width * 0.05,
                center.y + height * 0.56,
                6.0,
                4.0,
                style::HEADING_BLUE,
            );
        }
        BuildingType::MessHall => {
            draw_rectangle(
                center.x - width * 0.17,
                center.y + height * 0.31,
                width * 0.34,
                7.0,
                Color::new(0.17, 0.12, 0.07, 1.0),
            );
            for index in 0..4 {
                let place_x = center.x - width * 0.14 + index as f32 * width * 0.09;
                draw_circle(
                    place_x,
                    center.y + height * 0.34,
                    2.2,
                    Color::new(0.76, 0.68, 0.48, 1.0),
                );
            }
            for index in 0..3 {
                let stool_x = center.x - width * 0.16 + index as f32 * width * 0.16;
                draw_circle(
                    stool_x,
                    center.y + height * 0.44,
                    2.4,
                    Color::new(0.31, 0.22, 0.12, 0.95),
                );
            }
            draw_line(
                center.x - width * 0.22,
                center.y + height * 0.48,
                center.x + width * 0.22,
                center.y + height * 0.48,
                2.0,
                style::ACCENT_GOLD,
            );
            draw_circle(center.x, center.y + height * 0.48, 3.0, style::BAR_GOLD);
        }
        BuildingType::Workshop => {
            draw_rectangle(
                center.x - width * 0.18,
                center.y + height * 0.3,
                width * 0.36,
                8.0,
                Color::new(0.13, 0.1, 0.08, 1.0),
            );
            draw_line(
                center.x - width * 0.13,
                center.y + height * 0.32,
                center.x + width * 0.1,
                center.y + height * 0.32,
                1.0,
                style::ACCENT_GOLD,
            );
            draw_circle(
                center.x + width * 0.14,
                center.y + height * 0.27,
                2.2,
                Color::new(0.9, 0.58, 0.23, 0.9),
            );
            draw_rectangle(
                center.x - 6.0,
                center.y + height * 0.38,
                12.0,
                8.0,
                Color::new(0.07, 0.08, 0.08, 1.0),
            );
            for index in 0..3 {
                let spark_x = center.x + width * 0.03 + index as f32 * 4.0;
                draw_line(
                    spark_x,
                    center.y + height * 0.23,
                    spark_x + 2.0,
                    center.y + height * 0.18,
                    1.0,
                    style::ACCENT_GOLD,
                );
            }
            draw_rectangle(
                center.x - width * 0.25,
                center.y + height * 0.2,
                9.0,
                12.0,
                Color::new(0.09, 0.11, 0.11, 0.95),
            );
            draw_line(
                center.x - width * 0.245,
                center.y + height * 0.25,
                center.x - width * 0.12,
                center.y + height * 0.25,
                1.0,
                style::TEXT_MUTED,
            );
            draw_line(
                center.x + width * 0.18,
                center.y + height * 0.2,
                center.x + width * 0.25,
                center.y - 10.0,
                2.0,
                style::TEXT_MUTED,
            );
        }
        BuildingType::Storage => {
            draw_iso_diamond(
                center + vec2(0.0, height * 0.32),
                width * 0.44,
                height * 0.2,
                Color::new(0.19, 0.2, 0.18, 1.0),
            );
            for index in 0..3 {
                draw_rectangle(
                    center.x - 18.0 + index as f32 * 12.0,
                    center.y + height * 0.5,
                    9.0,
                    7.0,
                    Color::new(0.48, 0.42, 0.32, 1.0),
                );
                draw_rectangle(
                    center.x - 16.0 + index as f32 * 12.0,
                    center.y + height * 0.5,
                    5.0,
                    2.0,
                    Color::new(0.68, 0.58, 0.35, 0.9),
                );
            }
            draw_iso_diamond(
                center + vec2(width * 0.2, height * 0.42),
                width * 0.16,
                height * 0.08,
                Color::new(0.38, 0.45, 0.44, 0.9),
            );
        }
        BuildingType::ExplorationGate => {
            draw_iso_diamond(
                center + vec2(0.0, height * 0.5),
                width * 0.48,
                height * 0.18,
                Color::new(0.1, 0.12, 0.16, 0.86),
            );
            draw_line(
                center.x,
                center.y + height * 0.18,
                center.x,
                center.y - height * 0.4,
                2.0,
                style::TEXT_BODY,
            );
            draw_circle_lines(
                center.x,
                center.y - height * 0.45,
                8.0,
                1.0,
                style::HEADING_BLUE,
            );
            draw_circle(center.x, center.y - height * 0.45, 2.4, style::ACCENT_GOLD);
            let arch_y = center.y + height * 0.36;
            draw_line(
                center.x - width * 0.18,
                arch_y + 18.0,
                center.x - width * 0.18,
                arch_y - 10.0,
                3.0,
                style::HEADING_BLUE,
            );
            draw_line(
                center.x + width * 0.18,
                arch_y + 18.0,
                center.x + width * 0.18,
                arch_y - 10.0,
                3.0,
                style::HEADING_BLUE,
            );
            draw_line(
                center.x - width * 0.18,
                arch_y - 10.0,
                center.x + width * 0.18,
                arch_y - 10.0,
                3.0,
                style::HEADING_BLUE,
            );
            draw_line(
                center.x - width * 0.22,
                arch_y + 14.0,
                center.x - width * 0.34,
                arch_y + 24.0,
                1.2,
                Color::new(0.06, 0.065, 0.065, 0.9),
            );
            draw_line(
                center.x + width * 0.22,
                arch_y + 14.0,
                center.x + width * 0.34,
                arch_y + 24.0,
                1.2,
                Color::new(0.06, 0.065, 0.065, 0.9),
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerrainDetail {
    None,
    Scrap,
    Brush,
    Scorch,
    Wreckage,
    Cable,
    Track,
    SupplyCrate,
    HullPanel,
    SignalBeacon,
    FuelDrum,
}

fn terrain_color(cell_type: Option<CellType>, x: i32, y: i32) -> Color {
    let seed = terrain_seed(x, y);
    let tint = ((seed % 9) as f32 - 4.0) * 0.006;

    match cell_type {
        Some(CellType::Empty) => Color::new(0.18 + tint, 0.16 + tint, 0.105 + tint, 1.0),
        Some(CellType::Floor) => Color::new(0.235 + tint, 0.215 + tint, 0.15 + tint, 1.0),
        Some(CellType::Wall) => Color::new(0.145 + tint, 0.165 + tint, 0.145 + tint, 1.0),
        None => BLACK,
    }
}

fn terrain_detail(cell_type: Option<CellType>, x: i32, y: i32) -> TerrainDetail {
    if cell_type.is_none() {
        return TerrainDetail::None;
    }

    if let Some(detail) = crash_site_detail(x, y) {
        return detail;
    }

    let seed = terrain_seed(x, y);
    if seed % 31 == 0 {
        TerrainDetail::HullPanel
    } else if seed % 29 == 0 {
        TerrainDetail::SupplyCrate
    } else if seed % 23 == 0 {
        TerrainDetail::Scrap
    } else if seed % 19 == 0 {
        TerrainDetail::Scorch
    } else if seed % 13 == 0 {
        TerrainDetail::Brush
    } else {
        TerrainDetail::None
    }
}

fn crash_site_detail(x: i32, y: i32) -> Option<TerrainDetail> {
    if (10..=12).contains(&x) && y == 10 {
        return Some(TerrainDetail::SupplyCrate);
    }

    if (14..=15).contains(&x) && y == 5 {
        return Some(TerrainDetail::SignalBeacon);
    }

    if (5..=7).contains(&x) && (10..=11).contains(&y) && (x + y) % 2 == 0 {
        return Some(TerrainDetail::HullPanel);
    }

    if (12..=14).contains(&x) && y == 7 && x % 2 == 1 {
        return Some(TerrainDetail::FuelDrum);
    }

    if (6..=13).contains(&x) && (7..=9).contains(&y) && (x + y) % 3 == 0 {
        return Some(TerrainDetail::Wreckage);
    }

    if (4..=15).contains(&x) && (x - y).abs() <= 1 && (x + y) % 4 == 0 {
        return Some(TerrainDetail::Track);
    }

    if (7..=15).contains(&x) && (5..=11).contains(&y) && (x * 2 + y) % 11 == 0 {
        return Some(TerrainDetail::Cable);
    }

    None
}

fn draw_terrain_detail(center: Vec2, tile_w: f32, tile_h: f32, detail: TerrainDetail) {
    match detail {
        TerrainDetail::None => {}
        TerrainDetail::Scrap => {
            draw_line(
                center.x - tile_w * 0.11,
                center.y + tile_h * 0.46,
                center.x + tile_w * 0.06,
                center.y + tile_h * 0.34,
                1.0,
                Color::new(0.48, 0.48, 0.42, 0.65),
            );
            draw_circle(
                center.x + tile_w * 0.09,
                center.y + tile_h * 0.58,
                1.4,
                Color::new(0.62, 0.52, 0.34, 0.75),
            );
        }
        TerrainDetail::Brush => {
            draw_line(
                center.x - tile_w * 0.08,
                center.y + tile_h * 0.55,
                center.x - tile_w * 0.02,
                center.y + tile_h * 0.38,
                1.2,
                Color::new(0.22, 0.32, 0.16, 0.7),
            );
            draw_line(
                center.x + tile_w * 0.02,
                center.y + tile_h * 0.58,
                center.x + tile_w * 0.08,
                center.y + tile_h * 0.42,
                1.2,
                Color::new(0.18, 0.28, 0.13, 0.7),
            );
        }
        TerrainDetail::Scorch => {
            draw_iso_diamond(
                center + vec2(0.0, tile_h * 0.12),
                tile_w * 0.48,
                tile_h * 0.48,
                Color::new(0.05, 0.045, 0.035, 0.35),
            );
        }
        TerrainDetail::Wreckage => {
            draw_iso_diamond(
                center + vec2(tile_w * 0.04, tile_h * 0.35),
                tile_w * 0.32,
                tile_h * 0.18,
                Color::new(0.36, 0.35, 0.31, 0.8),
            );
            draw_line(
                center.x - tile_w * 0.12,
                center.y + tile_h * 0.42,
                center.x + tile_w * 0.18,
                center.y + tile_h * 0.34,
                1.2,
                Color::new(0.62, 0.52, 0.34, 0.78),
            );
            draw_circle(
                center.x + tile_w * 0.16,
                center.y + tile_h * 0.34,
                1.8,
                style::ACCENT_GOLD,
            );
        }
        TerrainDetail::Cable => {
            draw_line(
                center.x - tile_w * 0.2,
                center.y + tile_h * 0.5,
                center.x - tile_w * 0.04,
                center.y + tile_h * 0.43,
                1.3,
                Color::new(0.05, 0.055, 0.055, 0.82),
            );
            draw_line(
                center.x - tile_w * 0.04,
                center.y + tile_h * 0.43,
                center.x + tile_w * 0.18,
                center.y + tile_h * 0.54,
                1.3,
                Color::new(0.05, 0.055, 0.055, 0.82),
            );
        }
        TerrainDetail::Track => {
            draw_iso_diamond(
                center + vec2(0.0, tile_h * 0.2),
                tile_w * 0.72,
                tile_h * 0.34,
                Color::new(0.08, 0.07, 0.045, 0.28),
            );
        }
        TerrainDetail::SupplyCrate => {
            draw_iso_diamond(
                center + vec2(0.0, tile_h * 0.36),
                tile_w * 0.36,
                tile_h * 0.22,
                Color::new(0.42, 0.33, 0.21, 0.92),
            );
            draw_rectangle(
                center.x - tile_w * 0.11,
                center.y + tile_h * 0.32,
                tile_w * 0.22,
                tile_h * 0.22,
                Color::new(0.24, 0.18, 0.12, 0.92),
            );
            draw_line(
                center.x - tile_w * 0.1,
                center.y + tile_h * 0.39,
                center.x + tile_w * 0.1,
                center.y + tile_h * 0.39,
                1.0,
                style::ACCENT_GOLD,
            );
        }
        TerrainDetail::HullPanel => {
            draw_iso_diamond(
                center + vec2(tile_w * 0.02, tile_h * 0.32),
                tile_w * 0.46,
                tile_h * 0.2,
                Color::new(0.24, 0.3, 0.31, 0.82),
            );
            draw_line(
                center.x - tile_w * 0.16,
                center.y + tile_h * 0.34,
                center.x + tile_w * 0.16,
                center.y + tile_h * 0.42,
                1.0,
                Color::new(0.66, 0.7, 0.67, 0.7),
            );
        }
        TerrainDetail::SignalBeacon => {
            draw_line(
                center.x,
                center.y + tile_h * 0.5,
                center.x,
                center.y + tile_h * 0.05,
                1.6,
                Color::new(0.55, 0.58, 0.55, 0.92),
            );
            draw_circle(
                center.x,
                center.y + tile_h * 0.02,
                3.0,
                Color::new(
                    style::HEADING_BLUE.r,
                    style::HEADING_BLUE.g,
                    style::HEADING_BLUE.b,
                    0.9,
                ),
            );
            draw_circle_lines(
                center.x,
                center.y + tile_h * 0.02,
                6.0,
                1.0,
                Color::new(
                    style::HEADING_BLUE.r,
                    style::HEADING_BLUE.g,
                    style::HEADING_BLUE.b,
                    0.45,
                ),
            );
        }
        TerrainDetail::FuelDrum => {
            draw_rectangle(
                center.x - tile_w * 0.08,
                center.y + tile_h * 0.31,
                tile_w * 0.16,
                tile_h * 0.28,
                Color::new(0.34, 0.24, 0.18, 0.95),
            );
            draw_ellipse(
                center.x,
                center.y + tile_h * 0.31,
                tile_w * 0.08,
                tile_h * 0.04,
                0.0,
                Color::new(0.52, 0.38, 0.24, 0.95),
            );
            draw_line(
                center.x - tile_w * 0.07,
                center.y + tile_h * 0.46,
                center.x + tile_w * 0.07,
                center.y + tile_h * 0.46,
                1.0,
                style::ACCENT_GOLD,
            );
        }
    }
}

fn terrain_seed(x: i32, y: i32) -> u32 {
    let x = x as u32;
    let y = y as u32;
    x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ 0x9E37_79B9
}

fn placement_result_reason(result: &PlacementResult) -> &'static str {
    match result {
        PlacementResult::Success(_) => "Placement succeeded.",
        PlacementResult::OutOfBounds => "Footprint leaves the map.",
        PlacementResult::AreaOccupied => "Footprint overlaps blocked or occupied space.",
        PlacementResult::InvalidBuilding => "Building configuration is invalid.",
    }
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let mut truncated = text
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn job_color(job_preference: crate::data::colonist::JobPreference) -> Color {
    match job_preference {
        crate::data::colonist::JobPreference::Explorer => PURPLE,
        crate::data::colonist::JobPreference::Builder => YELLOW,
        crate::data::colonist::JobPreference::Cook => GREEN,
        crate::data::colonist::JobPreference::Hauler => GRAY,
        crate::data::colonist::JobPreference::None => WHITE,
    }
}

fn colonist_activity_summary(colonist: &Colonist) -> &'static str {
    match colonist.state {
        ColonistState::Idle => "Idle",
        ColonistState::Moving { .. } => "Moving",
        ColonistState::Working => "Working",
        ColonistState::Eating => "Eating",
        ColonistState::Sleeping => "Resting",
        ColonistState::OnMission { .. } => "On mission",
    }
}

fn sprite_pose_for_state(state: ColonistState) -> SpritePose {
    match state {
        ColonistState::Idle => SpritePose::Idle,
        ColonistState::Moving { .. } => SpritePose::Moving,
        ColonistState::Working => SpritePose::Working,
        ColonistState::Eating => SpritePose::Eating,
        ColonistState::Sleeping => SpritePose::Sleeping,
        ColonistState::OnMission { .. } => SpritePose::Moving,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SocialBodyLanguage {
    Supported(i32),
    Tense(i32),
}

impl SocialBodyLanguage {
    fn intensity(self) -> i32 {
        match self {
            SocialBodyLanguage::Supported(value) | SocialBodyLanguage::Tense(value) => value.abs(),
        }
    }

    fn color(self, alpha: f32) -> Color {
        match self {
            SocialBodyLanguage::Supported(_) => Color::new(
                style::BAR_GREEN.r,
                style::BAR_GREEN.g,
                style::BAR_GREEN.b,
                alpha,
            ),
            SocialBodyLanguage::Tense(_) => Color::new(
                style::ALERT_RED.r,
                style::ALERT_RED.g,
                style::ALERT_RED.b,
                alpha,
            ),
        }
    }

    fn symbol(self) -> &'static str {
        match self {
            SocialBodyLanguage::Supported(_) => "+",
            SocialBodyLanguage::Tense(_) => "!",
        }
    }
}

fn sprite_pose_for_colonist(
    colonist: &Colonist,
    social_signal: Option<SocialBodyLanguage>,
) -> SpritePose {
    sprite_pose_for_colonist_frame(colonist, social_signal, 0)
}

fn sprite_pose_for_colonist_frame(
    colonist: &Colonist,
    social_signal: Option<SocialBodyLanguage>,
    tick: u64,
) -> SpritePose {
    if let Some(signal) = social_signal {
        return match signal {
            SocialBodyLanguage::Supported(_) => {
                if social_pose_uses_alternate_frame(tick) {
                    SpritePose::SupportedReach
                } else {
                    SpritePose::Supported
                }
            }
            SocialBodyLanguage::Tense(_) => {
                if social_pose_uses_alternate_frame(tick) {
                    SpritePose::TenseGuarded
                } else {
                    SpritePose::Tense
                }
            }
        };
    }

    sprite_pose_for_state(colonist.state)
}

fn social_pose_uses_alternate_frame(tick: u64) -> bool {
    (tick / 45) % 2 == 1
}

fn shared_assignment_pin(first: &Colonist, second: &Colonist) -> bool {
    first
        .assigned_habitat
        .is_some_and(|id| second.assigned_habitat == Some(id))
        || first
            .assigned_workplace
            .is_some_and(|id| second.assigned_workplace == Some(id))
}

fn adjacent_positions(first: Position, second: Position) -> bool {
    (first.x - second.x).abs() + (first.y - second.y).abs() <= 1
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpaceAssignmentKind {
    Recovery,
    Work,
}

fn space_assignment_kind(
    job_preference: crate::data::colonist::JobPreference,
    building_type: BuildingType,
) -> Option<SpaceAssignmentKind> {
    if building_type == BuildingType::Habitat {
        return Some(SpaceAssignmentKind::Recovery);
    }

    (building_type == job_preference.work_building_type()).then_some(SpaceAssignmentKind::Work)
}

fn directive_log_detail(directive: PairDirective, first_name: &str, second_name: &str) -> String {
    match directive {
        PairDirective::Pair => format!(
            "{} and {} will prefer the same work and recovery spaces when the settlement has a choice.",
            first_name, second_name
        ),
        PairDirective::Separate => format!(
            "{} and {} will avoid sharing work and recovery spaces when another option exists.",
            first_name, second_name
        ),
    }
}

fn initial_toolbar_mode() -> ToolbarMode {
    std::env::var("TFL_START_TOOLBAR_MODE")
        .ok()
        .and_then(|value| toolbar_mode_from_name(&value))
        .unwrap_or(ToolbarMode::Build)
}

fn initial_selected_building(toolbar_mode: ToolbarMode) -> Option<BuildingType> {
    std::env::var("TFL_START_SELECTED_BUILDING")
        .ok()
        .and_then(|value| building_type_from_name(&value))
        .filter(|building_type| {
            toolbar_mode.uses_building_choices()
                && toolbar_buildings_for_mode(toolbar_mode).contains(building_type)
        })
}

fn initial_capture_preview_position() -> Option<Position> {
    let x = std::env::var("TFL_PREVIEW_GRID_X")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())?;
    let y = std::env::var("TFL_PREVIEW_GRID_Y")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())?;
    Some(Position::new(x, y))
}

fn seed_assign_spaces_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_ASSIGN_SPACES").is_ok_and(|value| value != "0") {
        return;
    }

    let placements = [
        (BuildingType::Habitat, Position::new(3, 4)),
        (BuildingType::Habitat, Position::new(8, 4)),
        (BuildingType::Workshop, Position::new(6, 8)),
        (BuildingType::Storage, Position::new(12, 8)),
    ];

    let mut habitat_id = None;
    let mut workshop_id = None;
    for (building_type, position) in placements {
        if let PlacementResult::Success(building_id) =
            data.building_system
                .try_place_building(&mut data.grid, building_type, position)
        {
            if building_type == BuildingType::Habitat && habitat_id.is_none() {
                habitat_id = Some(building_id);
            } else if building_type == BuildingType::Workshop {
                workshop_id = Some(building_id);
            }
        }
    }

    if let Some(colonist) = data.colonists.iter_mut().find(|colonist| colonist.id == 5) {
        colonist.assigned_habitat = habitat_id;
        colonist.assigned_workplace = workshop_id;
    }
    if let Some(colonist) = data.colonists.iter_mut().find(|colonist| colonist.id == 0) {
        colonist.assigned_habitat = habitat_id;
    }
}

fn seed_social_history_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_SOCIAL_HISTORY").is_ok_and(|value| value != "0") {
        return;
    }

    for entry in [
        SocialHistoryEntry::new(
            0,
            "Crash night summary",
            "The first shelter line held, but Alice and Fiona carried visible tension while Charlie and Evan kept field work steady.",
            "Use Assign to keep Alice and Fiona apart until recovery space improves.",
            50.0,
            1.0,
            2,
            1,
        ),
        SocialHistoryEntry::new(
            1,
            "Mess routine settled",
            "Shared meals improved morale around Bob and Diana, but the workshop queue still created late shifts.",
            "Keep cooks near supportive partners and reduce workshop crowding.",
            58.0,
            6.0,
            2,
            0,
        ),
        SocialHistoryEntry::new(
            2,
            "Workshop strain returned",
            "Alice and Fiona overlapped at the stockpile again, cutting into the recovery gains from yesterday.",
            "Separate tense workers before assigning the next salvage push.",
            47.0,
            -4.0,
            1,
            1,
        ),
        SocialHistoryEntry::new(
            3,
            "Habitat pairs adjusted",
            "Room pins gave Charlie and Evan a reliable recovery loop while Alice took quieter repair shifts.",
            "Protect the supportive pair and avoid crowding the west habitat.",
            61.0,
            8.0,
            2,
            0,
        ),
        SocialHistoryEntry::new(
            4,
            "Late repair friction",
            "The workshop recovered output, but Diana and Fiona clashed during the evening tool handoff.",
            "Move one of them to field prep before the next high-pressure day.",
            53.0,
            -7.0,
            1,
            1,
        ),
    ] {
        data.push_social_history(entry);
    }
}

fn seed_activity_poses_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_ACTIVITY_POSES").is_ok_and(|value| value != "0") {
        return;
    }

    data.time.speed = TimeSpeed::Paused;
    let pose_layout = [
        (0, Position::new(3, 7), ColonistState::Idle),
        (
            1,
            Position::new(6, 7),
            ColonistState::Moving {
                target: Position::new(7, 7),
            },
        ),
        (2, Position::new(9, 7), ColonistState::Working),
        (3, Position::new(12, 7), ColonistState::Eating),
        (4, Position::new(15, 7), ColonistState::Sleeping),
    ];

    for (index, position, state) in pose_layout {
        if let Some(colonist) = data.colonists.get_mut(index) {
            colonist.position = position;
            colonist.visual_x = position.x as f32 * 32.0;
            colonist.visual_y = position.y as f32 * 32.0;
            colonist.state = state;
        }
    }
}

fn initial_selected_colonist_id(data: &GameState, toolbar_mode: ToolbarMode) -> Option<u32> {
    std::env::var("TFL_START_SELECTED_COLONIST")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|id| data.colonists.iter().any(|colonist| colonist.id == *id))
        .or_else(|| {
            (toolbar_mode == ToolbarMode::Assign)
                .then(|| data.colonists.first().map(|colonist| colonist.id))
                .flatten()
        })
}

fn initial_selected_social_history_day(data: &GameState) -> Option<u32> {
    std::env::var("TFL_START_SOCIAL_HISTORY_DAY")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|day| data.social_history.iter().any(|entry| entry.day == *day))
}

fn write_social_archive_markdown(history: &[SocialHistoryEntry]) -> Result<PathBuf, String> {
    let output_dir = PathBuf::from("docs").join("exports");
    std::fs::create_dir_all(&output_dir)
        .map_err(|error| format!("Could not create {}: {}", output_dir.display(), error))?;
    let output_path = output_dir.join("social_archive.md");
    std::fs::write(&output_path, social_archive_markdown(history))
        .map_err(|error| format!("Could not write {}: {}", output_path.display(), error))?;
    Ok(output_path)
}

fn social_archive_markdown(history: &[SocialHistoryEntry]) -> String {
    let mut output = String::from("# The Final Landing Social Archive\n\n");
    output.push_str(&format!("Reports: {}\n\n", history.len()));

    for entry in history.iter().rev() {
        output.push_str(&format!("## Day {}: {}\n\n", entry.day, entry.title));
        output.push_str(&format!(
            "- Mood: {:.0}\n- Relationship: {:+.0}\n- Close pairs: {}\n- Strained pairs: {}\n\n",
            entry.average_mood, entry.average_relationship, entry.close_pairs, entry.strained_pairs
        ));
        output.push_str(&format!("{}\n\n", entry.detail));
        output.push_str(&format!("Recommendation: {}\n\n", entry.recommendation));
    }

    output
}

fn toolbar_mode_from_name(value: &str) -> Option<ToolbarMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "build" => Some(ToolbarMode::Build),
        "rooms" => Some(ToolbarMode::Rooms),
        "objects" => Some(ToolbarMode::Objects),
        "colony" => Some(ToolbarMode::Colony),
        "research" => Some(ToolbarMode::Research),
        "assign" => Some(ToolbarMode::Assign),
        "log" => Some(ToolbarMode::Log),
        _ => None,
    }
}

fn building_type_from_name(value: &str) -> Option<BuildingType> {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "habitat" => Some(BuildingType::Habitat),
        "mess_hall" | "messhall" => Some(BuildingType::MessHall),
        "workshop" => Some(BuildingType::Workshop),
        "storage" => Some(BuildingType::Storage),
        "exploration_gate" | "explorationgate" | "gate" => Some(BuildingType::ExplorationGate),
        _ => None,
    }
}

const ASSIGN_ROSTER_SLOT_COUNT: usize = 5;

fn assign_roster_page_count(
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> usize {
    let selected_exists = selected_colonist_id
        .and_then(|id| colonists.iter().position(|colonist| colonist.id == id))
        .is_some();
    let other_count = (0..colonists.len())
        .filter(|index| Some(colonists[*index].id) != selected_colonist_id)
        .filter(|index| {
            assign_roster_filter_matches(&colonists[*index], active_filter, active_role_filter)
                && assign_building_filter_matches(&colonists[*index], active_building_filter)
        })
        .count();
    let page_size = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(usize::from(selected_exists));

    ((other_count + page_size - 1) / page_size).max(1)
}

fn assign_visible_colonist_indices(
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    page: usize,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<usize> {
    let mut indices = Vec::new();

    let selected_index =
        selected_colonist_id.and_then(|id| colonists.iter().position(|colonist| colonist.id == id));

    if let Some(index) = selected_index {
        indices.push(index);
    }

    let open_slots = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(indices.len());
    let page = page.min(
        assign_roster_page_count(
            colonists,
            selected_colonist_id,
            active_filter,
            active_role_filter,
            active_building_filter,
        ) - 1,
    );

    let roster = assign_sorted_roster_indices(
        colonists,
        selected_index,
        active_filter,
        active_sort,
        active_role_filter,
        active_building_filter,
    );
    indices.extend(roster.into_iter().skip(page * open_slots).take(open_slots));

    indices
}

fn assign_sorted_roster_indices(
    colonists: &[Colonist],
    selected_index: Option<usize>,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<usize> {
    let mut indices = (0..colonists.len())
        .filter(|index| Some(*index) != selected_index)
        .filter(|index| {
            assign_roster_filter_matches(&colonists[*index], active_filter, active_role_filter)
                && assign_building_filter_matches(&colonists[*index], active_building_filter)
        })
        .collect::<Vec<_>>();

    match active_sort {
        AssignRosterSort::Roster => {}
        AssignRosterSort::Mood => indices.sort_by(|left, right| {
            colonists[*left]
                .mood
                .partial_cmp(&colonists[*right].mood)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| colonists[*left].id.cmp(&colonists[*right].id))
        }),
        AssignRosterSort::Bond => indices.sort_by(|left, right| {
            relationship_pressure_score(&colonists[*right])
                .cmp(&relationship_pressure_score(&colonists[*left]))
                .then_with(|| colonists[*left].id.cmp(&colonists[*right].id))
        }),
    }

    indices
}

fn assign_roster_filter_matches(
    colonist: &Colonist,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
) -> bool {
    let relationship_match = match active_filter {
        AssignRosterFilter::All => true,
        AssignRosterFilter::Risk => colonist.relationships.values().any(|value| *value <= -10),
        AssignRosterFilter::Support => colonist.relationships.values().any(|value| *value >= 10),
        AssignRosterFilter::Pinned => {
            colonist.assigned_habitat.is_some() || colonist.assigned_workplace.is_some()
        }
    };
    relationship_match && active_role_filter.is_none_or(|role| colonist.job_preference == role)
}

fn relationship_pressure_score(colonist: &Colonist) -> i32 {
    colonist
        .relationships
        .values()
        .map(|value| value.abs())
        .max()
        .unwrap_or(0)
}

fn assign_building_filter_matches(colonist: &Colonist, building_id: Option<u32>) -> bool {
    building_id.is_none_or(|id| {
        colonist.assigned_habitat == Some(id) || colonist.assigned_workplace == Some(id)
    })
}

fn next_assign_role_filter(current: Option<JobPreference>) -> Option<JobPreference> {
    match current {
        None => Some(JobPreference::Explorer),
        Some(JobPreference::Explorer) => Some(JobPreference::Builder),
        Some(JobPreference::Builder) => Some(JobPreference::Cook),
        Some(JobPreference::Cook) => Some(JobPreference::Hauler),
        Some(JobPreference::Hauler) | Some(JobPreference::None) => None,
    }
}

fn apply_batch_home_pin(
    colonists: &mut [Colonist],
    selected_id: u32,
    habitat_id: u32,
    visible_indices: &[usize],
    capacity: u32,
) -> Vec<String> {
    let mut assigned_count = colonists
        .iter()
        .filter(|colonist| colonist.assigned_habitat == Some(habitat_id))
        .count() as u32;
    let mut assigned = Vec::new();

    for index in visible_indices {
        if assigned_count >= capacity {
            break;
        }

        let Some(colonist) = colonists.get_mut(*index) else {
            continue;
        };
        if colonist.id == selected_id || colonist.assigned_habitat == Some(habitat_id) {
            continue;
        }

        colonist.assigned_habitat = Some(habitat_id);
        assigned_count += 1;
        assigned.push(colonist.name.clone());
    }

    assigned
}

fn apply_batch_work_pin(
    colonists: &mut [Colonist],
    selected_id: u32,
    workplace_id: u32,
    building_type: BuildingType,
    target_indices: &[usize],
) -> Vec<String> {
    let mut assigned = Vec::new();

    for index in target_indices {
        let Some(colonist) = colonists.get_mut(*index) else {
            continue;
        };
        if colonist.id == selected_id
            || colonist.assigned_workplace == Some(workplace_id)
            || colonist.job_preference.work_building_type() != building_type
        {
            continue;
        }

        colonist.assigned_workplace = Some(workplace_id);
        if matches!(
            colonist.state,
            ColonistState::Working | ColonistState::Moving { .. }
        ) {
            colonist.state = ColonistState::Idle;
            colonist.activity_location = ActivityLocation::None;
        }
        assigned.push(colonist.name.clone());
    }

    assigned
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BatchAssignmentScope {
    Page,
    All,
}

impl BatchAssignmentScope {
    fn label(self) -> &'static str {
        match self {
            BatchAssignmentScope::Page => "visible roster",
            BatchAssignmentScope::All => "all compatible survivors",
        }
    }
}

fn batch_assignment_log(
    title: &'static str,
    source_name: &str,
    pin_prefix: &str,
    building_id: u32,
    scope: BatchAssignmentScope,
    assigned: Vec<String>,
) -> (String, String) {
    let detail = if assigned.is_empty() {
        format!(
            "{} had no compatible survivors in {} to copy {}#{} to.",
            source_name,
            scope.label(),
            pin_prefix,
            building_id
        )
    } else {
        format!(
            "Copied {}#{} from {} to {} in {}.",
            pin_prefix,
            building_id,
            source_name,
            truncate_text(&assigned.join(", "), 45),
            scope.label()
        )
    };

    (title.to_string(), detail)
}

fn strongest_relationship_value(colonist: &Colonist) -> Option<i32> {
    colonist
        .relationships
        .values()
        .max_by_key(|value| value.abs())
        .copied()
}

fn average_relationship_between(first: &Colonist, second: &Colonist) -> i32 {
    let first_value = first.relationships.get(&second.id).copied().unwrap_or(0);
    let second_value = second.relationships.get(&first.id).copied().unwrap_or(0);

    if first_value == 0 {
        second_value
    } else if second_value == 0 {
        first_value
    } else {
        (first_value + second_value) / 2
    }
}

fn shared_social_location(first: &Colonist, second: &Colonist) -> bool {
    match (&first.activity_location, &second.activity_location) {
        (
            ActivityLocation::Building {
                building_id: first_id,
                ..
            },
            ActivityLocation::Building {
                building_id: second_id,
                ..
            },
        ) => first_id == second_id,
        (ActivityLocation::Ground(first_pos), ActivityLocation::Ground(second_pos)) => {
            first_pos == second_pos
        }
        _ => false,
    }
}

fn social_color(value: i32, alpha: f32) -> Color {
    if value >= 10 {
        Color::new(
            style::BAR_GREEN.r,
            style::BAR_GREEN.g,
            style::BAR_GREEN.b,
            alpha,
        )
    } else if value <= -10 {
        Color::new(
            style::ALERT_RED.r,
            style::ALERT_RED.g,
            style::ALERT_RED.b,
            alpha,
        )
    } else {
        Color::new(
            style::TEXT_MUTED.r,
            style::TEXT_MUTED.g,
            style::TEXT_MUTED.b,
            alpha,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{JobPreference, Trait};

    #[test]
    fn test_terrain_detail_is_deterministic_and_skips_missing_cells() {
        assert_eq!(
            terrain_detail(Some(CellType::Empty), 7, 11),
            terrain_detail(Some(CellType::Empty), 7, 11)
        );
        assert_eq!(terrain_detail(None, 7, 11), TerrainDetail::None);
    }

    #[test]
    fn test_crash_site_detail_adds_deterministic_map_dressing() {
        assert_eq!(crash_site_detail(10, 10), Some(TerrainDetail::SupplyCrate));
        assert_eq!(crash_site_detail(15, 5), Some(TerrainDetail::SignalBeacon));
        assert_eq!(crash_site_detail(5, 11), Some(TerrainDetail::HullPanel));
        assert_eq!(crash_site_detail(13, 7), Some(TerrainDetail::FuelDrum));
        assert_eq!(crash_site_detail(8, 7), Some(TerrainDetail::Wreckage));
        assert_eq!(crash_site_detail(4, 4), Some(TerrainDetail::Track));
        assert_eq!(crash_site_detail(8, 6), Some(TerrainDetail::Cable));
        assert_eq!(crash_site_detail(0, 0), None);
    }

    #[test]
    fn test_terrain_color_varies_without_leaving_palette() {
        let first = terrain_color(Some(CellType::Empty), 1, 1);
        let second = terrain_color(Some(CellType::Empty), 2, 1);

        assert_ne!(first, second);
        assert!((0.14..=0.22).contains(&first.r));
        assert!((0.08..=0.14).contains(&first.b));
    }

    #[test]
    fn test_average_relationship_uses_bidirectional_values() {
        let mut first = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        let mut second = Colonist::new(
            2,
            "Bob".to_string(),
            Position::new(1, 0),
            Trait::FastWalker,
            JobPreference::Explorer,
        );

        first.relationships.insert(2, 26);
        second.relationships.insert(1, 30);

        assert_eq!(average_relationship_between(&first, &second), 28);
        assert_eq!(strongest_relationship_value(&first), Some(26));
    }

    #[test]
    fn test_space_assignment_kind_matches_role_and_room() {
        assert_eq!(
            space_assignment_kind(JobPreference::Builder, BuildingType::Habitat),
            Some(SpaceAssignmentKind::Recovery)
        );
        assert_eq!(
            space_assignment_kind(JobPreference::Builder, BuildingType::Workshop),
            Some(SpaceAssignmentKind::Work)
        );
        assert_eq!(
            space_assignment_kind(JobPreference::Builder, BuildingType::MessHall),
            None
        );
        assert_eq!(
            space_assignment_kind(JobPreference::Cook, BuildingType::MessHall),
            Some(SpaceAssignmentKind::Work)
        );
    }

    #[test]
    fn test_shared_social_location_requires_same_building_or_ground_cell() {
        let mut first = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        let mut second = Colonist::new(
            2,
            "Bob".to_string(),
            Position::new(1, 0),
            Trait::FastWalker,
            JobPreference::Explorer,
        );

        first.activity_location = ActivityLocation::Building {
            building_id: 7,
            building_type: BuildingType::Workshop,
        };
        second.activity_location = ActivityLocation::Building {
            building_id: 7,
            building_type: BuildingType::Workshop,
        };

        assert!(shared_social_location(&first, &second));

        second.activity_location = ActivityLocation::Ground(Position::new(2, 2));
        assert!(!shared_social_location(&first, &second));
    }

    #[test]
    fn test_toolbar_mode_from_name_accepts_capture_modes() {
        assert_eq!(toolbar_mode_from_name("assign"), Some(ToolbarMode::Assign));
        assert_eq!(
            toolbar_mode_from_name(" Research "),
            Some(ToolbarMode::Research)
        );
        assert_eq!(toolbar_mode_from_name("missing"), None);
    }

    #[test]
    fn test_building_type_from_name_accepts_capture_names() {
        assert_eq!(
            building_type_from_name("mess hall"),
            Some(BuildingType::MessHall)
        );
        assert_eq!(
            building_type_from_name("exploration-gate"),
            Some(BuildingType::ExplorationGate)
        );
        assert_eq!(building_type_from_name("missing"), None);
    }

    #[test]
    fn test_assign_visible_indices_pin_selected_colonist_first() {
        let colonists = (0..6)
            .map(|id| {
                Colonist::new(
                    id,
                    format!("Colonist {}", id),
                    Position::new(id as i32, 0),
                    Trait::HardWorker,
                    JobPreference::Builder,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                0,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                None,
            ),
            vec![5, 0, 1, 2, 3]
        );
        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                None,
                0,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                None,
            ),
            vec![0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn test_assign_visible_indices_page_through_remaining_colonists() {
        let colonists = (0..8)
            .map(|id| {
                Colonist::new(
                    id,
                    format!("Colonist {}", id),
                    Position::new(id as i32, 0),
                    Trait::HardWorker,
                    JobPreference::Builder,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            assign_roster_page_count(&colonists, Some(5), AssignRosterFilter::All, None, None),
            2
        );
        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                1,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                None,
            ),
            vec![5, 4, 6, 7]
        );
        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                None,
                1,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                None,
            ),
            vec![5, 6, 7]
        );
    }

    #[test]
    fn test_assign_visible_indices_filter_and_sort_pressure() {
        let mut colonists = (0..6)
            .map(|id| {
                Colonist::new(
                    id,
                    format!("Colonist {}", id),
                    Position::new(id as i32, 0),
                    Trait::HardWorker,
                    JobPreference::Builder,
                )
            })
            .collect::<Vec<_>>();
        colonists[1].relationships.insert(2, -12);
        colonists[3].relationships.insert(4, -34);
        colonists[4].relationships.insert(3, 22);

        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                0,
                AssignRosterFilter::Risk,
                AssignRosterSort::Bond,
                None,
                None,
            ),
            vec![5, 3, 1]
        );
    }

    #[test]
    fn test_assign_visible_indices_filter_role() {
        let mut colonists = (0..6)
            .map(|id| {
                Colonist::new(
                    id,
                    format!("Colonist {}", id),
                    Position::new(id as i32, 0),
                    Trait::HardWorker,
                    JobPreference::Builder,
                )
            })
            .collect::<Vec<_>>();
        colonists[1].job_preference = JobPreference::Cook;
        colonists[4].job_preference = JobPreference::Cook;

        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                0,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                Some(JobPreference::Cook),
                None,
            ),
            vec![5, 1, 4]
        );
    }

    #[test]
    fn test_assign_visible_indices_filter_building_instance() {
        let mut colonists = (0..6)
            .map(|id| {
                Colonist::new(
                    id,
                    format!("Colonist {}", id),
                    Position::new(id as i32, 0),
                    Trait::HardWorker,
                    JobPreference::Builder,
                )
            })
            .collect::<Vec<_>>();
        colonists[1].assigned_habitat = Some(7);
        colonists[3].assigned_workplace = Some(7);
        colonists[4].assigned_habitat = Some(8);

        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                0,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                Some(7),
            ),
            vec![5, 1, 3]
        );
    }

    #[test]
    fn test_next_assign_role_filter_cycles_assignable_roles() {
        assert_eq!(next_assign_role_filter(None), Some(JobPreference::Explorer));
        assert_eq!(
            next_assign_role_filter(Some(JobPreference::Explorer)),
            Some(JobPreference::Builder)
        );
        assert_eq!(next_assign_role_filter(Some(JobPreference::Hauler)), None);
    }

    #[test]
    fn test_social_archive_markdown_exports_latest_report_first() {
        let history = vec![
            SocialHistoryEntry::new(
                1,
                "Early friction",
                "Alice and Fiona need space.",
                "Use Apart before the next work block.",
                46.0,
                -8.0,
                0,
                1,
            ),
            SocialHistoryEntry::new(
                2,
                "Shared meal",
                "Bob and Diana stabilized dinner.",
                "Keep the supportive pair together.",
                62.0,
                12.0,
                1,
                0,
            ),
        ];

        let export = social_archive_markdown(&history);

        assert!(export.contains("# The Final Landing Social Archive"));
        assert!(export.contains("Reports: 2"));
        assert!(export.find("Day 2").unwrap() < export.find("Day 1").unwrap());
        assert!(export.contains("Recommendation: Keep the supportive pair together."));
    }

    #[test]
    fn test_batch_home_pin_respects_visible_page_and_capacity() {
        let mut colonists = (0..5)
            .map(|id| {
                Colonist::new(
                    id,
                    format!("Colonist {}", id),
                    Position::new(id as i32, 0),
                    Trait::HardWorker,
                    JobPreference::Builder,
                )
            })
            .collect::<Vec<_>>();
        colonists[0].assigned_habitat = Some(7);

        let assigned = apply_batch_home_pin(&mut colonists, 0, 7, &[0, 1, 2, 3], 2);

        assert_eq!(assigned, vec!["Colonist 1".to_string()]);
        assert_eq!(colonists[1].assigned_habitat, Some(7));
        assert_eq!(colonists[2].assigned_habitat, None);
    }

    #[test]
    fn test_batch_work_pin_only_copies_to_compatible_visible_roles() {
        let mut colonists = vec![
            Colonist::new(
                0,
                "Alice".to_string(),
                Position::new(0, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            ),
            Colonist::new(
                1,
                "Bob".to_string(),
                Position::new(1, 0),
                Trait::FastWalker,
                JobPreference::Builder,
            ),
            Colonist::new(
                2,
                "Diana".to_string(),
                Position::new(2, 0),
                Trait::Gourmet,
                JobPreference::Cook,
            ),
        ];
        colonists[0].assigned_workplace = Some(9);
        colonists[1].state = ColonistState::Working;
        colonists[1].activity_location = ActivityLocation::Building {
            building_id: 3,
            building_type: BuildingType::Workshop,
        };

        let assigned =
            apply_batch_work_pin(&mut colonists, 0, 9, BuildingType::Workshop, &[0, 1, 2]);

        assert_eq!(assigned, vec!["Bob".to_string()]);
        assert_eq!(colonists[1].assigned_workplace, Some(9));
        assert_eq!(colonists[1].state, ColonistState::Idle);
        assert_eq!(colonists[1].activity_location, ActivityLocation::None);
        assert_eq!(colonists[2].assigned_workplace, None);
    }

    #[test]
    fn test_batch_assignment_log_names_all_colony_scope() {
        let (_title, detail) = batch_assignment_log(
            "Batch work pins",
            "Alice",
            "W",
            9,
            BatchAssignmentScope::All,
            vec!["Bob".to_string(), "Charlie".to_string()],
        );

        assert!(detail.contains("all compatible survivors"));
        assert!(detail.contains("Bob, Charlie"));
    }

    #[test]
    fn test_sprite_pose_tracks_colonist_state() {
        assert_eq!(sprite_pose_for_state(ColonistState::Idle), SpritePose::Idle);
        assert_eq!(
            sprite_pose_for_state(ColonistState::Moving {
                target: Position::new(1, 1)
            }),
            SpritePose::Moving
        );
        assert_eq!(
            sprite_pose_for_state(ColonistState::Working),
            SpritePose::Working
        );
        assert_eq!(
            sprite_pose_for_state(ColonistState::Eating),
            SpritePose::Eating
        );
        assert_eq!(
            sprite_pose_for_state(ColonistState::Sleeping),
            SpritePose::Sleeping
        );
    }

    #[test]
    fn test_social_body_language_overrides_idle_pose() {
        let colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );

        assert_eq!(
            sprite_pose_for_colonist(&colonist, Some(SocialBodyLanguage::Tense(-24))),
            SpritePose::Tense
        );
        assert_eq!(
            sprite_pose_for_colonist(&colonist, Some(SocialBodyLanguage::Supported(28))),
            SpritePose::Supported
        );
    }

    #[test]
    fn test_social_body_language_cycles_alternate_pose_frames() {
        let colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );

        assert_eq!(
            sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Supported(28)), 45),
            SpritePose::SupportedReach
        );
        assert_eq!(
            sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 45),
            SpritePose::TenseGuarded
        );
        assert_eq!(
            sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 90),
            SpritePose::Tense
        );
    }

    #[test]
    fn test_shared_assignment_and_adjacency_drive_social_contact() {
        let mut first = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(4, 4),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        let mut second = Colonist::new(
            2,
            "Bob".to_string(),
            Position::new(5, 4),
            Trait::FastWalker,
            JobPreference::Builder,
        );

        assert!(adjacent_positions(first.position, second.position));
        first.assigned_workplace = Some(9);
        second.assigned_workplace = Some(9);
        assert!(shared_assignment_pin(&first, &second));
        second.assigned_workplace = Some(10);
        assert!(!shared_assignment_pin(&first, &second));
    }

    #[test]
    fn test_building_outline_style_prioritizes_hover_over_assignment() {
        let hovered = building_outline_style(true, Some(style::BAR_GREEN)).unwrap();
        let assigned = building_outline_style(false, Some(style::BAR_GREEN)).unwrap();

        assert_eq!(hovered.1, 3.0);
        assert_eq!(assigned.1, 2.0);
        assert!(hovered.0.r > assigned.0.r);
        assert!(building_outline_style(false, None).is_none());
    }

    #[test]
    fn test_assignment_marker_with_filter_adds_filter_without_replacing_assignment() {
        let filtered = assignment_marker_with_filter(None, true).unwrap();
        assert_eq!(filtered.0, "FILTER");
        assert_eq!(filtered.1.r, style::ACCENT_GOLD.r);
        assert_eq!(filtered.1.g, style::ACCENT_GOLD.g);
        assert_eq!(filtered.1.b, style::ACCENT_GOLD.b);

        let assigned = assignment_marker_with_filter(Some(("HOME", style::BAR_GREEN)), true)
            .expect("assignment marker should remain visible");
        assert_eq!(assigned.0, "HOME");
        assert_eq!(assigned.1.r, style::BAR_GREEN.r);
        assert!(assignment_marker_with_filter(None, false).is_none());
    }

    #[test]
    fn test_assign_filter_outline_uses_gold_room_highlight() {
        let filtered =
            building_outline_style_for_assign_filter(true, Some(style::BAR_GREEN), true).unwrap();
        assert_eq!(filtered.0.r, style::ACCENT_GOLD.r);
        assert_eq!(filtered.0.g, style::ACCENT_GOLD.g);
        assert_eq!(filtered.0.b, style::ACCENT_GOLD.b);
        assert_eq!(filtered.1, 3.0);

        let assigned =
            building_outline_style_for_assign_filter(false, Some(style::BAR_GREEN), false).unwrap();
        assert_eq!(assigned.1, 2.0);
        assert!(building_outline_style_for_assign_filter(false, None, false).is_none());
    }
}

impl State for GameplayState {
    fn update(&mut self) -> StateTransition {
        // Debug toggle
        if is_key_pressed(KeyCode::F3) {
            self.debug_mode = !self.debug_mode;
        }
        let keyboard_captured = self.update_social_history_search_input();

        if let Some(transition) = self.scenario_restart_transition() {
            return transition;
        }

        // Time speed and priority controls (keyboard)
        if !keyboard_captured && is_key_pressed(KeyCode::Space) {
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

        self.update_pointer_ui_input();
        self.update_colonist_selection();

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
        let (mouse_x, mouse_y) = mouse_position();
        let game_area = self.layout.game_area();
        let grid_pos = self.iso_view().screen_to_grid(vec2(mouse_x, mouse_y));
        if game_area.contains(vec2(mouse_x, mouse_y))
            && self.data.grid.is_in_bounds(grid_pos.x, grid_pos.y)
        {
            self.hovered_cell = Some(grid_pos);
        } else {
            self.hovered_cell = None;
        }

        // Building system updates (keyboard)
        if !keyboard_captured {
            self.update_building_selection();
        }
        self.update_building_placement();

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
        let _ = draw_top_bar(
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
            self.toolbar_mode,
            self.selected_building,
            &self.data.resources,
            &mission_plans,
            &self.data.technology,
            self.data.missions.active_count(),
            &self.data.event_log,
            &self.data.social_history,
            self.assign_roster_page,
            self.assign_roster_filter,
            self.assign_roster_sort,
            self.assign_role_filter,
            self.assign_building_filter,
            self.social_history_page,
            self.social_history_filter,
            &self.social_history_query,
            self.social_history_search_active,
            self.selected_social_history_day,
            self.data.priority.active,
            &self.data.colonists,
            self.selected_colonist_id,
            &colony_summary,
        );
        draw_bottom_toolbar(&self.layout, self.toolbar_mode, self.selected_building);

        // Debug overlay
        if self.debug_mode {
            draw_debug_overlay(
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
