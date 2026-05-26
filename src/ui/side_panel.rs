//! Side panel UI component - building selection and info

use super::Layout;
use crate::data::building::BuildingType;
use crate::data::event_log::{ColonyLogEntry, LogCategory};
use crate::data::resources::ResourceState;
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

    // Stats section
    let stats_y = undo_y + 50.0;
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
        &format!("Status: {}", resources.condition.label()),
        rect.x + 15.0,
        stats_y + 135.0,
        16.0,
        condition_color(resources.condition.label()),
    );

    let log_y = stats_y + 165.0;
    draw_text("Colony Log", rect.x + 15.0, log_y, 18.0, WHITE);
    draw_line(
        rect.x + 10.0,
        log_y + 10.0,
        rect.x + rect.w - 10.0,
        log_y + 10.0,
        1.0,
        GRAY,
    );

    let visible_logs = logs.iter().rev().take(3).collect::<Vec<_>>();
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
        LogCategory::Colony => "C",
        LogCategory::System => "I",
    }
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
