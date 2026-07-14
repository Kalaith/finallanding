use crate::data::building::{Building, BuildingType};
use crate::data::colonist::{ActivityLocation, Colonist, ColonistState, JobPreference};
use crate::data::event_log::{LogCategory, SocialHistoryEntry};
use crate::data::game_state::GameState;
use crate::data::game_state::TimeSpeed;
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
use crate::systems::planning_system::PlanningSystem;
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
use crate::ui::{
    assign_batch_action_at, assign_filter_at, assign_page_action_at, assign_role_filter_at,
    assign_sort_at, draw_advisor_overlay, draw_bottom_toolbar, draw_colonist_inspector,
    draw_debug_overlay, draw_right_rail, draw_toolbar_context_panel, draw_top_bar, log_filter_at,
    log_page_action_at, log_search_action_at, log_timeline_row_at, restart_button_rect,
    social_history_page_count, social_timeline_day_at, toolbar_building_at_for_mode,
    toolbar_buildings_for_mode, toolbar_colonist_index_at, toolbar_context_rect,
    toolbar_mission_at, toolbar_mode_at, toolbar_priority_at, top_bar_priority_at,
    top_bar_speed_at, AssignBatchAction, AssignRosterFilter, AssignRosterSort, IsoView, Layout,
    LogFilter, LogSearchAction, PageAction, PlaceholderArt, ToolbarAssignData, ToolbarLogData,
    ToolbarMode, ToolbarPanelData, ToolbarResearchData,
};
use macroquad::prelude::*;
use macroquad_toolkit::debug::DebugOverlay;
use macroquad_toolkit::input::InputState;
use std::path::PathBuf;

const SECONDS_PER_GAME_TICK: f32 = 0.25;

pub struct GameplayState {
    pub data: GameState,
    pub(crate) hovered_cell: Option<Position>,
    /// Currently selected building type for placement (None = not in build mode)
    pub(crate) selected_building: Option<BuildingType>,
    /// Fixed preview grid position used only by screenshot verification captures.
    pub(crate) capture_preview_position: Option<Position>,
    /// Selected colonist for relationship inspection.
    pub(crate) selected_colonist_id: Option<u32>,
    /// Time event collector for processing time-based events
    pub(crate) time_events: TimeEventCollector,
    /// Previous tick for event detection
    pub(crate) prev_tick: u64,
    /// Accumulates real time before advancing the simulation by game ticks
    pub(crate) time_accumulator: f32,
    /// UI layout configuration
    pub(crate) layout: Layout,
    /// Smoothed FPS/frame-time overlay with colony stat lines, toggled by F3.
    pub(crate) debug_overlay: DebugOverlay,
    /// Active bottom-toolbar mode.
    pub(crate) toolbar_mode: ToolbarMode,
    /// Current page in the Assign mode roster.
    pub(crate) assign_roster_page: usize,
    /// Active filter in the Assign mode roster.
    pub(crate) assign_roster_filter: AssignRosterFilter,
    /// Active sort in the Assign mode roster.
    pub(crate) assign_roster_sort: AssignRosterSort,
    /// Optional work-role filter in the Assign mode roster.
    pub(crate) assign_role_filter: Option<JobPreference>,
    /// Optional room/work-space instance filter in the Assign mode roster.
    pub(crate) assign_building_filter: Option<u32>,
    /// Current page in the Log mode social archive.
    pub(crate) social_history_page: usize,
    /// Active filter in the Log mode social archive.
    pub(crate) social_history_filter: LogFilter,
    /// Search query for the Log mode social archive.
    pub(crate) social_history_query: String,
    /// Whether typed keys should edit the Log mode social archive search.
    pub(crate) social_history_search_active: bool,
    /// Selected daily social report for persistent Log drilldown.
    pub(crate) selected_social_history_day: Option<u32>,
    /// Placeholder visual assets extracted from the rebuild reference.
    pub(crate) art: PlaceholderArt,
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
            debug_overlay: DebugOverlay::new(),
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
}

#[path = "game_state_assign_batch_commands.rs"]
mod game_state_assign_batch_commands;
#[path = "game_state_assignment_batch_rules.rs"]
mod game_state_assignment_batch_rules;
pub(crate) use game_state_assignment_batch_rules::*;
#[path = "game_state_assign_filter_commands.rs"]
mod game_state_assign_filter_commands;
#[path = "game_state_assign_roster_commands.rs"]
mod game_state_assign_roster_commands;
#[path = "game_state_assign_space_commands.rs"]
mod game_state_assign_space_commands;
#[path = "game_state_assignment_roster.rs"]
mod game_state_assignment_roster;
pub(crate) use game_state_assignment_roster::*;
#[path = "game_state_assignment_space_rules.rs"]
mod game_state_assignment_space_rules;
pub(crate) use game_state_assignment_space_rules::*;
#[path = "game_state_building_commands.rs"]
mod game_state_building_commands;
#[path = "game_state_keyboard_input.rs"]
mod game_state_keyboard_input;
#[path = "game_state_lifecycle.rs"]
mod game_state_lifecycle;
#[path = "game_state_log_commands.rs"]
mod game_state_log_commands;
#[path = "game_state_map_selection.rs"]
mod game_state_map_selection;
#[path = "game_state_mission_commands.rs"]
mod game_state_mission_commands;
#[path = "game_state_placement_results.rs"]
mod game_state_placement_results;
pub(crate) use game_state_placement_results::*;
#[path = "game_state_pointer_bounds.rs"]
mod game_state_pointer_bounds;
#[path = "game_state_priority_commands.rs"]
mod game_state_priority_commands;
#[path = "game_state_queries.rs"]
mod game_state_queries;
#[path = "game_state_relationship_commands.rs"]
mod game_state_relationship_commands;
#[path = "game_state_relationship_contact.rs"]
mod game_state_relationship_contact;
pub(crate) use game_state_relationship_contact::*;
#[path = "game_state_relationship_directive_logs.rs"]
mod game_state_relationship_directive_logs;
pub(crate) use game_state_relationship_directive_logs::*;
#[path = "game_state_setup.rs"]
mod game_state_setup;
pub(crate) use game_state_setup::*;
#[path = "game_state_simulation.rs"]
mod game_state_simulation;
#[path = "game_state_social_archive.rs"]
mod game_state_social_archive;
pub(crate) use game_state_social_archive::*;
#[path = "game_state_text.rs"]
mod game_state_text;
pub(crate) use game_state_text::*;
#[path = "game_state_toolbar_input.rs"]
mod game_state_toolbar_input;
