use super::Layout;
use crate::data::building::BuildingType;
use crate::data::colonist::Colonist;
use crate::data::game_state::GameState;
use crate::data::resources::ResourceState;
use crate::systems::summary_system::ColonyPressureSummary;
use crate::ui::art::PlaceholderArt;
use crate::ui::style;
use macroquad::prelude::*;

pub fn draw_right_rail(
    layout: &Layout,
    state: &GameState,
    storage_capacity: i32,
    daily_supply_need: i32,
    colony_summary: &ColonyPressureSummary,
    art: &PlaceholderArt,
) {
    let rail = layout.right_panel();
    let minimap = Rect::new(rail.x, rail.y, rail.w, 156.0);
    draw_minimap(minimap, state);

    let resources = Rect::new(rail.x, minimap.y + minimap.h + 12.0, rail.w, 206.0);
    draw_resources(
        resources,
        &state.resources,
        storage_capacity,
        daily_supply_need,
    );

    let colonists = Rect::new(
        rail.x,
        resources.y + resources.h + 12.0,
        rail.w,
        rail.y + rail.h - resources.y - resources.h - 12.0,
    );
    draw_colonist_list(colonists, &state.colonists, colony_summary, art);
}

fn draw_minimap(rect: Rect, state: &GameState) {
    style::draw_panel(rect);
    style::draw_section_title("LOCAL MAP", rect.x + 16.0, rect.y + 29.0);

    let map = Rect::new(rect.x + 14.0, rect.y + 42.0, rect.w - 28.0, rect.h - 56.0);
    draw_rectangle(map.x, map.y, map.w, map.h, Color::new(0.13, 0.16, 0.1, 1.0));
    draw_rectangle_lines(map.x, map.y, map.w, map.h, 1.0, style::PANEL_BORDER);

    let cell_w = map.w / state.grid.width as f32;
    let cell_h = map.h / state.grid.height as f32;
    for building in state.building_system.buildings() {
        let color = building_color(building.building_type);
        let (w, h) = building.size();
        draw_rectangle(
            map.x + building.position.x as f32 * cell_w,
            map.y + building.position.y as f32 * cell_h,
            w as f32 * cell_w,
            h as f32 * cell_h,
            color,
        );
    }

    for colonist in &state.colonists {
        if colonist.is_on_mission() {
            continue;
        }
        draw_circle(
            map.x + colonist.position.x as f32 * cell_w,
            map.y + colonist.position.y as f32 * cell_h,
            2.5,
            style::TEXT_PRIMARY,
        );
    }
}

fn draw_resources(rect: Rect, resources: &ResourceState, storage_capacity: i32, daily_need: i32) {
    style::draw_panel(rect);
    style::draw_section_title("RESOURCES", rect.x + 16.0, rect.y + 29.0);
    let rows = [
        (
            "Food",
            resources.supplies,
            storage_capacity,
            style::BAR_GOLD,
        ),
        ("Salvage", resources.salvage, 220, style::TEXT_BODY),
        ("Metal", resources.salvage / 2, 120, style::TEXT_MUTED),
        (
            "Plastic",
            resources.prepared_meals + 12,
            80,
            style::BAR_CYAN,
        ),
        (
            "Fabric",
            resources.prepared_meals + 6,
            60,
            style::TEXT_MUTED,
        ),
        (
            "Fuel",
            resources.exploration_progress as i32 + 8,
            40,
            style::ACCENT_GOLD,
        ),
    ];

    for (index, (label, value, cap, color)) in rows.iter().enumerate() {
        let y = rect.y + 58.0 + index as f32 * 23.0;
        draw_circle(rect.x + 22.0, y - 4.0, 4.0, *color);
        draw_text(label, rect.x + 36.0, y, style::SMALL_SIZE, style::TEXT_BODY);
        let value_text = if *label == "Food" {
            format!("{} / {}", value, daily_need)
        } else {
            value.to_string()
        };
        let width = measure_text(&value_text, None, style::SMALL_SIZE as u16, 1.0).width;
        draw_text(
            &value_text,
            rect.x + rect.w - width - 16.0,
            y,
            style::SMALL_SIZE,
            if *value < (*cap / 5).max(1) {
                style::ALERT_RED
            } else {
                style::TEXT_PRIMARY
            },
        );
        draw_line(
            rect.x + 16.0,
            y + 10.0,
            rect.x + rect.w - 16.0,
            y + 10.0,
            1.0,
            style::PANEL_DIVIDER,
        );
    }
}

fn draw_colonist_list(
    rect: Rect,
    colonists: &[Colonist],
    summary: &ColonyPressureSummary,
    art: &PlaceholderArt,
) {
    style::draw_panel(rect);
    let capacity = 10;
    style::draw_section_title("COLONISTS", rect.x + 16.0, rect.y + 29.0);
    let count_label = format!("{} / {}", colonists.len(), capacity);
    let count_width = measure_text(&count_label, None, style::SMALL_SIZE as u16, 1.0).width;
    draw_text(
        &count_label,
        rect.x + rect.w - count_width - 16.0,
        rect.y + 29.0,
        style::SMALL_SIZE,
        style::ACCENT_GOLD,
    );

    for (index, colonist) in colonists.iter().take(7).enumerate() {
        let y = rect.y + 59.0 + index as f32 * 33.0;
        let portrait = Rect::new(rect.x + 16.0, y - 22.0, 25.0, 25.0);
        if let Some(texture) = art.colonist_portrait(colonist.id) {
            draw_texture_ex(
                texture,
                portrait.x,
                portrait.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(portrait.w, portrait.h)),
                    ..Default::default()
                },
            );
        } else {
            draw_circle(
                portrait.x + portrait.w * 0.5,
                portrait.y + portrait.h * 0.5,
                9.0,
                Color::new(0.45, 0.34, 0.25, 1.0),
            );
        }
        draw_rectangle_lines(
            portrait.x,
            portrait.y,
            portrait.w,
            portrait.h,
            1.0,
            style::PANEL_BORDER,
        );
        draw_text(
            &style::truncate_text(&colonist.name, 18),
            rect.x + 48.0,
            y,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        let mood = mood_face(colonist.mood);
        let mood_width = measure_text(mood, None, style::BODY_SIZE as u16, 1.0).width;
        draw_text(
            mood,
            rect.x + rect.w - mood_width - 17.0,
            y,
            style::BODY_SIZE,
            mood_color(colonist.mood),
        );
        draw_line(
            rect.x + 16.0,
            y + 10.0,
            rect.x + rect.w - 16.0,
            y + 10.0,
            1.0,
            style::PANEL_DIVIDER,
        );
    }

    let footer = format!(
        "Mood {:.0} | close {} | tense {}",
        summary.average_mood, summary.close_pairs, summary.strained_pairs
    );
    draw_text(
        &style::truncate_text(&footer, 33),
        rect.x + 16.0,
        rect.y + rect.h - 18.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
}

fn building_color(building_type: BuildingType) -> Color {
    let (r, g, b) = building_type.color();
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 0.85)
}

fn mood_face(mood: f32) -> &'static str {
    if mood >= 65.0 {
        ":)"
    } else if mood >= 35.0 {
        ":|"
    } else {
        ":("
    }
}

fn mood_color(mood: f32) -> Color {
    if mood >= 65.0 {
        style::BAR_GREEN
    } else if mood >= 35.0 {
        style::BAR_GOLD
    } else {
        style::ALERT_RED
    }
}
