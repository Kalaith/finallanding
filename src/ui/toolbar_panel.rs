use super::Layout;
use crate::data::building::BuildingType;
use crate::data::colonist::{relationship_label, Colonist};
use crate::data::event_log::{ColonyLogEntry, LogCategory, SocialHistoryEntry};
use crate::data::priority::ColonyPriority;
use crate::data::resources::ResourceState;
use crate::data::technology::{TechId, TechnologyState};
use crate::systems::assignment_system::{AssignmentPressure, AssignmentSystem};
use crate::systems::mission_system::MissionPlan;
use crate::systems::relationship_directive_system::{PairDirective, RelationshipDirectiveSystem};
use crate::systems::summary_system::{ColonyPressureSummary, RelationshipPairSummary};
use crate::ui::font::draw_text;
use crate::ui::hit_zones::{
    assign_batch_home_rect, assign_batch_work_rect, assign_page_next_rect,
    assign_page_previous_rect, log_page_next_rect, log_page_previous_rect,
    toolbar_buildings_for_mode, toolbar_context_item_rect, toolbar_context_rect,
    toolbar_list_item_rect, ToolbarMode,
};
use crate::ui::style;
use crate::ui::tooltip::draw_tooltip_near_mouse;
use macroquad::prelude::*;

pub const SOCIAL_TIMELINE_PAGE_SIZE: usize = 3;
const ASSIGN_ROSTER_SLOT_COUNT: usize = 5;

