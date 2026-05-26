//! Side panel UI component - building selection and info

use super::Layout;
use crate::data::building::BuildingType;
use crate::data::event_log::{ColonyLogEntry, LogCategory};
use crate::data::resources::ResourceState;
use crate::data::scenario::ScenarioOutcome;
use crate::data::technology::{TechId, TechnologyState};
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

    let btn_start_y = section_y + 45.0;
    let btn_height = 32.0;
    let btn_padding = 4.0;

    for (i, (building_type, key, icon)) in buildings.iter().enumerate() {
        let btn_y = btn_start_y + i as f32 * (btn_height + btn_padding);
        let btn_x = rect.x + 10.0;
        let btn_w = rect.w - 20.0;

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
        let is_hovered =
            mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_height;
        let bg_color = if is_hovered && !is_selected {
            Color::new(bg_color.r + 0.1, bg_color.g + 0.1, bg_color.b + 0.1, 1.0)
        } else {
            bg_color
        };

        let mut button_surface = SurfaceStyle::new(bg_color);
        if is_selected {
            button_surface = button_surface.with_border(2.0, GREEN);
        }
        draw_surface(Rect::new(btn_x, btn_y, btn_w, btn_height), &button_surface);

        // Color swatch
        draw_rectangle(btn_x + 5.0, btn_y + 6.0, 20.0, 20.0, building_color);
        draw_rectangle_lines(btn_x + 5.0, btn_y + 6.0, 20.0, 20.0, 1.0, WHITE);

        // Label
        let label = format!("{} [{}] {}", icon, key, building_type.name());
        let can_afford = resources.salvage >= building_type.salvage_cost();
        let label_color = if can_afford { WHITE } else { GRAY };
        draw_text(&label, btn_x + 32.0, btn_y + 22.0, 14.0, label_color);

        let cost_label = format!("{}", building_type.salvage_cost());
        let cost_width = measure_text(&cost_label, None, 14, 1.0).width;
        let cost_color = if can_afford { LIGHTGRAY } else { RED };
        draw_text(
            &cost_label,
            btn_x + btn_w - cost_width - 8.0,
            btn_y + 22.0,
            14.0,
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
    let undo_y = btn_start_y + buildings.len() as f32 * (btn_height + btn_padding) + 10.0;
    let undo_x = rect.x + 10.0;
    let undo_w = rect.w - 20.0;

    let (mx, my) = mouse_position();
    let undo_hovered = mx >= undo_x && mx <= undo_x + undo_w && my >= undo_y && my <= undo_y + 28.0;
    let undo_bg = if undo_hovered {
        Color::new(0.4, 0.3, 0.3, 1.0)
    } else {
        dark::PANEL_HEADER
    };

    let undo_surface = SurfaceStyle::new(undo_bg);
    draw_surface(Rect::new(undo_x, undo_y, undo_w, 28.0), &undo_surface);
    draw_text(
        "↩ [Z] Undo Last",
        undo_x + 10.0,
        undo_y + 19.0,
        14.0,
        LIGHTGRAY,
    );

    if undo_hovered && is_mouse_button_pressed(MouseButton::Left) {
        result.undo_requested = true;
    }

    let mission_y = undo_y + 42.0;
    draw_text("Missions", rect.x + 15.0, mission_y, 18.0, WHITE);
    let mission_btn_y = mission_y + 12.0;
    let mission_hovered =
        mx >= undo_x && mx <= undo_x + undo_w && my >= mission_btn_y && my <= mission_btn_y + 28.0;
    let mission_bg = if mission_hovered {
        Color::new(0.25, 0.3, 0.4, 1.0)
    } else {
        dark::PANEL_HEADER
    };
    draw_surface(
        Rect::new(undo_x, mission_btn_y, undo_w, 28.0),
        &SurfaceStyle::new(mission_bg),
    );
    draw_text(
        &format!(
            "[M] 1m Scan  Risk {}%  Away {}",
            mission_danger_percent, active_mission_count
        ),
        undo_x + 8.0,
        mission_btn_y + 19.0,
        13.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Duration: {} minute", mission_duration_minutes),
        rect.x + 15.0,
        mission_btn_y + 44.0,
        11.0,
        GRAY,
    );

    // Objective section
    let objective_y = mission_btn_y + 68.0;
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

    // Stats section
    let stats_y = objective_y + 60.0;
    draw_text("📊 Stats", rect.x + 15.0, stats_y, 18.0, WHITE);
    draw_line(
        rect.x + 10.0,
        stats_y + 10.0,
        rect.x + rect.w - 10.0,
        stats_y + 10.0,
        1.0,
        GRAY,
    );

    draw_text(
        &format!("Buildings: {}", building_count),
        rect.x + 15.0,
        stats_y + 35.0,
        16.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Colonists: {}", colonist_count),
        rect.x + 15.0,
        stats_y + 55.0,
        16.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Supplies: {}/{}", resources.supplies, storage_capacity),
        rect.x + 15.0,
        stats_y + 75.0,
        16.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Salvage: {}", resources.salvage),
        rect.x + 15.0,
        stats_y + 95.0,
        16.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Daily need: {}", daily_supply_need),
        rect.x + 15.0,
        stats_y + 115.0,
        16.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Meals ready: {}", resources.prepared_meals),
        rect.x + 15.0,
        stats_y + 135.0,
        16.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Status: {}", resources.condition.label()),
        rect.x + 15.0,
        stats_y + 155.0,
        16.0,
        condition_color(resources.condition.label()),
    );
    draw_text(
        &format!(
            "Tech: {}/{}",
            technology.unlocked_count(),
            TechId::all().len()
        ),
        rect.x + 15.0,
        stats_y + 175.0,
        16.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!("Next: {}", next_tech_label(technology.next_locked_tech())),
        rect.x + 15.0,
        stats_y + 195.0,
        13.0,
        GRAY,
    );

    let log_y = stats_y + 220.0;
    draw_text("Colony Log", rect.x + 15.0, log_y, 18.0, WHITE);
    draw_line(
        rect.x + 10.0,
        log_y + 10.0,
        rect.x + rect.w - 10.0,
        log_y + 10.0,
        1.0,
        GRAY,
    );

    let visible_logs = logs.iter().rev().take(2).collect::<Vec<_>>();
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
            let y = log_y + 32.0 + i as f32 * 30.0;
            let prefix = category_prefix(log.category);
            let title = truncate_text(&log.title, 24);
            let detail = truncate_text(&log.detail, 27);
            draw_text(
                &format!("{} {:02}:{:02} {}", prefix, log.hour, log.minute, title),
                rect.x + 15.0,
                y,
                11.0,
                LIGHTGRAY,
            );
            draw_text(&detail, rect.x + 15.0, y + 13.0, 10.0, GRAY);
        }
    }

    // Help section
    let help_y = rect.y + rect.h - 80.0;
    draw_line(
        rect.x + 10.0,
        help_y,
        rect.x + rect.w - 10.0,
        help_y,
        1.0,
        GRAY,
    );
    draw_text("Controls", rect.x + 15.0, help_y + 20.0, 16.0, WHITE);
    draw_text("[Space] Pause", rect.x + 15.0, help_y + 40.0, 12.0, GRAY);
    draw_text("[1/2/3] Speed", rect.x + 100.0, help_y + 40.0, 12.0, GRAY);
    draw_text(
        "[Esc] Cancel  [F3] Debug",
        rect.x + 15.0,
        help_y + 55.0,
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
