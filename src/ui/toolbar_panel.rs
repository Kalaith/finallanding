use super::Layout;
use crate::data::building::BuildingType;
use crate::data::colonist::Colonist;
use crate::data::event_log::{ColonyLogEntry, LogCategory};
use crate::data::priority::ColonyPriority;
use crate::data::resources::ResourceState;
use crate::data::technology::{TechId, TechnologyState};
use crate::systems::assignment_system::{AssignmentPressure, AssignmentSystem};
use crate::systems::mission_system::MissionPlan;
use crate::ui::font::draw_text;
use crate::ui::hit_zones::{
    toolbar_buildings_for_mode, toolbar_context_item_rect, toolbar_context_rect,
    toolbar_list_item_rect, ToolbarMode,
};
use crate::ui::style;
use crate::ui::tooltip::draw_tooltip_near_mouse;
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
    active_priority: ColonyPriority,
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
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
            draw_build_context(context, mode, selected_building, resources)
        }
        ToolbarMode::Colony => draw_colony_context(context, active_priority),
        ToolbarMode::Research => {
            draw_research_context(context, mission_plans, technology, active_mission_count)
        }
        ToolbarMode::Assign => draw_assign_context(context, colonists, selected_colonist_id),
        ToolbarMode::Log => draw_log_context(context, logs),
    }
}

fn draw_build_context(
    context: Rect,
    mode: ToolbarMode,
    selected_building: Option<BuildingType>,
    resources: &ResourceState,
) {
    let mut hovered_building = None;
    for (index, building_type) in toolbar_buildings_for_mode(mode).iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let can_afford = resources.salvage >= building_type.salvage_cost();
        let selected = selected_building == Some(*building_type);
        let hovered = rect.contains(mouse_position().into());
        if hovered {
            hovered_building = Some(*building_type);
        }
        style::draw_button(rect, selected, hovered);
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

    let helper = match mode {
        ToolbarMode::Rooms => "Room plans shape sleep, meals, and storage pressure.",
        ToolbarMode::Objects => "Work objects produce salvage and survey returns.",
        _ => "Plans reserve salvage and define colony space.",
    };
    draw_text(
        helper,
        context.x + 18.0,
        context.y + 111.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    if let Some(building_type) = hovered_building {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            building_type.name(),
            &format!(
                "{} Cost: {} salvage.",
                building_type.placement_impact(),
                building_type.salvage_cost()
            ),
        );
    }
}

fn draw_colony_context(context: Rect, active_priority: ColonyPriority) {
    let mut hovered_priority = None;
    for (index, priority) in ColonyPriority::all().iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let hovered = rect.contains(mouse_position().into());
        if hovered {
            hovered_priority = Some(*priority);
        }
        style::draw_button(rect, *priority == active_priority, hovered);
        draw_text(
            priority.short_label(),
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_text(
            &style::truncate_text(priority.description(), 26),
            rect.x + 10.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }

    if let Some(priority) = hovered_priority {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            priority.label(),
            priority.description(),
        );
    }
}

fn draw_research_context(
    context: Rect,
    mission_plans: &[MissionPlan],
    technology: &TechnologyState,
    active_mission_count: usize,
) {
    let mut hovered_plan = None;
    for (index, plan) in mission_plans.iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let hovered = rect.contains(mouse_position().into());
        if hovered {
            hovered_plan = Some(plan);
        }
        style::draw_button(rect, plan.recommended, hovered);
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

    if let Some(plan) = hovered_plan {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            plan.definition.name,
            &format!(
                "{} {}",
                plan.definition.description, plan.definition.reward_profile
            ),
        );
    }
}

fn draw_assign_context(context: Rect, colonists: &[Colonist], selected_colonist_id: Option<u32>) {
    let mut hovered_forecast = None;
    let mut hovered_name = None;
    for (index, colonist) in colonists.iter().take(5).enumerate() {
        let rect = toolbar_list_item_rect(context, index);
        let selected = selected_colonist_id == Some(colonist.id);
        let next_role = colonist.job_preference.next_assignable();
        let forecast = AssignmentSystem::forecast_role_change(colonists, colonist.id, next_role);
        let hovered = rect.contains(mouse_position().into());
        if hovered {
            hovered_forecast = Some(forecast.clone());
            hovered_name = Some(colonist.name.clone());
        }
        style::draw_button(rect, selected, hovered);
        draw_rectangle(
            rect.x,
            rect.y,
            3.0,
            rect.h,
            assignment_pressure_color(forecast.pressure),
        );
        draw_text(
            &style::truncate_text(&colonist.name, 11),
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_text(
            &style::truncate_text(
                &format!(
                    "{} -> {}",
                    colonist.job_preference.label(),
                    next_role.label()
                ),
                15,
            ),
            rect.x + 10.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            style::HEADING_BLUE,
        );
    }

    draw_text(
        "Roles redirect next work block and mission eligibility.",
        context.x + 18.0,
        context.y + 111.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    if let (Some(name), Some(forecast)) = (hovered_name, hovered_forecast) {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &name, &forecast.detail);
    }
}

fn draw_log_context(context: Rect, logs: &[ColonyLogEntry]) {
    let mut hovered_log = None;
    for (index, log) in logs.iter().rev().take(3).enumerate() {
        let y = context.y + 54.0 + index as f32 * 22.0;
        let row = Rect::new(context.x + 12.0, y - 14.0, context.w - 24.0, 18.0);
        if row.contains(mouse_position().into()) {
            hovered_log = Some(log);
            draw_rectangle(
                row.x,
                row.y,
                row.w,
                row.h,
                Color::new(0.1, 0.14, 0.15, 0.65),
            );
        }
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

    if let Some(log) = hovered_log {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &log.title, &log.detail);
    }
}

fn toolbar_tooltip_bounds(context: Rect) -> Rect {
    Rect::new(
        context.x,
        (context.y - 58.0).max(0.0),
        context.w,
        context.h + 58.0,
    )
}

fn assignment_pressure_color(pressure: AssignmentPressure) -> Color {
    match pressure {
        AssignmentPressure::Supported => style::BAR_GREEN,
        AssignmentPressure::Neutral => style::HEADING_BLUE,
        AssignmentPressure::Tense => style::ALERT_RED,
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