pub fn draw_toolbar_context_panel(
    layout: &Layout,
    mode: ToolbarMode,
    selected_building: Option<BuildingType>,
    resources: &ResourceState,
    mission_plans: &[MissionPlan],
    technology: &TechnologyState,
    active_mission_count: usize,
    logs: &[ColonyLogEntry],
    social_history: &[SocialHistoryEntry],
    assign_roster_page: usize,
    social_history_page: usize,
    active_priority: ColonyPriority,
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    colony_summary: &ColonyPressureSummary,
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
        ToolbarMode::Assign => draw_assign_context(
            context,
            colonists,
            selected_colonist_id,
            assign_roster_page,
            technology,
        ),
        ToolbarMode::Log => draw_log_context(
            context,
            logs,
            social_history,
            social_history_page,
            colony_summary,
        ),
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

fn draw_assign_context(
    context: Rect,
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    assign_roster_page: usize,
    technology: &TechnologyState,
) {
    let mut hovered_forecast = None;
    let mut hovered_name = None;
    let mut hovered_directive = None;
    let page_count = assign_roster_page_count(colonists, selected_colonist_id);
    let current_page = assign_roster_page.min(page_count.saturating_sub(1));
    if page_count > 1 {
        draw_assign_page_controls(context, current_page, page_count);
    }

    for (slot, colonist) in assign_visible_colonists(colonists, selected_colonist_id, current_page)
        .into_iter()
        .enumerate()
    {
        let rect = toolbar_list_item_rect(context, slot);
        let selected = selected_colonist_id == Some(colonist.id);
        let hovered = rect.contains(mouse_position().into());
        let pair_action = selected_colonist_id
            .filter(|selected_id| *selected_id != colonist.id)
            .and_then(|selected_id| assign_pair_action(colonists, selected_id, colonist.id));
        let pin_warning = assignment_pin_warning(colonist, colonists, technology);

        style::draw_button(rect, selected, hovered);
        draw_rectangle(
            rect.x,
            rect.y,
            3.0,
            rect.h,
            pin_warning
                .as_ref()
                .filter(|_| selected)
                .map(|_| style::ALERT_RED)
                .or_else(|| {
                    pair_action
                        .as_ref()
                        .map(|action| directive_color(action.directive))
                })
                .unwrap_or_else(|| {
                    let next_role = colonist.job_preference.next_assignable();
                    let forecast =
                        AssignmentSystem::forecast_role_change(colonists, colonist.id, next_role);
                    assignment_pressure_color(forecast.pressure)
                }),
        );
        draw_text(
            &style::truncate_text(&colonist.name, 11),
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );

        if selected {
            if hovered {
                hovered_directive =
                    Some(selected_assignment_detail(colonist, colonists, technology));
                hovered_name = Some(colonist.name.clone());
            }
            let label = pin_warning
                .as_ref()
                .map(|warning| format!("{} {}", warning.label, selected_assignment_label(colonist)))
                .unwrap_or_else(|| selected_assignment_label(colonist));
            draw_text(
                &style::truncate_text(&label, 17),
                rect.x + 10.0,
                rect.y + 34.0,
                style::TINY_SIZE,
                if pin_warning.is_some() {
                    style::ALERT_RED
                } else {
                    style::TEXT_BODY
                },
            );
        } else if let Some(action) = pair_action {
            if hovered {
                hovered_directive = Some(action.detail);
                hovered_name = Some(colonist.name.clone());
            }
            draw_text(
                &style::truncate_text(&action.label, 16),
                rect.x + 10.0,
                rect.y + 34.0,
                style::TINY_SIZE,
                directive_color(action.directive),
            );
        } else {
            let next_role = colonist.job_preference.next_assignable();
            let forecast =
                AssignmentSystem::forecast_role_change(colonists, colonist.id, next_role);
            if hovered {
                hovered_forecast = Some(forecast.clone());
                hovered_name = Some(colonist.name.clone());
            }
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
    }

    let selected_colonist =
        selected_colonist_id.and_then(|id| colonists.iter().find(|colonist| colonist.id == id));
    let selected_warning = selected_colonist
        .and_then(|colonist| assignment_pin_warning(colonist, colonists, technology));
    if let Some(colonist) = selected_colonist {
        draw_assign_batch_controls(context, colonist);
    }
    let footer = selected_colonist
        .map(|colonist| format!("Selected {} | click map spaces to pin rooms", colonist.name))
        .unwrap_or_else(|| {
            "Roles, pair directives, and space directives shape work blocks.".to_string()
        });
    let footer = selected_warning
        .as_ref()
        .map(|warning| warning.detail.clone())
        .unwrap_or(footer);
    draw_text(
        &style::truncate_text(&footer, 76),
        context.x + 18.0,
        context.y + 111.0,
        style::TINY_SIZE,
        if selected_warning.is_some() {
            style::ALERT_RED
        } else {
            style::TEXT_MUTED
        },
    );

    if let (Some(name), Some(detail)) = (hovered_name.clone(), hovered_directive) {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &name, &detail);
    } else if let (Some(name), Some(forecast)) = (hovered_name, hovered_forecast) {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &name, &forecast.detail);
    }
}

