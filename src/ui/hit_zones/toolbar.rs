use crate::data::building::BuildingType;
use crate::data::mission::MissionType;
use crate::data::priority::ColonyPriority;
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, HitTarget};

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
    hit_test(
        ToolbarMode::all()
            .iter()
            .enumerate()
            .map(|(index, mode)| HitTarget::new(toolbar_button_rect(toolbar, index), *mode)),
        vec2(x, y),
    )
}

pub fn toolbar_context_rect(toolbar: Rect) -> Rect {
    Rect::new(toolbar.x, toolbar.y - 138.0, toolbar.w, 126.0)
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

pub fn toolbar_building_at_for_mode(
    context: Rect,
    mode: ToolbarMode,
    x: f32,
    y: f32,
) -> Option<BuildingType> {
    hit_test(
        toolbar_buildings_for_mode(mode)
            .iter()
            .enumerate()
            .map(|(index, building_type)| {
                HitTarget::new(toolbar_context_item_rect(context, index), *building_type)
            }),
        vec2(x, y),
    )
}

pub fn toolbar_priority_at(context: Rect, x: f32, y: f32) -> Option<ColonyPriority> {
    hit_test(
        ColonyPriority::all()
            .iter()
            .enumerate()
            .map(|(index, priority)| {
                HitTarget::new(toolbar_context_item_rect(context, index), *priority)
            }),
        vec2(x, y),
    )
}

pub fn toolbar_colonist_index_at(context: Rect, count: usize, x: f32, y: f32) -> Option<usize> {
    hit_test(
        (0..count.min(5))
            .map(|index| HitTarget::new(toolbar_list_item_rect(context, index), index)),
        vec2(x, y),
    )
}

pub fn toolbar_mission_at(context: Rect, x: f32, y: f32) -> Option<MissionType> {
    hit_test(
        MissionType::all()
            .iter()
            .enumerate()
            .map(|(index, mission_type)| {
                HitTarget::new(toolbar_context_item_rect(context, index), *mission_type)
            }),
        vec2(x, y),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn center(rect: Rect) -> (f32, f32) {
        (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
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
            toolbar_building_at_for_mode(context, ToolbarMode::Build, habitat_x, habitat_y),
            Some(BuildingType::Habitat)
        );
        assert_eq!(
            toolbar_building_at_for_mode(context, ToolbarMode::Build, gate_x, gate_y),
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
}
