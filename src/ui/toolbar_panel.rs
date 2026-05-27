use super::Layout;
use crate::data::building::BuildingType;
use crate::data::event_log::{ColonyLogEntry, LogCategory};
use crate::data::resources::ResourceState;
use crate::data::technology::{TechId, TechnologyState};
use crate::systems::mission_system::MissionPlan;
use crate::ui::hit_zones::{toolbar_context_item_rect, toolbar_context_rect, ToolbarMode};
use crate::ui::style;
use macroquad::prelude::*;

pub fn draw_toolbar_context_panel(
    layout: &Layout,
    mode: ToolbarMode,
    selected_building: Option<BuildingType>,
    resources: &ResourceState,
    mission_plans: &[MissionPlan],
    technology: &TechnologyState,
    active_mission_count: usize,
    logs: &[ColonyLogEntry],
) {
    let context = toolbar_context_rect(layout.bottom_toolbar());
    style::draw_panel(context);
    style::draw_section_title(
        mode.label().to_uppercase().as_str(),
        context.x + 14.0,
        context.y + 27.0,
    );

    match mode {
        ToolbarMode::Build | ToolbarMode::Rooms | ToolbarMode::Objects => {
            draw_build_context(context, selected_building, resources)
        }
        ToolbarMode::Colony => draw_colony_context(context),
        ToolbarMode::Research => {
            draw_research_context(context, mission_plans, technology, active_mission_count)
        }
        ToolbarMode::Assign => draw_assign_context(context),
        ToolbarMode::Log => draw_log_context(context, logs),
    }
}

fn draw_build_context(
    context: Rect,
    selected_building: Option<BuildingType>,
    resources: &ResourceState,
) {
    for (index, building_type) in BuildingType::all().iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let can_afford = resources.salvage >= building_type.salvage_cost();
        let selected = selected_building == Some(*building_type);
        style::draw_button(rect, selected, rect.contains(mouse_position().into()));
        let (r, g, b) = building_type.color();
        draw_rectangle(
            rect.x + 9.0,
            rect.y + 9.0,
            14.0,
            14.0,
            Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0),
        );
        draw_text(
            &style::truncate_text(building_type.name(), 12),
            rect.x + 30.0,
            rect.y + 18.0,
            style::TINY_SIZE,
            if can_afford {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_MUTED
            },
        );
        draw_text(
            &format!("{} salvage", building_type.salvage_cost()),
            rect.x + 30.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            if can_afford {
                style::TEXT_BODY
            } else {
                style::ALERT_RED
            },
        );
    }
}

fn draw_colony_context(context: Rect) {
    let lines = [
        "1 Recovery lowers pressure and mission danger.",
        "2 Stockpile boosts food, storage, and repair work.",
        "3 Survey pushes exploration and research returns.",
    ];
    for (index, line) in lines.iter().enumerate() {
        draw_text(
            line,
            context.x + 18.0,
            context.y + 56.0 + index as f32 * 22.0,
            style::SMALL_SIZE,
            style::TEXT_BODY,
        );
    }
}

fn draw_research_context(
    context: Rect,
    mission_plans: &[MissionPlan],
    technology: &TechnologyState,
    active_mission_count: usize,
) {
    for (index, plan) in mission_plans.iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        style::draw_button(
            rect,
            plan.recommended,
            rect.contains(mouse_position().into()),
        );
        draw_text(
            plan.definition.short_name,
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_text(
            &format!(
                "{}m | {}%",
                plan.definition.duration_minutes, plan.danger_percent
            ),
            rect.x + 10.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }

    let tech_label = technology
        .next_locked_tech()
        .map(|tech| tech.name())
        .unwrap_or("All field tech unlocked");
    draw_text(
        &format!(
            "Away {} | Tech {}/{} | Next: {}",
            active_mission_count,
            technology.unlocked_count(),
            TechId::all().len(),
            tech_label
        ),
        context.x + 18.0,
        context.y + 109.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
}

fn draw_assign_context(context: Rect) {
    draw_text(
        "Click a colonist in the colony view to inspect relationships and current work.",
        context.x + 18.0,
        context.y + 62.0,
        style::SMALL_SIZE,
        style::TEXT_BODY,
    );
    draw_text(
        "Detailed job assignment is queued for the next rebuild pass.",
        context.x + 18.0,
        context.y + 86.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
}

fn draw_log_context(context: Rect, logs: &[ColonyLogEntry]) {
    for (index, log) in logs.iter().rev().take(3).enumerate() {
        let y = context.y + 54.0 + index as f32 * 22.0;
        draw_text(
            category_prefix(log.category),
            context.x + 18.0,
            y,
            style::TINY_SIZE,
            style::HEADING_BLUE,
        );
        draw_text(
            &style::truncate_text(
                &format!("{:02}:{:02} {}", log.hour, log.minute, log.title),
                64,
            ),
            context.x + 52.0,
            y,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }
}

fn category_prefix(category: LogCategory) -> &'static str {
    match category {
        LogCategory::Time => "TIME",
        LogCategory::Social => "SOC",
        LogCategory::Work => "WORK",
        LogCategory::Mood => "MOOD",
        LogCategory::Resource => "RES",
        LogCategory::Mission => "MIS",
        LogCategory::Technology => "TECH",
        LogCategory::Colony => "COL",
        LogCategory::System => "SYS",
    }
}