fn draw_assign_batch_controls(context: Rect, selected_colonist: &Colonist) {
    let home = assign_batch_home_rect(context);
    let work = assign_batch_work_rect(context);
    let mouse = mouse_position().into();
    let home_enabled = selected_colonist.assigned_habitat.is_some();
    let work_enabled = selected_colonist.assigned_workplace.is_some();

    style::draw_button(home, false, home_enabled && home.contains(mouse));
    style::draw_button(work, false, work_enabled && work.contains(mouse));
    draw_text(
        "COPY H",
        home.x + 7.0,
        home.y + 12.0,
        style::TINY_SIZE,
        if home_enabled {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_text(
        "COPY W",
        work.x + 7.0,
        work.y + 12.0,
        style::TINY_SIZE,
        if work_enabled {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
}

fn draw_assign_page_controls(context: Rect, current_page: usize, page_count: usize) {
    let previous = assign_page_previous_rect(context);
    let next = assign_page_next_rect(context);
    let mouse = mouse_position().into();
    let can_go_previous = current_page > 0;
    let can_go_next = current_page + 1 < page_count;

    style::draw_button(previous, false, can_go_previous && previous.contains(mouse));
    style::draw_button(next, false, can_go_next && next.contains(mouse));
    draw_text(
        "<",
        previous.x + 10.0,
        previous.y + 12.0,
        style::TINY_SIZE,
        if can_go_previous {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_text(
        ">",
        next.x + 10.0,
        next.y + 12.0,
        style::TINY_SIZE,
        if can_go_next {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_text(
        &format!("{}/{}", current_page + 1, page_count),
        context.x + context.w - 63.0,
        context.y + 25.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
}

struct AssignPairAction {
    label: String,
    detail: String,
    directive: PairDirective,
}

fn assign_roster_page_count(colonists: &[Colonist], selected_colonist_id: Option<u32>) -> usize {
    let selected_exists = selected_colonist_id
        .and_then(|id| colonists.iter().position(|colonist| colonist.id == id))
        .is_some();
    let other_count = colonists.len().saturating_sub(usize::from(selected_exists));
    let page_size = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(usize::from(selected_exists));

    ((other_count + page_size - 1) / page_size).max(1)
}

fn assign_visible_colonists<'a>(
    colonists: &'a [Colonist],
    selected_colonist_id: Option<u32>,
    page: usize,
) -> Vec<&'a Colonist> {
    let mut visible = Vec::new();

    let selected_id =
        selected_colonist_id.filter(|id| colonists.iter().any(|colonist| colonist.id == *id));
    if let Some(selected_id) = selected_id {
        if let Some(colonist) = colonists.iter().find(|colonist| colonist.id == selected_id) {
            visible.push(colonist);
        }
    }

    let open_slots = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(visible.len());
    let page = page.min(assign_roster_page_count(colonists, selected_colonist_id) - 1);

    visible.extend(
        colonists
            .iter()
            .filter(|colonist| Some(colonist.id) != selected_id)
            .skip(page * open_slots)
            .take(open_slots),
    );

    visible
}

fn assign_pair_action(
    colonists: &[Colonist],
    selected_id: u32,
    target_id: u32,
) -> Option<AssignPairAction> {
    let current =
        RelationshipDirectiveSystem::directive_for_pair(colonists, selected_id, target_id);
    let directive = current.or_else(|| {
        RelationshipDirectiveSystem::recommended_directive(colonists, selected_id, target_id)
    })?;
    let value =
        RelationshipDirectiveSystem::average_relationship(colonists, selected_id, target_id)
            .unwrap_or(0);
    let label = match current {
        Some(active) => format!("{} set {:+}", active.label(), value),
        None => format!("{} {:+}", directive.label(), value),
    };
    let detail = RelationshipDirectiveSystem::directive_detail(colonists, selected_id, target_id);

    Some(AssignPairAction {
        label,
        detail,
        directive,
    })
}

fn selected_assignment_label(colonist: &Colonist) -> String {
    let home = colonist
        .assigned_habitat
        .map(|id| format!("H#{}", id))
        .unwrap_or_else(|| "H--".to_string());
    let work = colonist
        .assigned_workplace
        .map(|id| format!("W#{}", id))
        .unwrap_or_else(|| "W--".to_string());
    format!("{} {}", home, work)
}

fn selected_assignment_detail(
    colonist: &Colonist,
    colonists: &[Colonist],
    technology: &TechnologyState,
) -> String {
    let base = format!(
        "Click this card to cycle role. Click a compatible map building to pin or clear recovery/work space. Current pins: {}.",
        selected_assignment_label(colonist)
    );
    assignment_pin_warning(colonist, colonists, technology)
        .map(|warning| format!("{} {}", base, warning.detail))
        .unwrap_or(base)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AssignmentPinWarning {
    label: String,
    detail: String,
}

fn assignment_pin_warning(
    colonist: &Colonist,
    colonists: &[Colonist],
    technology: &TechnologyState,
) -> Option<AssignmentPinWarning> {
    if let Some(habitat_id) = colonist.assigned_habitat {
        let count = colonists
            .iter()
            .filter(|candidate| candidate.assigned_habitat == Some(habitat_id))
            .count() as u32;
        let capacity = 2 + technology.habitat_capacity_bonus();
        if count > capacity {
            return Some(AssignmentPinWarning {
                label: "CAP".to_string(),
                detail: format!(
                    "Habitat #{} over capacity: {}/{} pinned survivors.",
                    habitat_id, count, capacity
                ),
            });
        }

        if let Some((name, value)) = first_assignment_conflict(
            colonist,
            colonists,
            AssignmentPinLocation::Habitat(habitat_id),
        ) {
            return Some(AssignmentPinWarning {
                label: "TENSE".to_string(),
                detail: format!(
                    "{}: {} {:+} in H#{}. Pin another room or use Apart.",
                    name,
                    relationship_label(value),
                    value,
                    habitat_id
                ),
            });
        }
    }

    if let Some(workplace_id) = colonist.assigned_workplace {
        if let Some((name, value)) = first_assignment_conflict(
            colonist,
            colonists,
            AssignmentPinLocation::Work(workplace_id),
        ) {
            return Some(AssignmentPinWarning {
                label: "TENSE".to_string(),
                detail: format!(
                    "{}: {} {:+} at W#{}. Pin another space or use Apart.",
                    name,
                    relationship_label(value),
                    value,
                    workplace_id
                ),
            });
        }
    }

    None
}

#[derive(Clone, Copy)]
enum AssignmentPinLocation {
    Habitat(u32),
    Work(u32),
}

fn first_assignment_conflict(
    colonist: &Colonist,
    colonists: &[Colonist],
    location: AssignmentPinLocation,
) -> Option<(String, i32)> {
    colonists
        .iter()
        .filter(|candidate| candidate.id != colonist.id)
        .filter(|candidate| match location {
            AssignmentPinLocation::Habitat(id) => candidate.assigned_habitat == Some(id),
            AssignmentPinLocation::Work(id) => candidate.assigned_workplace == Some(id),
        })
        .filter_map(|candidate| {
            let value = RelationshipDirectiveSystem::average_relationship(
                colonists,
                colonist.id,
                candidate.id,
            )
            .unwrap_or(0);
            (value <= -10).then(|| (candidate.name.clone(), value))
        })
        .min_by_key(|(_, value)| *value)
}

fn draw_log_context(
    context: Rect,
    logs: &[ColonyLogEntry],
    social_history: &[SocialHistoryEntry],
    social_history_page: usize,
    summary: &ColonyPressureSummary,
) {
    let mut hovered_history = None;
    let social_brief = social_brief_lines(summary);
    draw_text(
        &social_brief.header,
        context.x + 18.0,
        context.y + 51.0,
        style::TINY_SIZE,
        social_brief.color,
    );
    draw_text(
        &style::truncate_text(&social_brief.detail, 72),
        context.x + 18.0,
        context.y + 68.0,
        style::TINY_SIZE,
        style::TEXT_BODY,
    );

    let page_count = social_history_page_count(social_history);
    let current_page = social_history_page.min(page_count.saturating_sub(1));
    let timeline = social_timeline_rows(social_history, current_page);
    if !timeline.is_empty() {
        draw_text(
            "SOCIAL TIMELINE",
            context.x + 18.0,
            context.y + 82.0,
            style::TINY_SIZE,
            style::HEADING_BLUE,
        );
        if page_count > 1 {
            draw_log_page_controls(context, current_page, page_count);
        }

        for (index, row) in timeline.iter().enumerate() {
            let y = context.y + 94.0 + index as f32 * 13.0;
            let rect = Rect::new(context.x + 12.0, y - 11.0, context.w - 24.0, 13.0);
            if rect.contains(mouse_position().into()) {
                hovered_history = Some(row);
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.1, 0.14, 0.15, 0.7),
                );
            }
            draw_rectangle(rect.x, rect.y, 3.0, rect.h, row.color);
            draw_text(
                &format!("D{}", row.day),
                rect.x + 9.0,
                y,
                style::TINY_SIZE,
                row.color,
            );
            draw_text(
                &style::truncate_text(&row.title, 34),
                rect.x + 39.0,
                y,
                style::TINY_SIZE,
                style::TEXT_BODY,
            );
            draw_text(
                &row.metrics,
                rect.x + rect.w - 104.0,
                y,
                style::TINY_SIZE,
                style::TEXT_MUTED,
            );
        }

        if let Some(row) = hovered_history {
            draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &row.title, &row.detail);
        }
        return;
    }

    let mut hovered_log = None;
    for (index, log) in logs.iter().rev().take(2).enumerate() {
        let y = context.y + 91.0 + index as f32 * 20.0;
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

fn draw_log_page_controls(context: Rect, current_page: usize, page_count: usize) {
    let previous = log_page_previous_rect(context);
    let next = log_page_next_rect(context);
    let mouse = mouse_position().into();
    let can_go_previous = current_page > 0;
    let can_go_next = current_page + 1 < page_count;

    style::draw_button(previous, false, can_go_previous && previous.contains(mouse));
    style::draw_button(next, false, can_go_next && next.contains(mouse));
    draw_text(
        "<",
        previous.x + 10.0,
        previous.y + 12.0,
        style::TINY_SIZE,
        if can_go_previous {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_text(
        ">",
        next.x + 10.0,
        next.y + 12.0,
        style::TINY_SIZE,
        if can_go_next {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_text(
        &format!("{}/{}", current_page + 1, page_count),
        context.x + context.w - 63.0,
        context.y + 84.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
}

struct SocialTimelineRow {
    day: u32,
    title: String,
    detail: String,
    metrics: String,
    color: Color,
}

pub fn social_history_page_count(history: &[SocialHistoryEntry]) -> usize {
    ((history.len() + SOCIAL_TIMELINE_PAGE_SIZE - 1) / SOCIAL_TIMELINE_PAGE_SIZE).max(1)
}

fn social_timeline_rows(history: &[SocialHistoryEntry], page: usize) -> Vec<SocialTimelineRow> {
    let page = page.min(social_history_page_count(history).saturating_sub(1));
    history
        .iter()
        .rev()
        .skip(page * SOCIAL_TIMELINE_PAGE_SIZE)
        .take(SOCIAL_TIMELINE_PAGE_SIZE)
        .map(|entry| SocialTimelineRow {
            day: entry.day,
            title: entry.title.clone(),
            detail: format!("{} {}", entry.detail, entry.recommendation),
            metrics: format!(
                "M{:.0} R{:+.0} T{}",
                entry.average_mood, entry.average_relationship, entry.strained_pairs
            ),
            color: social_history_color(entry),
        })
        .collect()
}

fn social_history_color(entry: &SocialHistoryEntry) -> Color {
    if entry.strained_pairs > 0 || entry.average_relationship < -5.0 {
        style::ALERT_RED
    } else if entry.close_pairs > 0 || entry.average_relationship > 8.0 {
        style::BAR_GREEN
    } else {
        style::HEADING_BLUE
    }
}

struct SocialBriefLines {
    header: String,
    detail: String,
    color: Color,
}

fn social_brief_lines(summary: &ColonyPressureSummary) -> SocialBriefLines {
    let color = if summary.strained_pairs > 0 {
        style::ALERT_RED
    } else if summary.close_pairs > 0 {
        style::BAR_GREEN
    } else {
        style::HEADING_BLUE
    };

    let header = format!(
        "Social pressure: mood {:.0} | close {} | tense {}",
        summary.average_mood, summary.close_pairs, summary.strained_pairs
    );
    let detail = if let Some(pair) = summary
        .weakest_pair
        .as_ref()
        .filter(|pair| pair.value <= -10)
    {
        pair_line("Watch", pair)
    } else if let Some(pair) = summary
        .strongest_pair
        .as_ref()
        .filter(|pair| pair.value >= 10)
    {
        pair_line("Protect", pair)
    } else {
        "No strong social signal yet; routine will shape the first bonds.".to_string()
    };

    SocialBriefLines {
        header,
        detail,
        color,
    }
}

fn pair_line(prefix: &str, pair: &RelationshipPairSummary) -> String {
    format!(
        "{} {} / {}: {} {:+}",
        prefix, pair.first_name, pair.second_name, pair.label, pair.value
    )
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

fn directive_color(directive: PairDirective) -> Color {
    match directive {
        PairDirective::Pair => style::BAR_GREEN,
        PairDirective::Separate => style::ALERT_RED,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{JobPreference, Trait};
    use crate::data::types::Position;

    fn test_colonist(id: u32) -> Colonist {
        Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        )
    }

    #[test]
    fn test_assign_visible_colonists_pin_selected_first() {
        let colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        let visible = assign_visible_colonists(&colonists, Some(5), 0)
            .into_iter()
            .map(|colonist| colonist.id)
            .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 0, 1, 2, 3]);
    }

    #[test]
    fn test_assign_visible_colonists_page_through_roster() {
        let colonists = (0..8).map(test_colonist).collect::<Vec<_>>();
        let page = assign_visible_colonists(&colonists, Some(5), 1)
            .into_iter()
            .map(|colonist| colonist.id)
            .collect::<Vec<_>>();

        assert_eq!(assign_roster_page_count(&colonists, Some(5)), 2);
        assert_eq!(page, vec![5, 4, 6, 7]);
    }

    #[test]
    fn test_assign_pair_action_reports_active_directive() {
        let mut colonists = vec![test_colonist(1), test_colonist(2)];
        colonists[0].relationships.insert(2, -24);
        colonists[1].relationships.insert(1, -20);
        colonists[0].avoided_partner_id = Some(2);
        colonists[1].avoided_partner_id = Some(1);

        let action = assign_pair_action(&colonists, 1, 2).unwrap();

        assert_eq!(action.directive, PairDirective::Separate);
        assert_eq!(action.label, "Apart set -22");
    }

    #[test]
    fn test_selected_assignment_label_reports_room_pins() {
        let mut colonist = test_colonist(1);
        assert_eq!(selected_assignment_label(&colonist), "H-- W--");

        colonist.assigned_habitat = Some(3);
        colonist.assigned_workplace = Some(8);

        assert_eq!(selected_assignment_label(&colonist), "H#3 W#8");
        assert!(selected_assignment_detail(
            &colonist,
            &[colonist.clone()],
            &TechnologyState::default()
        )
        .contains("H#3 W#8"));
    }

    #[test]
    fn test_assignment_pin_warning_flags_over_capacity_habitat() {
        let mut colonists = vec![test_colonist(1), test_colonist(2), test_colonist(3)];
        for colonist in &mut colonists {
            colonist.assigned_habitat = Some(7);
        }

        let warning =
            assignment_pin_warning(&colonists[0], &colonists, &TechnologyState::default()).unwrap();

        assert_eq!(warning.label, "CAP");
        assert!(warning.detail.contains("3/2"));
    }

    #[test]
    fn test_assignment_pin_warning_flags_tense_shared_workplace() {
        let mut colonists = vec![test_colonist(1), test_colonist(2)];
        colonists[0].assigned_workplace = Some(9);
        colonists[1].assigned_workplace = Some(9);
        colonists[0].relationships.insert(2, -24);
        colonists[1].relationships.insert(1, -20);

        let warning =
            assignment_pin_warning(&colonists[0], &colonists, &TechnologyState::default()).unwrap();

        assert_eq!(warning.label, "TENSE");
        assert!(warning.detail.contains("Colonist 2"));
        assert!(warning.detail.contains("W#9"));
    }

    #[test]
    fn test_social_brief_prioritizes_tense_pair() {
        let summary = ColonyPressureSummary {
            average_mood: 47.0,
            average_relationship: -2.0,
            close_pairs: 1,
            strained_pairs: 1,
            connected_pairs: vec![],
            tense_pairs: vec![],
            strongest_pair: None,
            weakest_pair: Some(RelationshipPairSummary {
                first_name: "Alice".to_string(),
                second_name: "Fiona".to_string(),
                value: -24,
                label: "Tense",
            }),
        };

        let brief = social_brief_lines(&summary);

        assert!(brief.header.contains("tense 1"));
        assert_eq!(brief.detail, "Watch Alice / Fiona: Tense -24");
        assert_eq!(brief.color, style::ALERT_RED);
    }

    #[test]
    fn test_social_brief_names_strongest_pair_when_stable() {
        let summary = ColonyPressureSummary {
            average_mood: 62.0,
            average_relationship: 4.0,
            close_pairs: 1,
            strained_pairs: 0,
            connected_pairs: vec![],
            tense_pairs: vec![],
            strongest_pair: Some(RelationshipPairSummary {
                first_name: "Charlie".to_string(),
                second_name: "Evan".to_string(),
                value: 28,
                label: "Friendly",
            }),
            weakest_pair: None,
        };

        let brief = social_brief_lines(&summary);

        assert_eq!(brief.detail, "Protect Charlie / Evan: Friendly +28");
        assert_eq!(brief.color, style::BAR_GREEN);
    }

    #[test]
    fn test_latest_social_history_is_available_to_log_context() {
        let history = SocialHistoryEntry::new(
            2,
            "Day 2 summary",
            "Relationships stabilized.",
            "Keep Charlie and Evan together.",
            64.0,
            5.0,
            1,
            0,
        );

        assert_eq!(history.day, 2);
        assert_eq!(history.recommendation, "Keep Charlie and Evan together.");
    }

    #[test]
    fn test_social_timeline_rows_show_latest_three_days_first() {
        let history = (0..5)
            .map(|day| {
                SocialHistoryEntry::new(
                    day,
                    format!("Day {} summary", day),
                    "Social detail.",
                    "Recommendation.",
                    50.0 + day as f32,
                    day as f32,
                    day,
                    0,
                )
            })
            .collect::<Vec<_>>();

        let rows = social_timeline_rows(&history, 0);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].day, 4);
        assert_eq!(rows[1].day, 3);
        assert_eq!(rows[2].day, 2);
        assert_eq!(rows[0].metrics, "M54 R+4 T0");
    }

    #[test]
    fn test_social_timeline_rows_page_through_archive() {
        let history = (0..7)
            .map(|day| SocialHistoryEntry::new(day, "", "", "", 50.0, day as f32, 0, 0))
            .collect::<Vec<_>>();

        let first_page = social_timeline_rows(&history, 0);
        let second_page = social_timeline_rows(&history, 1);
        let last_page = social_timeline_rows(&history, 2);
        let clamped_page = social_timeline_rows(&history, 99);

        assert_eq!(social_history_page_count(&history), 3);
        assert_eq!(
            first_page.iter().map(|row| row.day).collect::<Vec<_>>(),
            vec![6, 5, 4]
        );
        assert_eq!(
            second_page.iter().map(|row| row.day).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(
            last_page.iter().map(|row| row.day).collect::<Vec<_>>(),
            vec![0]
        );
        assert_eq!(clamped_page[0].day, 0);
    }

    #[test]
    fn test_social_timeline_colors_pressure_and_support() {
        let tense = SocialHistoryEntry::new(2, "", "", "", 42.0, -2.0, 0, 1);
        let close = SocialHistoryEntry::new(3, "", "", "", 68.0, 9.0, 1, 0);
        let neutral = SocialHistoryEntry::new(4, "", "", "", 55.0, 0.0, 0, 0);

        assert_eq!(social_history_color(&tense), style::ALERT_RED);
        assert_eq!(social_history_color(&close), style::BAR_GREEN);
        assert_eq!(social_history_color(&neutral), style::HEADING_BLUE);
    }
}
