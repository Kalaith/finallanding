use super::Layout;
use crate::systems::advisor_system::{AdvisorPlan, AdvisorSeverity};
use crate::systems::objective_system::{ObjectiveCard, ObjectiveStatus};
use crate::ui::style;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_advisor_overlay(layout: &Layout, objectives: &[ObjectiveCard], plan: &AdvisorPlan) {
    let rail = layout.left_panel();
    let objective_rect = Rect::new(rail.x, rail.y, rail.w, 172.0);
    style::draw_panel(objective_rect);

    style::draw_section_title(
        "OBJECTIVES",
        objective_rect.x + 18.0,
        objective_rect.y + 31.0,
    );
    for (index, objective) in objectives.iter().take(4).enumerate() {
        draw_objective_card(
            objective_rect.x + 14.0,
            objective_rect.y + 50.0 + index as f32 * 29.0,
            objective_rect.w - 28.0,
            objective,
        );
    }

    let alert_y = objective_rect.y + objective_rect.h + 12.0;
    for (index, line) in plan.lines.iter().take(2).enumerate() {
        let row = Rect::new(rail.x, alert_y + index as f32 * 44.0, rail.w, 38.0);
        draw_rectangle(row.x, row.y, row.w, row.h, alert_bg_color(line.severity));
        draw_rectangle_lines(row.x, row.y, row.w, row.h, 1.0, style::PANEL_BORDER);
        draw_rectangle(
            row.x + 14.0,
            row.y + 12.0,
            10.0,
            10.0,
            severity_color(line.severity),
        );
        draw_ui_text(
            &style::truncate_text(&line.title, 25),
            row.x + 34.0,
            row.y + 16.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_ui_text(
            &style::truncate_text(&line.detail, 35),
            row.x + 34.0,
            row.y + 31.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }
}

fn draw_objective_card(x: f32, y: f32, width: f32, objective: &ObjectiveCard) {
    let status_color = status_color(objective.status);
    draw_rectangle(x, y, width, 24.0, Color::new(0.045, 0.06, 0.065, 0.74));
    draw_rectangle(x, y, 3.0, 24.0, status_color);
    draw_rectangle_lines(x, y, width, 24.0, 1.0, style::PANEL_DIVIDER);

    let box_x = x + 10.0;
    let box_y = y + 6.0;
    draw_rectangle_lines(box_x, box_y, 10.0, 10.0, 1.0, style::TEXT_MUTED);
    if objective.status == ObjectiveStatus::Complete {
        draw_line(
            box_x + 2.0,
            box_y + 5.0,
            box_x + 5.0,
            box_y + 9.0,
            2.0,
            style::BAR_GREEN,
        );
        draw_line(
            box_x + 5.0,
            box_y + 9.0,
            box_x + 11.0,
            box_y + 0.0,
            2.0,
            style::BAR_GREEN,
        );
    }

    draw_ui_text(
        &style::truncate_text(&objective.title, 23),
        x + 28.0,
        y + 11.0,
        style::TINY_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_ui_text(
        &style::truncate_text(&objective.detail, 29),
        x + 28.0,
        y + 22.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    style::draw_progress_bar(
        Rect::new(x + width - 73.0, y + 8.0, 58.0, 7.0),
        objective.progress,
        status_color,
    );
}

fn status_color(status: ObjectiveStatus) -> Color {
    match status {
        ObjectiveStatus::Complete => style::BAR_GREEN,
        ObjectiveStatus::Active => style::HEADING_BLUE,
        ObjectiveStatus::AtRisk => style::ALERT_RED,
    }
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
