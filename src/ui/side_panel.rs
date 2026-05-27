//! Side panel UI component - building selection and info

use super::Layout;
use crate::data::building::BuildingType;
use crate::data::event_log::{ColonyLogEntry, LogCategory};
use crate::data::resources::ResourceState;
use crate::data::scenario::ScenarioOutcome;
use crate::data::technology::{TechId, TechnologyState};
use crate::systems::mission_system::MissionPlan;
use crate::systems::summary_system::{ColonyPressureSummary, RelationshipPairSummary};
use crate::ui::font::{draw_text, measure_text};
use crate::ui::hit_zones::{build_button_rect, mission_button_rect, undo_button_rect};
use crate::ui::style;
use macroquad::prelude::*;

/// Result of drawing the side panel
pub struct SidePanelResult {
    pub selected_building: Option<BuildingType>,
    pub undo_requested: bool,
}

/// Draw the side panel with building selection
pub fn draw_side_panel(
    layout: &Layout,
    current_selection: Option<BuildingType>,
    building_count: usize,
    colonist_count: usize,
    resources: &ResourceState,
    storage_capacity: i32,
    daily_supply_need: i32,
    objective: &str,
    outcome: ScenarioOutcome,
    active_mission_count: usize,
    mission_plans: &[MissionPlan],
    technology: &TechnologyState,
    colony_summary: &ColonyPressureSummary,
    logs: &[ColonyLogEntry],
) -> SidePanelResult {
    let rect = layout.side_panel();

    style::draw_panel(rect);

    // Section: Buildings
    let section_y = rect.y + 10.0;
    style::draw_section_title("BUILDINGS", rect.x + 15.0, section_y + 20.0);
    draw_line(
        rect.x + 10.0,
        section_y + 30.0,
        rect.x + rect.w - 10.0,
        section_y + 30.0,
        1.0,
        GRAY,
    );

    let buildings = [
        (BuildingType::Habitat, "Q", "🏠"),
        (BuildingType::MessHall, "W", "🍽"),
        (BuildingType::Workshop, "E", "🔧"),
        (BuildingType::Storage, "R", "📦"),
        (BuildingType::ExplorationGate, "T", "🚀"),
    ];

    let mut result = SidePanelResult {
        selected_building: current_selection,
        undo_requested: false,
    };

    for (i, (building_type, key, icon)) in buildings.iter().enumerate() {
        let button_rect = build_button_rect(rect, i);

        let is_selected = current_selection == Some(*building_type);
        let (r, g, b) = building_type.color();
        let building_color = Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);

        // Hover effect
        let (mx, my) = mouse_position();
        let is_hovered = button_rect.contains(Vec2::new(mx, my));
        style::draw_button(button_rect, is_selected, is_hovered);

        // Color swatch
        draw_rectangle(
            button_rect.x + 5.0,
            button_rect.y + 5.0,
            18.0,
            18.0,
            building_color,
        );
        draw_rectangle_lines(
            button_rect.x + 5.0,
            button_rect.y + 5.0,
            18.0,
            18.0,
            1.0,
            WHITE,
        );

        // Label
        let label = format!("{} [{}] {}", icon, key, building_type.name());
        let can_afford = resources.salvage >= building_type.salvage_cost();
        let label_color = if can_afford { WHITE } else { GRAY };
        draw_text(
            &label,
            button_rect.x + 30.0,
            button_rect.y + 19.0,
            13.0,
            label_color,
        );

        let cost_label = format!("{}", building_type.salvage_cost());
        let cost_width = measure_text(&cost_label, None, 13, 1.0).width;
        let cost_color = if can_afford { LIGHTGRAY } else { RED };
        draw_text(
            &cost_label,
            button_rect.x + button_rect.w - cost_width - 8.0,
            button_rect.y + 19.0,
            13.0,
            cost_color,
        );

        // Click detection
        if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
            if is_selected {
                result.selected_building = None; // Deselect
            } else {
                result.selected_building = Some(*building_type);
            }
        }
    }

    // Undo button
    let undo_rect = undo_button_rect(rect);

    let (mx, my) = mouse_position();
    let undo_hovered = undo_rect.contains(Vec2::new(mx, my));
    let undo_bg = if undo_hovered {
        Color::new(0.4, 0.3, 0.3, 1.0)
    } else {
        Color::new(0.075, 0.095, 0.105, 0.95)
    };

    draw_rectangle(undo_rect.x, undo_rect.y, undo_rect.w, undo_rect.h, undo_bg);
    draw_rectangle_lines(
        undo_rect.x,
        undo_rect.y,
        undo_rect.w,
        undo_rect.h,
        1.0,
        style::PANEL_BORDER,
    );
    draw_text(
        "↩ [Z] Undo Last",
        undo_rect.x + 10.0,
        undo_rect.y + 19.0,
        14.0,
        LIGHTGRAY,
    );

    if undo_hovered && is_mouse_button_pressed(MouseButton::Left) {
        result.undo_requested = true;
    }

    let first_mission_rect = mission_button_rect(rect, 0);
    let mission_y = first_mission_rect.y - 12.0;
    style::draw_section_title("MISSIONS", rect.x + 15.0, mission_y);
    let cooldown_remaining = mission_plans
        .first()
        .map(|plan| plan.cooldown_remaining)
        .unwrap_or(0);
    let ready_color = if cooldown_remaining > 0 { ORANGE } else { GRAY };

    for (index, plan) in mission_plans.iter().enumerate() {
        let mission_rect = mission_button_rect(rect, index);
        let mission_hovered = mission_rect.contains(Vec2::new(mx, my));
        let mission_bg = if plan.recommended {
            Color::new(0.18, 0.34, 0.28, 1.0)
        } else if mission_hovered {
            Color::new(0.25, 0.3, 0.4, 1.0)
        } else {
            Color::new(0.075, 0.095, 0.105, 0.95)
        };
        draw_rectangle(
            mission_rect.x,
            mission_rect.y,
            mission_rect.w,
            mission_rect.h,
            mission_bg,
        );
        draw_rectangle_lines(
            mission_rect.x,
            mission_rect.y,
            mission_rect.w,
            mission_rect.h,
            if plan.recommended { 2.0 } else { 1.0 },
            if plan.recommended {
                style::BAR_GREEN
            } else {
                style::PANEL_BORDER
            },
        );

        let marker = if plan.recommended { "REC" } else { "   " };
        draw_text(
            &format!(
                "{} {} {}m R{}%",
                marker,
                plan.definition.short_name,
                plan.definition.duration_minutes,
                plan.danger_percent
            ),
            mission_rect.x + 7.0,
            mission_rect.y + 17.0,
            12.0,
            LIGHTGRAY,
        );
        draw_text(
            &truncate_text(plan.definition.reward_profile, 18),
            mission_rect.x + 117.0,
            mission_rect.y + 17.0,
            10.0,
            GRAY,
        );
    }

    let mission_tail_y = mission_plans
        .len()
        .checked_sub(1)
        .map(|index| mission_button_rect(rect, index).y + mission_button_rect(rect, index).h)
        .unwrap_or(first_mission_rect.y);
    draw_text(
        &format!(
            "Away {} | {}",
            active_mission_count,
            if cooldown_remaining > 0 {
                format!("Regroup {}m", cooldown_remaining)
            } else {
                "M launches rec".to_string()
            }
        ),
        rect.x + 15.0,
        mission_tail_y + 15.0,
        11.0,
        ready_color,
    );
    if let Some(recommended) = mission_plans.iter().find(|plan| plan.recommended) {
        draw_text(
            &truncate_text(&recommended.recommendation_reason, 34),
            rect.x + 15.0,
            mission_tail_y + 29.0,
            10.0,
            GRAY,
        );
    }

    // Objective section
    let objective_y = mission_tail_y + 43.0;
    draw_text("Objective", rect.x + 15.0, objective_y, 18.0, WHITE);
    draw_text(
        &truncate_text(objective, 32),
        rect.x + 15.0,
        objective_y + 22.0,
        12.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Run: {}", outcome.label()),
        rect.x + 15.0,
        objective_y + 40.0,
        12.0,
        scenario_color(outcome),
    );

    // Colony section
    let stats_y = objective_y + 55.0;
    draw_text("Colony", rect.x + 15.0, stats_y, 18.0, WHITE);
    draw_line(
        rect.x + 10.0,
        stats_y + 10.0,
        rect.x + rect.w - 10.0,
        stats_y + 10.0,
        1.0,
        GRAY,
    );

    draw_text(
        &format!(
            "Buildings {}  Colonists {}  {}",
            building_count,
            colonist_count,
            resources.condition.label()
        ),
        rect.x + 15.0,
        stats_y + 29.0,
        13.0,
        condition_color(resources.condition.label()),
    );
    draw_text(
        &format!(
            "Supplies {}/{}  Need {}",
            resources.supplies, storage_capacity, daily_supply_need
        ),
        rect.x + 15.0,
        stats_y + 45.0,
        13.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!(
            "Salvage {}  Meals {}  Tech {}/{}",
            resources.salvage,
            resources.prepared_meals,
            technology.unlocked_count(),
            TechId::all().len()
        ),
        rect.x + 15.0,
        stats_y + 61.0,
        13.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!(
            "Mood {:.0}  Rel {:+.0}  C{} T{}",
            colony_summary.average_mood,
            colony_summary.average_relationship,
            colony_summary.close_pairs,
            colony_summary.strained_pairs
        ),
        rect.x + 15.0,
        stats_y + 79.0,
        13.0,
        LIGHTGRAY,
    );
    draw_text(
        &truncate_text(
            &watchlist_text("Connected", &colony_summary.connected_pairs),
            32,
        ),
        rect.x + 15.0,
        stats_y + 95.0,
        12.0,
        relationship_color(colony_summary.average_relationship),
    );
    draw_text(
        &truncate_text(&watchlist_text("Tense", &colony_summary.tense_pairs), 32),
        rect.x + 15.0,
        stats_y + 111.0,
        12.0,
        GRAY,
    );
    draw_text(
        &format!(
            "Next tech: {}",
            next_tech_label(technology.next_locked_tech())
        ),
        rect.x + 15.0,
        stats_y + 127.0,
        11.0,
        GRAY,
    );

    let log_y = (stats_y + 145.0).min(rect.y + rect.h - 82.0);
    draw_text("Colony Log", rect.x + 15.0, log_y, 18.0, WHITE);
    draw_line(
        rect.x + 10.0,
        log_y + 10.0,
        rect.x + rect.w - 10.0,
        log_y + 10.0,
        1.0,
        GRAY,
    );

    let visible_logs = logs.iter().rev().take(1).collect::<Vec<_>>();
    if visible_logs.is_empty() {
        draw_text(
            "No notable events yet",
            rect.x + 15.0,
            log_y + 34.0,
            12.0,
            GRAY,
        );
    } else {
        for (i, log) in visible_logs.iter().enumerate() {
            let y = log_y + 31.0 + i as f32 * 42.0;
            let prefix = category_prefix(log.category);
            let title = truncate_text(&log.title, 24);
            draw_text(
                &format!("{} {:02}:{:02} {}", prefix, log.hour, log.minute, title),
                rect.x + 15.0,
                y,
                11.0,
                LIGHTGRAY,
            );
            for (line_index, detail_line) in wrapped_lines(&log.detail, 31, 2).iter().enumerate() {
                draw_text(
                    detail_line,
                    rect.x + 15.0,
                    y + 14.0 + line_index as f32 * 12.0,
                    10.0,
                    GRAY,
                );
            }
        }
    }

    result
}

