use crate::data::building::BuildingType;
use crate::data::game_state::TimeSpeed;
use crate::data::mission::MissionType;
use crate::data::priority::ColonyPriority;
use macroquad::prelude::Rect;

pub const TOP_BAR_BUTTON_Y: f32 = 10.0;
pub const TOP_BAR_BUTTON_H: f32 = 30.0;
pub const SPEED_BUTTON_W: f32 = 50.0;
pub const SPEED_BUTTON_START_X: f32 = 300.0;
pub const PRIORITY_LABEL_X: f32 = 850.0;
pub const PRIORITY_BUTTON_W: f32 = 68.0;
pub const PRIORITY_BUTTON_START_X: f32 = 915.0;
pub const BUTTON_GAP: f32 = 5.0;

const PANEL_SECTION_OFFSET_Y: f32 = 10.0;
const BUILD_BUTTON_START_OFFSET_Y: f32 = 55.0;
const BUILD_BUTTON_H: f32 = 28.0;
const BUILD_BUTTON_GAP: f32 = 3.0;
const PANEL_BUTTON_X_PAD: f32 = 10.0;
const UNDO_OFFSET_Y: f32 = 10.0;
const UNDO_BUTTON_H: f32 = 28.0;
const MISSION_BUTTON_OFFSET_Y: f32 = 52.0;
const MISSION_BUTTON_H: f32 = 25.0;
const MISSION_BUTTON_GAP: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidePanelHit {
    Building(BuildingType),
    Undo,
    Mission(MissionType),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarMode {
    Build,
    Rooms,
    Objects,
    Colony,
    Research,
    Assign,
    Log,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageAction {
    Previous,
    Next,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignBatchAction {
    Home,
    Work,
}

impl ToolbarMode {
    pub fn all() -> &'static [ToolbarMode] {
        &[
            ToolbarMode::Build,
            ToolbarMode::Rooms,
            ToolbarMode::Objects,
            ToolbarMode::Colony,
            ToolbarMode::Research,
            ToolbarMode::Assign,
            ToolbarMode::Log,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            ToolbarMode::Build => "Build",
            ToolbarMode::Rooms => "Rooms",
            ToolbarMode::Objects => "Objects",
            ToolbarMode::Colony => "Colony",
            ToolbarMode::Research => "Research",
            ToolbarMode::Assign => "Assign",
            ToolbarMode::Log => "Log",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            ToolbarMode::Build => "B",
            ToolbarMode::Rooms => "R",
            ToolbarMode::Objects => "O",
            ToolbarMode::Colony => "C",
            ToolbarMode::Research => "T",
            ToolbarMode::Assign => "A",
            ToolbarMode::Log => "L",
        }
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            ToolbarMode::Build => "All construction plans.",
            ToolbarMode::Rooms => "Living, meal, and storage rooms.",
            ToolbarMode::Objects => "Work structures for salvage and survey.",
            ToolbarMode::Colony => "Settlement-wide work priority.",
            ToolbarMode::Research => "Field missions and technology recovery.",
            ToolbarMode::Assign => "Retask survivor work roles.",
            ToolbarMode::Log => "Recent colony events.",
        }
    }

    pub fn uses_building_choices(self) -> bool {
        matches!(
            self,
            ToolbarMode::Build | ToolbarMode::Rooms | ToolbarMode::Objects
        )
    }
}

const ROOM_BUILDINGS: &[BuildingType] = &[
    BuildingType::Habitat,
    BuildingType::MessHall,
    BuildingType::Storage,
];
const OBJECT_BUILDINGS: &[BuildingType] = &[BuildingType::Workshop, BuildingType::ExplorationGate];

pub fn menu_start_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(screen_width * 0.5 - 100.0, screen_height * 0.5, 200.0, 50.0)
}

pub fn restart_button_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 90.0,
        screen_height * 0.5 + 48.0,
        180.0,
        38.0,
    )
}

