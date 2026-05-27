use super::Layout;
use crate::data::building::BuildingType;
use crate::ui::hit_zones::{toolbar_button_rect, ToolbarMode};
use crate::ui::style;
use crate::ui::tooltip::draw_tooltip_near_mouse;
use macroquad::prelude::*;

pub fn draw_bottom_toolbar(
    layout: &Layout,
    active_mode: ToolbarMode,
    selected_building: Option<BuildingType>,
) {
    let rect = layout.bottom_toolbar();
    style::draw_panel(rect);
    let mut hovered_mode = None;

    for (index, mode) in ToolbarMode::all().iter().enumerate() {
        let button = toolbar_button_rect(rect, index);
        let hovered = button.contains(mouse_position().into());
        if hovered {
            hovered_mode = Some(*mode);
        }
        let active = active_mode == *mode;
        style::draw_button(button, active, hovered);
        let icon = mode.icon();
        let icon_width = measure_text(icon, None, 21, 1.0).width;
        draw_text(
            icon,
            button.x + (button.w - icon_width) * 0.5,
            button.y + 24.0,
            21.0,
            style::HEADING_BLUE,
        );
        let label = mode.label();
        let label_width = measure_text(label, None, style::SMALL_SIZE as u16, 1.0).width;
        draw_text(
            label,
            button.x + (button.w - label_width) * 0.5,
            button.y + 47.0,
            style::SMALL_SIZE,
            style::TEXT_BODY,
        );
    }

    if let Some(building) = selected_building {
        let helper = format!(
            "Placing {} | {} salvage | {}",
            building.name(),
            building.salvage_cost(),
            building.planning_role()
        );
        let helper_width = measure_text(&helper, None, style::TINY_SIZE as u16, 1.0).width;
        draw_text(
            &helper,
            rect.x + (rect.w - helper_width) * 0.5,
            rect.y - 8.0,
            style::TINY_SIZE,
            style::TEXT_MUTED,
        );
    }

    if let Some(mode) = hovered_mode {
        draw_tooltip_near_mouse(
            Rect::new(
                0.0,
                layout.top_bar_height,
                screen_width(),
                screen_height() - layout.top_bar_height,
            ),
            mode.label(),
            mode.tooltip(),
        );
    }
}