fn category_prefix(category: LogCategory) -> &'static str {
    match category {
        LogCategory::Time => "T",
        LogCategory::Social => "S",
        LogCategory::Work => "W",
        LogCategory::Mood => "M",
        LogCategory::Resource => "R",
        LogCategory::Mission => "E",
        LogCategory::Technology => "K",
        LogCategory::Colony => "C",
        LogCategory::System => "I",
    }
}

fn next_tech_label(tech_id: Option<TechId>) -> &'static str {
    tech_id.map(|tech| tech.name()).unwrap_or("Complete")
}

fn condition_color(label: &str) -> Color {
    match label {
        "Stable" => GREEN,
        "Strained" => YELLOW,
        "Critical" => ORANGE,
        "Collapsed" => RED,
        _ => LIGHTGRAY,
    }
}

fn scenario_color(outcome: ScenarioOutcome) -> Color {
    match outcome {
        ScenarioOutcome::InProgress => LIGHTGRAY,
        ScenarioOutcome::Victory => GREEN,
        ScenarioOutcome::Failure => RED,
    }
}

fn relationship_color(value: f32) -> Color {
    if value >= 8.0 {
        GREEN
    } else if value <= -8.0 {
        ORANGE
    } else {
        LIGHTGRAY
    }
}

fn watchlist_text(prefix: &str, pairs: &[RelationshipPairSummary]) -> String {
    pairs
        .first()
        .map(|pair| {
            format!(
                "{}: {}/{} {} ({:+})",
                prefix, pair.first_name, pair.second_name, pair.label, pair.value
            )
        })
        .unwrap_or_else(|| format!("{}: none yet", prefix))
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

fn wrapped_lines(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let projected_len = if current.is_empty() {
            word.chars().count()
        } else {
            current.chars().count() + 1 + word.chars().count()
        };

        if projected_len > max_chars && !current.is_empty() {
            lines.push(current);
            current = word.to_string();

            if lines.len() == max_lines {
                break;
            }
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if lines.len() < max_lines && !current.is_empty() {
        lines.push(current);
    }

    if lines.len() == max_lines && text.chars().count() > lines.join(" ").chars().count() {
        if let Some(last) = lines.last_mut() {
            *last = truncate_text(last, max_chars);
        }
    }

    lines
}