pub fn speed_button_rect(index: usize) -> Rect {
    Rect::new(
        SPEED_BUTTON_START_X + index as f32 * (SPEED_BUTTON_W + BUTTON_GAP),
        TOP_BAR_BUTTON_Y,
        SPEED_BUTTON_W,
        TOP_BAR_BUTTON_H,
    )
}

pub fn priority_button_rect(index: usize) -> Rect {
    Rect::new(
        PRIORITY_BUTTON_START_X + index as f32 * (PRIORITY_BUTTON_W + BUTTON_GAP),
        TOP_BAR_BUTTON_Y,
        PRIORITY_BUTTON_W,
        TOP_BAR_BUTTON_H,
    )
}

pub fn top_bar_speed_at(x: f32, y: f32) -> Option<TimeSpeed> {
    [
        TimeSpeed::Paused,
        TimeSpeed::Normal,
        TimeSpeed::Fast,
        TimeSpeed::SuperFast,
    ]
    .iter()
    .enumerate()
    .find_map(|(index, speed)| contains(speed_button_rect(index), x, y).then_some(*speed))
}

pub fn top_bar_priority_at(x: f32, y: f32) -> Option<ColonyPriority> {
    ColonyPriority::all()
        .iter()
        .enumerate()
        .find_map(|(index, priority)| {
            contains(priority_button_rect(index), x, y).then_some(*priority)
        })
}

pub fn build_button_rect(panel: Rect, index: usize) -> Rect {
    Rect::new(
        panel.x + PANEL_BUTTON_X_PAD,
        panel.y
            + PANEL_SECTION_OFFSET_Y
            + BUILD_BUTTON_START_OFFSET_Y
            + index as f32 * (BUILD_BUTTON_H + BUILD_BUTTON_GAP),
        panel.w - PANEL_BUTTON_X_PAD * 2.0,
        BUILD_BUTTON_H,
    )
}

pub fn undo_button_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + PANEL_BUTTON_X_PAD,
        panel.y
            + PANEL_SECTION_OFFSET_Y
            + BUILD_BUTTON_START_OFFSET_Y
            + BuildingType::all().len() as f32 * (BUILD_BUTTON_H + BUILD_BUTTON_GAP)
            + UNDO_OFFSET_Y,
        panel.w - PANEL_BUTTON_X_PAD * 2.0,
        UNDO_BUTTON_H,
    )
}

pub fn mission_button_rect(panel: Rect, index: usize) -> Rect {
    let undo = undo_button_rect(panel);
    Rect::new(
        undo.x,
        undo.y + MISSION_BUTTON_OFFSET_Y + index as f32 * (MISSION_BUTTON_H + MISSION_BUTTON_GAP),
        undo.w,
        MISSION_BUTTON_H,
    )
}

pub fn toolbar_button_rect(toolbar: Rect, index: usize) -> Rect {
    let button_w = toolbar.w / ToolbarMode::all().len() as f32;
    Rect::new(
        toolbar.x + index as f32 * button_w,
        toolbar.y + 8.0,
        button_w,
        toolbar.h - 16.0,
    )
}

pub fn toolbar_mode_at(toolbar: Rect, x: f32, y: f32) -> Option<ToolbarMode> {
    ToolbarMode::all()
        .iter()
        .enumerate()
        .find_map(|(index, mode)| {
            contains(toolbar_button_rect(toolbar, index), x, y).then_some(*mode)
        })
}

pub fn toolbar_context_rect(toolbar: Rect) -> Rect {
    Rect::new(toolbar.x, toolbar.y - 138.0, toolbar.w, 126.0)
}

pub fn log_page_previous_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 96.0, context.y + 72.0, 28.0, 17.0)
}

pub fn log_page_next_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 34.0, context.y + 72.0, 28.0, 17.0)
}

