use super::Layout;
use crate::systems::advisor_system::{AdvisorPlan, AdvisorSeverity};
use crate::ui::style;
use macroquad::prelude::*;

pub fn draw_advisor_overlay(layout: &Layout, objective: &str, plan: &AdvisorPlan) {
    let rail = layout.left_panel();
    let objective_rect = Rect::new(rail.x, rail.y, rail.w, 190.0);
    style::draw_panel(objective_rect);

    style::draw_section_title(
        "OBJECTIVES",
        objective_rect.x + 18.0,
        objective_rect.y + 31.0,
    );
    draw_objective_row(
        objective_rect.x + 20.0,
        objective_rect.y + 67.0,
        true,
        "Stabilize the colony",
    );
    draw_objective_row(
        objective_rect.x + 20.0,
        objective_rect.y + 97.0,
        false,
        &style::truncate_text(objective, 30),
    );
    draw_objective_row(
        objective_rect.x + 20.0,
        objective_rect.y + 127.0,
        false,
        "Respond to current pressure",
    );
    draw_objective_row(
        objective_rect.x + 20.0,
        objective_rect.y + 157.0,
        true,
        "Keep relationships readable",
    );

    let alert_y = objective_rect.y + objective_rect.h + 16.0;
    for (index, line) in plan.lines.iter().take(2).enumerate() {
        let row = Rect::new(rail.x, alert_y + index as f32 * 48.0, rail.w, 40.0);
        draw_rectangle(row.x, row.y, row.w, row.h, alert_bg_color(line.severity));
        draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, style::PANEL_BORDER);
        draw_rectangle(
            row.x + 14.0,
            row.y + 13.0,
            10.0,
            10.0,
            severity_color(line.severity),
        );
        draw_text(
            &style::truncate_text(&line.title, 25),
            row.x + 34.0,
            row.y + 17.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_text(
            &style::truncate_text(&line.detail, 35),
            row.x + 34.0,
            row.y + 33.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }
}

fn draw_objective_row(x: f32, y: f32, checked: bool, label: &str) {
    draw_rectangle_lines(x, y - 11.0, 12.0, 12.0, 1.0, style::TEXT_MUTED);
    if checked {
        draw_line(x + 3.0, y - 5.0, x + 6.0, y - 1.0, 2.0, style::BAR_GREEN);
        draw_line(x + 6.0, y - 1.0, x + 12.0, y - 10.0, 2.0, style::BAR_GREEN);
    }
    draw_text(label, x + 28.0, y, style::BODY_SIZE, style::TEXT_BODY);
}

fn alert_bg_color(severity: AdvisorSeverity) -> Color {
    match severity {
        AdvisorSeverity::Stable => Color::new(0.07, 0.12, 0.1, 0.88),
        AdvisorSeverity::Action => Color::new(0.13, 0.11, 0.06, 0.88),
        AdvisorSeverity::Warning => Color::new(0.16, 0.07, 0.055, 0.9),
    }
}

fn severity_color(severity: AdvisorSeverity) -> Color {
    match severity {
        AdvisorSeverity::Stable => style::BAR_GREEN,
        AdvisorSeverity::Action => style::BAR_GOLD,
        AdvisorSeverity::Warning => style::ALERT_RED,
    }
}
