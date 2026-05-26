use super::Layout;
use crate::systems::advisor_system::{AdvisorPlan, AdvisorSeverity};
use macroquad::prelude::*;

pub fn draw_advisor_overlay(layout: &Layout, plan: &AdvisorPlan) {
    let game_area = layout.game_area();
    let width = 360.0;
    let height = 112.0;
    let x = game_area.x + 14.0;
    let y = game_area.y + game_area.h - height - 14.0;

    draw_rectangle(x, y, width, height, Color::new(0.04, 0.05, 0.06, 0.86));
    draw_rectangle_lines(x, y, width, height, 1.0, Color::new(0.45, 0.5, 0.55, 0.85));

    draw_text(
        &truncate(&plan.headline, 40),
        x + 12.0,
        y + 22.0,
        15.0,
        WHITE,
    );
    draw_line(
        x + 10.0,
        y + 30.0,
        x + width - 10.0,
        y + 30.0,
        1.0,
        Color::new(0.35, 0.38, 0.42, 0.8),
    );

    for (index, line) in plan.lines.iter().take(3).enumerate() {
        let row_y = y + 49.0 + index as f32 * 22.0;
        draw_rectangle(
            x + 12.0,
            row_y - 8.0,
            7.0,
            7.0,
            severity_color(line.severity),
        );
        draw_text(&truncate(&line.title, 28), x + 26.0, row_y, 12.0, LIGHTGRAY);
        draw_text(
            &truncate(&line.detail, 45),
            x + 128.0,
            row_y,
            11.0,
            Color::new(0.66, 0.69, 0.72, 1.0),
        );
    }
}

fn severity_color(severity: AdvisorSeverity) -> Color {
    match severity {
        AdvisorSeverity::Stable => GREEN,
        AdvisorSeverity::Action => YELLOW,
        AdvisorSeverity::Warning => ORANGE,
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
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
