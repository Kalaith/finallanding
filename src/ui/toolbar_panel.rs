use super::Layout;
use crate::data::building::BuildingType;
use crate::data::colonist::Colonist;
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
    social_history: &[SocialHistoryEntry],
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
        ToolbarMode::Assign => draw_assign_context(context, colonists, selected_colonist_id),
        ToolbarMode::Log => draw_log_context(context, logs, social_history, colony_summary),
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
    let mut hovered_directive = None;
    for (slot, colonist) in assign_visible_colonists(colonists, selected_colonist_id)
        .into_iter()
        .enumerate()
    {
        let rect = toolbar_list_item_rect(context, slot);
        let selected = selected_colonist_id == Some(colonist.id);
        let hovered = rect.contains(mouse_position().into());
        let pair_action = selected_colonist_id
            .filter(|selected_id| *selected_id != colonist.id)
            .and_then(|selected_id| assign_pair_action(colonists, selected_id, colonist.id));

        style::draw_button(rect, selected, hovered);
        draw_rectangle(
            rect.x,
            rect.y,
            3.0,
            rect.h,
            pair_action
                .as_ref()
                .map(|action| directive_color(action.directive))
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

        if let Some(action) = pair_action {
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

    let footer = selected_colonist_id
        .and_then(|id| colonists.iter().find(|colonist| colonist.id == id))
        .map(|colonist| format!("Selected {} | role and social directives", colonist.name))
        .unwrap_or_else(|| {
            "Roles, pair directives, and space directives shape work blocks.".to_string()
        });
    draw_text(
        &style::truncate_text(&footer, 76),
        context.x + 18.0,
        context.y + 111.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    if let (Some(name), Some(detail)) = (hovered_name.clone(), hovered_directive) {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &name, &detail);
    } else if let (Some(name), Some(forecast)) = (hovered_name, hovered_forecast) {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &name, &forecast.detail);
    }
}

struct AssignPairAction {
    label: String,
    detail: String,
    directive: PairDirective,
}

fn assign_visible_colonists<'a>(
    colonists: &'a [Colonist],
    selected_colonist_id: Option<u32>,
) -> Vec<&'a Colonist> {
    let mut visible = Vec::new();

    if let Some(selected_id) = selected_colonist_id {
        if let Some(colonist) = colonists.iter().find(|colonist| colonist.id == selected_id) {
            visible.push(colonist);
        }
    }

    for colonist in colonists {
        if visible.iter().any(|visible| visible.id == colonist.id) {
            continue;
        }

        visible.push(colonist);
        if visible.len() >= 5 {
            break;
        }
    }

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

fn draw_log_context(
    context: Rect,
    logs: &[ColonyLogEntry],
    social_history: &[SocialHistoryEntry],
    summary: &ColonyPressureSummary,
) {
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

    if let Some(history) = social_history.last() {
        draw_text(
            &style::truncate_text(&format!("Day {}: {}", history.day, history.title), 54),
            context.x + 18.0,
            context.y + 87.0,
            style::TINY_SIZE,
            style::HEADING_BLUE,
        );
        draw_text(
            &style::truncate_text(&history.recommendation, 72),
            context.x + 18.0,
            context.y + 104.0,
            style::TINY_SIZE,
            style::TEXT_MUTED,
        );
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
        let visible = assign_visible_colonists(&colonists, Some(5))
            .into_iter()
            .map(|colonist| colonist.id)
            .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 0, 1, 2, 3]);
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
}
