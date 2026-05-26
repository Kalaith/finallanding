//! Top bar UI component - time display and speed controls

use super::Layout;
use crate::data::game_state::TimeSpeed;
use crate::data::priority::ColonyPriority;
use crate::data::resources::ResourceState;
use crate::systems::time_system::TimeSystem;
use macroquad::prelude::*;
use macroquad_toolkit::colors::dark;
use macroquad_toolkit::ui::*;

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

    let surface = SurfaceStyle::new(dark::PANEL)
        .with_header(rect.h, dark::PANEL)
        .with_header_divider(2.0, dark::PANEL_HEADER);
    draw_surface(rect, &surface);

    // Title
    draw_text("The Final Landing", 15.0, 32.0, 24.0, WHITE);

    // Time display
    let (day, hour, minute) = TimeSystem::get_time_of_day(tick);
    let time_str = format!("Day {}, {:02}:{:02}", day, hour, minute);
    let is_night = TimeSystem::is_night(tick);
    let time_color = if is_night {
        Color::new(0.6, 0.7, 1.0, 1.0)
    } else {
        Color::new(1.0, 0.9, 0.6, 1.0)
    };
    let time_icon = if is_night { "☾" } else { "☀" };

    draw_text(
        &format!("{} {}", time_icon, time_str),
        220.0,
        32.0,
        22.0,
        time_color,
    );

    // Speed controls
    let mut new_speed: Option<TimeSpeed> = None;
    let btn_y = 10.0;
    let btn_h = 30.0;
    let btn_w = 50.0;
    let btn_start_x = 420.0;

    let speeds = [
        (TimeSpeed::Paused, "⏸", "Pause"),
        (TimeSpeed::Normal, "1x", "Normal"),
        (TimeSpeed::Fast, "2x", "Fast"),
        (TimeSpeed::SuperFast, "4x", "Super"),
    ];

    for (i, (speed, label, _tooltip)) in speeds.iter().enumerate() {
        let btn_x = btn_start_x + (i as f32 * (btn_w + 5.0));
        let is_active = current_speed == *speed;

        let bg_color = if is_active {
            dark::ACCENT
        } else {
            dark::PANEL_HEADER
        };

        let btn_surface = SurfaceStyle::new(bg_color).with_border(1.0, GRAY);
        draw_surface(Rect::new(btn_x, btn_y, btn_w, btn_h), &btn_surface);

        let text_w = measure_text(label, None, 16, 1.0).width;
        draw_text(
            label,
            btn_x + (btn_w - text_w) / 2.0,
            btn_y + 20.0,
            16.0,
            WHITE,
        );

        // Check for click
        let (mx, my) = mouse_position();
        if mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h {
            if is_mouse_button_pressed(MouseButton::Left) {
                new_speed = Some(*speed);
            }
        }
    }

    draw_text("Priority", 650.0, 30.0, 13.0, GRAY);
    let priority_btn_y = 10.0;
    let priority_btn_h = 30.0;
    let priority_btn_w = 68.0;
    let priority_start_x = 725.0;

    for (i, priority) in ColonyPriority::all().iter().enumerate() {
        let btn_x = priority_start_x + (i as f32 * (priority_btn_w + 5.0));
        let is_active = current_priority == *priority;
        let bg_color = if is_active {
            Color::new(0.25, 0.42, 0.34, 1.0)
        } else {
            dark::PANEL_HEADER
        };

        let btn_surface = SurfaceStyle::new(bg_color).with_border(1.0, GRAY);
        draw_surface(
            Rect::new(btn_x, priority_btn_y, priority_btn_w, priority_btn_h),
            &btn_surface,
        );

        let label = format!("[{}] {}", priority.shortcut(), priority.short_label());
        let text_w = measure_text(&label, None, 13, 1.0).width;
        draw_text(
            &label,
            btn_x + (priority_btn_w - text_w) / 2.0,
            priority_btn_y + 20.0,
            13.0,
            WHITE,
        );
    }

    let priority_end =
        priority_start_x + ColonyPriority::all().len() as f32 * (priority_btn_w + 5.0) - 5.0;
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
        draw_text(&status_label, status_x, 32.0, 16.0, LIGHTGRAY);
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
            draw_text(&compact_status, compact_x, 32.0, 14.0, LIGHTGRAY);
        }
    }

    new_speed
}