pub fn log_page_action_at(context: Rect, x: f32, y: f32) -> Option<PageAction> {
    if contains(log_page_previous_rect(context), x, y) {
        return Some(PageAction::Previous);
    }
    if contains(log_page_next_rect(context), x, y) {
        return Some(PageAction::Next);
    }

    None
}

pub fn assign_page_previous_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 96.0, context.y + 13.0, 28.0, 17.0)
}

pub fn assign_page_next_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 34.0, context.y + 13.0, 28.0, 17.0)
}

pub fn assign_page_action_at(context: Rect, x: f32, y: f32) -> Option<PageAction> {
    if contains(assign_page_previous_rect(context), x, y) {
        return Some(PageAction::Previous);
    }
    if contains(assign_page_next_rect(context), x, y) {
        return Some(PageAction::Next);
    }

    None
}

pub fn assign_batch_home_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 132.0, context.y + 94.0, 58.0, 17.0)
}

pub fn assign_batch_work_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 68.0, context.y + 94.0, 58.0, 17.0)
}

pub fn assign_batch_action_at(context: Rect, x: f32, y: f32) -> Option<AssignBatchAction> {
    if contains(assign_batch_home_rect(context), x, y) {
        return Some(AssignBatchAction::Home);
    }
    if contains(assign_batch_work_rect(context), x, y) {
        return Some(AssignBatchAction::Work);
    }

    None
}

pub fn toolbar_context_item_rect(context: Rect, index: usize) -> Rect {
    let columns = 5;
    let gap = 8.0;
    let item_w = (context.w - 24.0 - gap * (columns - 1) as f32) / columns as f32;
    let item_h = 43.0;
    let col = index % columns;
    let row = index / columns;
    Rect::new(
        context.x + 12.0 + col as f32 * (item_w + gap),
        context.y + 42.0 + row as f32 * (item_h + gap),
        item_w,
        item_h,
    )
}

pub fn toolbar_list_item_rect(context: Rect, index: usize) -> Rect {
    toolbar_context_item_rect(context, index)
}

pub fn toolbar_buildings_for_mode(mode: ToolbarMode) -> &'static [BuildingType] {
    match mode {
        ToolbarMode::Build => BuildingType::all(),
        ToolbarMode::Rooms => ROOM_BUILDINGS,
        ToolbarMode::Objects => OBJECT_BUILDINGS,
        ToolbarMode::Colony | ToolbarMode::Research | ToolbarMode::Assign | ToolbarMode::Log => &[],
    }
}

pub fn toolbar_building_at(context: Rect, x: f32, y: f32) -> Option<BuildingType> {
    toolbar_building_at_for_mode(context, ToolbarMode::Build, x, y)
}

pub fn toolbar_building_at_for_mode(
    context: Rect,
    mode: ToolbarMode,
    x: f32,
    y: f32,
) -> Option<BuildingType> {
    toolbar_buildings_for_mode(mode)
        .iter()
        .enumerate()
        .find_map(|(index, building_type)| {
            contains(toolbar_context_item_rect(context, index), x, y).then_some(*building_type)
        })
}

pub fn toolbar_priority_at(context: Rect, x: f32, y: f32) -> Option<ColonyPriority> {
    ColonyPriority::all()
        .iter()
        .enumerate()
        .find_map(|(index, priority)| {
            contains(toolbar_context_item_rect(context, index), x, y).then_some(*priority)
        })
}

pub fn toolbar_colonist_index_at(context: Rect, count: usize, x: f32, y: f32) -> Option<usize> {
    (0..count.min(5)).find(|index| contains(toolbar_list_item_rect(context, *index), x, y))
}

pub fn toolbar_mission_at(context: Rect, x: f32, y: f32) -> Option<MissionType> {
    MissionType::all()
        .iter()
        .enumerate()
        .find_map(|(index, mission_type)| {
            contains(toolbar_context_item_rect(context, index), x, y).then_some(*mission_type)
        })
}

