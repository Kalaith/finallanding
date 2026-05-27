//! Top bar UI component - time display and speed controls

use super::Layout;
use crate::data::game_state::TimeSpeed;
use crate::data::priority::ColonyPriority;
use crate::data::resources::ResourceState;
use crate::systems::time_system::TimeSystem;
use crate::ui::hit_zones::{
    priority_button_rect, speed_button_rect, top_bar_speed_at, BUTTON_GAP, PRIORITY_BUTTON_START_X,
    PRIORITY_BUTTON_W, PRIORITY_LABEL_X,
};
use crate::ui::style;
use macroquad::prelude::*;

/// Draw the top bar with time and speed controls
/// Returns the new TimeSpeed if changed, None otherwise
pub fn draw_top_bar(
    layout: &Layout,
    tick: u64,
    current_speed: TimeSpeed,
    colonist_count: usize,
    average_mood: f32,
    resources: &ResourceState,
    current_priority: ColonyPriority,
) -> Option<TimeSpeed> {
    let rect = layout.top_bar();

    style::draw_deep_panel(Rect::new(12.0, 12.0, rect.w.min(840.0), rect.h - 16.0));

    // Title
    draw_text(
        "THE FINAL LANDING",
        30.0,
        42.0,
        style::TITLE_SIZE,
        style::TEXT_PRIMARY,
    );

    // Time display
    let (day, hour, minute) = TimeSystem::get_time_of_day(tick);
    let time_str = format!("Day {}, {:02}:{:02}", day, hour, minute);
    let is_night = TimeSystem::is_night(tick);
    let time_color = if is_night {
        Color::new(0.6, 0.7, 1.0, 1.0)
    } else {
        Color::new(1.0, 0.9, 0.6, 1.0)
    };
    let time_icon = if is_night { "MOON" } else { "SUN" };

    draw_text(
        &format!("{}  {}", time_str, time_icon),
        565.0,
        42.0,
        18.0,
        time_color,
    );

    // Speed controls
    let mut new_speed: Option<TimeSpeed> = None;
    let speeds = [
        (TimeSpeed::Paused, "II", "Pause"),
        (TimeSpeed::Normal, ">", "Normal"),
        (TimeSpeed::Fast, ">>", "Fast"),
        (TimeSpeed::SuperFast, ">>>", "Super"),
    ];

    for (i, (speed, label, _tooltip)) in speeds.iter().enumerate() {
        let button_rect = speed_button_rect(i);
        let is_active = current_speed == *speed;

        let (mx, my) = mouse_position();
        style::draw_button(
            button_rect,
            is_active,
            button_rect.contains(Vec2::new(mx, my)),
        );

        let text_w = measure_text(label, None, 16, 1.0).width;
        draw_text(
            label,
            button_rect.x + (button_rect.w - text_w) / 2.0,
            button_rect.y + 20.0,
            16.0,
            if is_active {
                style::HEADING_BLUE
            } else {
                style::TEXT_BODY
            },
        );

        if is_mouse_button_pressed(MouseButton::Left) {
            new_speed = top_bar_speed_at(mx, my);
        }
    }

    draw_text("PRIORITY", PRIORITY_LABEL_X, 24.0, 12.0, style::TEXT_MUTED);

    for (i, priority) in ColonyPriority::all().iter().enumerate() {
        let button_rect = priority_button_rect(i);
        let is_active = current_priority == *priority;
        let (mx, my) = mouse_position();
        style::draw_button(
            button_rect,
            is_active,
            button_rect.contains(Vec2::new(mx, my)),
        );

        let label = format!("[{}] {}", priority.shortcut(), priority.short_label());
        let text_w = measure_text(&label, None, 13, 1.0).width;
        draw_text(
            &label,
            button_rect.x + (button_rect.w - text_w) / 2.0,
            button_rect.y + 20.0,
            13.0,
            if is_active {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_BODY
            },
        );
    }

    let priority_end = PRIORITY_BUTTON_START_X
        + ColonyPriority::all().len() as f32 * (PRIORITY_BUTTON_W + BUTTON_GAP)
        - BUTTON_GAP;
    let status_label = format!(
        "C:{} Mood:{:.0} Supplies:{} Salvage:{} {}",
        colonist_count,
        average_mood,
        resources.supplies,
        resources.salvage,
        resources.condition.label()
    );
    let status_x = priority_end + 18.0;
    let status_width = measure_text(&status_label, None, 16, 1.0).width;
    if status_x + status_width <= rect.w - 10.0 {
        draw_text(&status_label, status_x, 42.0, 16.0, style::TEXT_BODY);
    } else {
        let compact_status = format!(
            "Mood:{:.0} S:{} {}",
            average_mood,
            resources.supplies,
            resources.condition.label()
        );
        let compact_width = measure_text(&compact_status, None, 14, 1.0).width;
        let compact_x = rect.w - compact_width - 10.0;
        if compact_x > priority_end + 10.0 {
            draw_text(&compact_status, compact_x, 42.0, 14.0, style::TEXT_BODY);
        }
    }

    new_speed
}
