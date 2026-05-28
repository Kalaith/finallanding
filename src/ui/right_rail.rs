use super::Layout;
use crate::data::building::BuildingType;
use crate::data::colonist::{relationship_label, Colonist};
use crate::data::game_state::GameState;
use crate::data::resources::ResourceState;
use crate::systems::summary_system::ColonyPressureSummary;
use crate::ui::art::PlaceholderArt;
use crate::ui::font::{draw_text, measure_text};
use crate::ui::style;
use macroquad::prelude::*;

struct ResourceRow {
    label: &'static str,
    value_text: String,
    detail: String,
    progress: f32,
    color: Color,
    alert: bool,
}

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
    for y in 0..state.grid.height {
        for x in 0..state.grid.width {
            if (x + y) % 2 == 0 {
                draw_rectangle(
                    map.x + x as f32 * cell_w,
                    map.y + y as f32 * cell_h,
                    cell_w,
                    cell_h,
                    Color::new(0.1, 0.13, 0.09, 0.5),
                );
            }
        }
    }

    for building in state.building_system.buildings() {
        let color = building_color(building.building_type);
        let (w, h) = building.size();
        let footprint = Rect::new(
            map.x + building.position.x as f32 * cell_w,
            map.y + building.position.y as f32 * cell_h,
            w as f32 * cell_w,
            h as f32 * cell_h,
        );
        draw_rectangle(footprint.x, footprint.y, footprint.w, footprint.h, color);
        draw_rectangle_lines(
            footprint.x,
            footprint.y,
            footprint.w,
            footprint.h,
            1.0,
            Color::new(0.88, 0.86, 0.74, 0.7),
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

    let mission_count = state.missions.active_count();
    if mission_count > 0 {
        draw_line(
            map.x + map.w * 0.64,
            map.y + map.h * 0.5,
            map.x + map.w - 10.0,
            map.y + 12.0,
            1.0,
            style::ACCENT_GOLD,
        );
        draw_circle(map.x + map.w - 10.0, map.y + 12.0, 4.0, style::ACCENT_GOLD);
        draw_text(
            &format!("{} away", mission_count),
            map.x + 10.0,
            map.y + map.h - 8.0,
            style::TINY_SIZE,
            style::TEXT_MUTED,
        );
    }
}

fn draw_resources(rect: Rect, resources: &ResourceState, storage_capacity: i32, daily_need: i32) {
    style::draw_panel(rect);
    style::draw_section_title("RESOURCES", rect.x + 16.0, rect.y + 29.0);
    let rows = resource_rows(resources, storage_capacity, daily_need);

    for (index, row) in rows.iter().enumerate() {
        let y = rect.y + 54.0 + index as f32 * 25.0;
        draw_circle(rect.x + 22.0, y - 4.0, 4.0, row.color);
        draw_text(
            row.label,
            rect.x + 36.0,
            y,
            style::SMALL_SIZE,
            if row.alert {
                style::ALERT_RED
            } else {
                style::TEXT_BODY
            },
        );
        let width = measure_text(&row.value_text, None, style::SMALL_SIZE as u16, 1.0).width;
        draw_text(
            &row.value_text,
            rect.x + rect.w - width - 16.0,
            y,
            style::SMALL_SIZE,
            if row.alert {
                style::ALERT_RED
            } else {
                style::TEXT_PRIMARY
            },
        );
        style::draw_progress_bar(
            Rect::new(rect.x + 36.0, y + 6.0, rect.w - 98.0, 6.0),
            row.progress,
            row.color,
        );
        draw_text(
            &style::truncate_text(&row.detail, 22),
            rect.x + rect.w - 56.0,
            y + 12.0,
            style::TINY_SIZE,
            style::TEXT_MUTED,
        );
    }
}

fn resource_rows(
    resources: &ResourceState,
    storage_capacity: i32,
    daily_need: i32,
) -> Vec<ResourceRow> {
    let daily_need = daily_need.max(1);
    let food_days = resources.supplies as f32 / daily_need as f32;
    let food_alert = resources.supplies < daily_need * 2;
    vec![
        ResourceRow {
            label: "Food",
            value_text: format!("{}", resources.supplies),
            detail: format!("{:.1} days", food_days),
            progress: resources.supplies as f32 / storage_capacity.max(1) as f32,
            color: style::BAR_GOLD,
            alert: food_alert,
        },
        ResourceRow {
            label: "Salvage",
            value_text: format!("{}", resources.salvage),
            detail: "build stock".to_string(),
            progress: resources.salvage as f32 / 120.0,
            color: style::TEXT_BODY,
            alert: resources.salvage < 10,
        },
        ResourceRow {
            label: "Meals",
            value_text: format!("{}", resources.prepared_meals),
            detail: format!("-{} need", resources.prepared_meals.min(daily_need)),
            progress: resources.prepared_meals as f32 / daily_need as f32,
            color: style::BAR_GREEN,
            alert: false,
        },
        ResourceRow {
            label: "Survey",
            value_text: format!("{}", resources.exploration_progress),
            detail: "field work".to_string(),
            progress: resources.exploration_progress as f32 / 100.0,
            color: style::BAR_CYAN,
            alert: false,
        },
        ResourceRow {
            label: "Repair",
            value_text: format!("{}", resources.workshop_progress),
            detail: "workshop".to_string(),
            progress: resources.workshop_progress as f32 / 100.0,
            color: style::ACCENT_GOLD,
            alert: false,
        },
        ResourceRow {
            label: "Hauling",
            value_text: format!("{}", resources.hauling_progress),
            detail: "storage".to_string(),
            progress: resources.hauling_progress as f32 / 100.0,
            color: style::TEXT_MUTED,
            alert: false,
        },
    ]
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
        if let Some(value) = strongest_relationship_value(colonist) {
            let chip = format!("{} {:+}", relationship_label(value), value);
            let chip_width = measure_text(&chip, None, style::TINY_SIZE as u16, 1.0).width;
            draw_text(
                &chip,
                rect.x + rect.w - chip_width - 42.0,
                y,
                style::TINY_SIZE,
                relationship_color(value),
            );
        }
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
    style::mood_face(mood)
}

fn mood_color(mood: f32) -> Color {
    style::mood_color(mood)
}

fn strongest_relationship_value(colonist: &Colonist) -> Option<i32> {
    colonist
        .relationships
        .values()
        .max_by_key(|value| value.abs())
        .copied()
}

fn relationship_color(value: i32) -> Color {
    style::relationship_color(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{JobPreference, Trait};
    use crate::data::types::Position;

    #[test]
    fn test_resource_rows_use_real_gameplay_tracks() {
        let mut resources = ResourceState::default();
        resources.supplies = 5;
        resources.prepared_meals = 2;
        resources.exploration_progress = 17;
        resources.workshop_progress = 23;
        resources.hauling_progress = 31;

        let rows = resource_rows(&resources, 40, 4);
        let labels = rows.iter().map(|row| row.label).collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec!["Food", "Salvage", "Meals", "Survey", "Repair", "Hauling"]
        );
        assert_eq!(rows[0].detail, "1.2 days");
        assert!(rows[0].alert);
        assert_eq!(rows[2].detail, "-2 need");
    }

    #[test]
    fn test_strongest_relationship_value_uses_absolute_pressure() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        colonist.relationships.insert(2, 18);
        colonist.relationships.insert(3, -31);

        assert_eq!(strongest_relationship_value(&colonist), Some(-31));
        assert_eq!(relationship_color(-31), style::ALERT_RED);
    }
}
