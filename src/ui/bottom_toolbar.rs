use super::Layout;
use crate::data::building::BuildingType;
use crate::ui::style;
use macroquad::prelude::*;

pub fn draw_bottom_toolbar(layout: &Layout, selected_building: Option<BuildingType>) {
    let rect = layout.bottom_toolbar();
    style::draw_panel(rect);

    let tools = [
        ("Build", "B"),
        ("Rooms", "R"),
        ("Objects", "O"),
        ("Colony", "C"),
        ("Research", "T"),
        ("Assign", "A"),
        ("Log", "L"),
    ];

    let button_w = rect.w / tools.len() as f32;
    for (index, (label, icon)) in tools.iter().enumerate() {
        let button = Rect::new(
            rect.x + index as f32 * button_w,
            rect.y + 8.0,
            button_w,
            rect.h - 16.0,
        );
        let hovered = button.contains(mouse_position().into());
        let active = index == 0 && selected_building.is_some();
        style::draw_button(button, active, hovered);
        let icon_width = measure_text(icon, None, 21, 1.0).width;
        draw_text(
            icon,
            button.x + (button.w - icon_width) * 0.5,
            button.y + 24.0,
            21.0,
            style::HEADING_BLUE,
        );
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
        let helper = format!("Q/W/E/R/T place {} | Z undo | Esc cancel", building.name());
        let helper_width = measure_text(&helper, None, style::TINY_SIZE as u16, 1.0).width;
        draw_text(
            &helper,
            rect.x + (rect.w - helper_width) * 0.5,
            rect.y - 8.0,
            style::TINY_SIZE,
            style::TEXT_MUTED,
        );
    }
}