pub fn side_panel_hit_at(panel: Rect, x: f32, y: f32) -> Option<SidePanelHit> {
    if !contains(panel, x, y) {
        return None;
    }

    for (index, building_type) in BuildingType::all().iter().enumerate() {
        if contains(build_button_rect(panel, index), x, y) {
            return Some(SidePanelHit::Building(*building_type));
        }
    }

    if contains(undo_button_rect(panel), x, y) {
        return Some(SidePanelHit::Undo);
    }

    for (index, mission_type) in MissionType::all().iter().enumerate() {
        if contains(mission_button_rect(panel, index), x, y) {
            return Some(SidePanelHit::Mission(*mission_type));
        }
    }

    None
}

fn contains(rect: Rect, x: f32, y: f32) -> bool {
    x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn center(rect: Rect) -> (f32, f32) {
        (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
    }

    #[test]
    fn test_menu_start_rect_contains_button_center() {
        let rect = menu_start_rect(1280.0, 720.0);

        assert!(contains(rect, 640.0, 385.0));
    }

    #[test]
    fn test_restart_rect_contains_button_center() {
        let rect = restart_button_rect(1280.0, 720.0);

        assert!(contains(rect, 640.0, 427.0));
    }

    #[test]
    fn test_top_bar_speed_hit_zones_match_visible_buttons() {
        let (x, y) = center(speed_button_rect(1));

        assert_eq!(top_bar_speed_at(x, y), Some(TimeSpeed::Normal));
        assert_eq!(top_bar_speed_at(10.0, y), None);
    }

    #[test]
    fn test_top_bar_priority_hit_zones_match_visible_buttons() {
        let (x, y) = center(priority_button_rect(2));

        assert_eq!(top_bar_priority_at(x, y), Some(ColonyPriority::Survey));
    }

    #[test]
    fn test_side_panel_building_undo_and_mission_hits() {
        let panel = Rect::new(1060.0, 50.0, 220.0, 670.0);
        let (habitat_x, habitat_y) = center(build_button_rect(panel, 0));
        let (gate_x, gate_y) = center(build_button_rect(panel, 4));
        let (undo_x, undo_y) = center(undo_button_rect(panel));
        let (supply_x, supply_y) = center(mission_button_rect(panel, 0));
        let (scan_x, scan_y) = center(mission_button_rect(panel, 1));
        let (deep_x, deep_y) = center(mission_button_rect(panel, 2));

        assert_eq!(
            side_panel_hit_at(panel, habitat_x, habitat_y),
            Some(SidePanelHit::Building(BuildingType::Habitat))
        );
        assert_eq!(
            side_panel_hit_at(panel, gate_x, gate_y),
            Some(SidePanelHit::Building(BuildingType::ExplorationGate))
        );
        assert_eq!(
            side_panel_hit_at(panel, undo_x, undo_y),
            Some(SidePanelHit::Undo)
        );
        assert_eq!(
            side_panel_hit_at(panel, supply_x, supply_y),
            Some(SidePanelHit::Mission(MissionType::SupplyRun))
        );
        assert_eq!(
            side_panel_hit_at(panel, scan_x, scan_y),
            Some(SidePanelHit::Mission(MissionType::PerimeterScan))
        );
        assert_eq!(
            side_panel_hit_at(panel, deep_x, deep_y),
            Some(SidePanelHit::Mission(MissionType::DeepSurvey))
        );
    }

    #[test]
    fn test_toolbar_mode_hit_zones_match_visible_buttons() {
        let toolbar = Rect::new(380.0, 640.0, 520.0, 66.0);
        let (build_x, build_y) = center(toolbar_button_rect(toolbar, 0));
        let (log_x, log_y) = center(toolbar_button_rect(toolbar, 6));

        assert_eq!(
            toolbar_mode_at(toolbar, build_x, build_y),
            Some(ToolbarMode::Build)
        );
        assert_eq!(
            toolbar_mode_at(toolbar, log_x, log_y),
            Some(ToolbarMode::Log)
        );
        assert_eq!(toolbar_mode_at(toolbar, 10.0, 10.0), None);
    }

    #[test]
    fn test_toolbar_context_hits_buildings_and_missions() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (habitat_x, habitat_y) = center(toolbar_context_item_rect(context, 0));
        let (gate_x, gate_y) = center(toolbar_context_item_rect(context, 4));

        assert_eq!(
            toolbar_building_at(context, habitat_x, habitat_y),
            Some(BuildingType::Habitat)
        );
        assert_eq!(
            toolbar_building_at(context, gate_x, gate_y),
            Some(BuildingType::ExplorationGate)
        );
        assert_eq!(
            toolbar_mission_at(context, habitat_x, habitat_y),
            Some(MissionType::SupplyRun)
        );
    }

    #[test]
    fn test_toolbar_building_filters_match_modes() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (first_x, first_y) = center(toolbar_context_item_rect(context, 0));
        let (third_x, third_y) = center(toolbar_context_item_rect(context, 2));

        assert_eq!(
            toolbar_building_at_for_mode(context, ToolbarMode::Rooms, first_x, first_y),
            Some(BuildingType::Habitat)
        );
        assert_eq!(
            toolbar_building_at_for_mode(context, ToolbarMode::Rooms, third_x, third_y),
            Some(BuildingType::Storage)
        );
        assert_eq!(
            toolbar_building_at_for_mode(context, ToolbarMode::Objects, third_x, third_y),
            None
        );
    }

    #[test]
    fn test_toolbar_priority_and_colonist_hits() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (priority_x, priority_y) = center(toolbar_context_item_rect(context, 1));
        let (colonist_x, colonist_y) = center(toolbar_list_item_rect(context, 4));

        assert_eq!(
            toolbar_priority_at(context, priority_x, priority_y),
            Some(ColonyPriority::Stockpile)
        );
        assert_eq!(
            toolbar_colonist_index_at(context, 5, colonist_x, colonist_y),
            Some(4)
        );
        assert_eq!(
            toolbar_colonist_index_at(context, 4, colonist_x, colonist_y),
            None
        );
    }

    #[test]
    fn test_log_page_hit_zones_match_archive_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (prev_x, prev_y) = center(log_page_previous_rect(context));
        let (next_x, next_y) = center(log_page_next_rect(context));

        assert_eq!(
            log_page_action_at(context, prev_x, prev_y),
            Some(PageAction::Previous)
        );
        assert_eq!(
            log_page_action_at(context, next_x, next_y),
            Some(PageAction::Next)
        );
        assert_eq!(
            log_page_action_at(context, context.x + 18.0, context.y + 82.0),
            None
        );
    }

    #[test]
    fn test_assign_page_hit_zones_match_roster_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (prev_x, prev_y) = center(assign_page_previous_rect(context));
        let (next_x, next_y) = center(assign_page_next_rect(context));

        assert_eq!(
            assign_page_action_at(context, prev_x, prev_y),
            Some(PageAction::Previous)
        );
        assert_eq!(
            assign_page_action_at(context, next_x, next_y),
            Some(PageAction::Next)
        );
        assert_eq!(
            assign_page_action_at(context, context.x + 18.0, context.y + 82.0),
            None
        );
    }

    #[test]
    fn test_assign_batch_hit_zones_match_copy_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (home_x, home_y) = center(assign_batch_home_rect(context));
        let (work_x, work_y) = center(assign_batch_work_rect(context));

        assert_eq!(
            assign_batch_action_at(context, home_x, home_y),
            Some(AssignBatchAction::Home)
        );
        assert_eq!(
            assign_batch_action_at(context, work_x, work_y),
            Some(AssignBatchAction::Work)
        );
        assert_eq!(
            assign_batch_action_at(context, context.x + 18.0, context.y + 111.0),
            None
        );
    }
}
