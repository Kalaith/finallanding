use crate::ui::font::draw_text;
use macroquad::prelude::*;
use macroquad_toolkit::input::is_hovered_rect;
use macroquad_toolkit::ui::{draw_surface, SurfaceStyle};

pub const PANEL_BG: Color = Color::new(0.075, 0.095, 0.1, 0.9);
pub const PANEL_BG_DEEP: Color = Color::new(0.04, 0.055, 0.06, 0.94);
pub const PANEL_BORDER: Color = Color::new(0.22, 0.3, 0.33, 0.82);
pub const PANEL_DIVIDER: Color = Color::new(0.14, 0.2, 0.22, 0.75);
pub const TEXT_PRIMARY: Color = Color::new(0.93, 0.925, 0.91, 1.0);
pub const TEXT_BODY: Color = Color::new(0.63, 0.64, 0.63, 1.0);
pub const TEXT_MUTED: Color = Color::new(0.55, 0.55, 0.54, 1.0);
pub const HEADING_BLUE: Color = Color::new(0.57, 0.69, 0.71, 1.0);
pub const ACCENT_GOLD: Color = Color::new(0.68, 0.53, 0.24, 1.0);
pub const ACCENT_BLUE: Color = Color::new(0.31, 0.44, 0.56, 1.0);
pub const ALERT_RED: Color = Color::new(0.66, 0.27, 0.22, 1.0);
pub const BAR_GREEN: Color = Color::new(0.35, 0.56, 0.26, 1.0);
pub const BAR_GOLD: Color = Color::new(0.68, 0.53, 0.24, 1.0);
pub const BAR_RED: Color = Color::new(0.61, 0.21, 0.18, 1.0);
pub const BAR_CYAN: Color = Color::new(0.25, 0.54, 0.67, 1.0);

pub const TITLE_SIZE: f32 = 22.0;
pub const SECTION_SIZE: f32 = 13.0;
pub const BODY_SIZE: f32 = 14.0;
pub const SMALL_SIZE: f32 = 12.0;
pub const TINY_SIZE: f32 = 10.0;

pub fn draw_panel(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_BG);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, PANEL_BORDER);
    draw_rectangle(
        rect.x + 1.0,
        rect.y + 1.0,
        rect.w - 2.0,
        32.0_f32.min(rect.h - 2.0),
        Color::new(0.08, 0.115, 0.13, 0.5),
    );
}

pub fn draw_deep_panel(rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, PANEL_BG_DEEP);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, PANEL_BORDER);
}

pub fn draw_section_title(text: &str, x: f32, y: f32) {
    draw_text(text, x, y, SECTION_SIZE, HEADING_BLUE);
}

pub fn button_hovered(rect: Rect) -> bool {
    is_hovered_rect(rect)
}

pub fn draw_button(rect: Rect, active: bool, hovered: bool) {
    let color = if active {
        Color::new(0.12, 0.22, 0.25, 0.98)
    } else if hovered {
        Color::new(0.12, 0.16, 0.18, 0.98)
    } else {
        Color::new(0.075, 0.095, 0.105, 0.95)
    };
    let surface = SurfaceStyle::new(color).with_border(
        if active { 2.0 } else { 1.0 },
        if active { HEADING_BLUE } else { PANEL_BORDER },
    );
    draw_surface(rect, &surface);
}

pub fn draw_progress_bar(rect: Rect, value: f32, fill: Color) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.09, 0.11, 0.11, 1.0),
    );
    draw_rectangle(rect.x, rect.y, rect.w * value.clamp(0.0, 1.0), rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, PANEL_DIVIDER);
}

pub fn truncate_text(text: &str, max_chars: usize) -> String {
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
