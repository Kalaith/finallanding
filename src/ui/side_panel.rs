//! Side panel UI component - building selection and info

use super::Layout;
use crate::data::building::BuildingType;
use crate::data::event_log::{ColonyLogEntry, LogCategory};
use crate::data::resources::ResourceState;
use crate::data::scenario::ScenarioOutcome;
use crate::data::technology::{TechId, TechnologyState};
use crate::systems::summary_system::{ColonyPressureSummary, RelationshipPairSummary};
use crate::ui::hit_zones::{build_button_rect, mission_button_rect, undo_button_rect};
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::{draw_surface, SurfaceStyle};

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
    mission_duration_minutes: u64,
    mission_danger_percent: u32,
    technology: &TechnologyState,
    colony_summary: &ColonyPressureSummary,
    logs: &[ColonyLogEntry],
) -> SidePanelResult {
    let rect = layout.side_panel();

    let surface = SurfaceStyle::new(dark::PANEL).with_left_accent(2.0, dark::PANEL_HEADER);
    draw_surface(rect, &surface);

    // Section: Buildings
    let section_y = rect.y + 10.0;
    draw_text("🏗 Buildings", rect.x + 15.0, section_y + 20.0, 20.0, WHITE);
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

        // Background
        let bg_color = if is_selected {
            Color::new(0.2, 0.4, 0.3, 1.0)
        } else {
            dark::PANEL_HEADER
        };

        // Hover effect
        let (mx, my) = mouse_position();
        let is_hovered = button_rect.contains(Vec2::new(mx, my));
        let bg_color = if is_hovered && !is_selected {
            Color::new(bg_color.r + 0.1, bg_color.g + 0.1, bg_color.b + 0.1, 1.0)
        } else {
            bg_color
        };

        let mut button_surface = SurfaceStyle::new(bg_color);
        if is_selected {
            button_surface = button_surface.with_border(2.0, GREEN);
        }
        draw_surface(button_rect, &button_surface);

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
        dark::PANEL_HEADER
    };

    let undo_surface = SurfaceStyle::new(undo_bg);
    draw_surface(undo_rect, &undo_surface);
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

    let mission_btn_rect = mission_button_rect(rect);
    let mission_y = mission_btn_rect.y - 12.0;
    draw_text("Missions", rect.x + 15.0, mission_y, 18.0, WHITE);
    let mission_hovered = mission_btn_rect.contains(Vec2::new(mx, my));
    let mission_bg = if mission_hovered {
        Color::new(0.25, 0.3, 0.4, 1.0)
    } else {
        dark::PANEL_HEADER
    };
    draw_surface(mission_btn_rect, &SurfaceStyle::new(mission_bg));
    draw_text(
        &format!(
            "[M] 1m Scan  Risk {}%  Away {}",
            mission_danger_percent, active_mission_count
        ),
        mission_btn_rect.x + 8.0,
        mission_btn_rect.y + 19.0,
        13.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Duration: {} minute", mission_duration_minutes),
        rect.x + 15.0,
        mission_btn_rect.y + 44.0,
        11.0,
        GRAY,
    );

    // Objective section
    let objective_y = mission_btn_rect.y + 62.0;
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
        &format!("Buildings {}  Colonists {}", building_count, colonist_count),
        rect.x + 15.0,
        stats_y + 29.0,
        13.0,
        LIGHTGRAY,
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
        &format!("Status {}", resources.condition.label()),
        rect.x + 15.0,
        stats_y + 77.0,
        13.0,
        condition_color(resources.condition.label()),
    );
    draw_text(
        &format!(
            "Mood {:.0}  Relations {:+.0}",
            colony_summary.average_mood, colony_summary.average_relationship
        ),
        rect.x + 15.0,
        stats_y + 96.0,
        13.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!(
            "Close {}  Tense {}",
            colony_summary.close_pairs, colony_summary.strained_pairs
        ),
        rect.x + 15.0,
        stats_y + 112.0,
        13.0,
        relationship_color(colony_summary.average_relationship),
    );
    draw_text(
        &truncate_text(
            &watchlist_text("Connected", &colony_summary.connected_pairs),
            32,
        ),
        rect.x + 15.0,
        stats_y + 128.0,
        12.0,
        LIGHTGRAY,
    );
    draw_text(
        &truncate_text(&watchlist_text("Tense", &colony_summary.tense_pairs), 32),
        rect.x + 15.0,
        stats_y + 144.0,
        12.0,
        GRAY,
    );
    draw_text(
        &format!(
            "Next tech: {}",
            next_tech_label(technology.next_locked_tech())
        ),
        rect.x + 15.0,
        stats_y + 160.0,
        11.0,
        GRAY,
    );

    let help_y = rect.y + rect.h - 28.0;
    let log_y = (stats_y + 174.0).min(help_y - 68.0);
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

    // Help section
    draw_line(
        rect.x + 10.0,
        help_y,
        rect.x + rect.w - 10.0,
        help_y,
        1.0,
        GRAY,
    );
    draw_text(
        "[Space] Pause  [1/2/3] Priority",
        rect.x + 15.0,
        help_y + 18.0,
        12.0,
        GRAY,
    );

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
