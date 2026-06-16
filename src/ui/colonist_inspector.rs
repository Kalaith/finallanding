use super::Layout;
use crate::data::colonist::{
    relationship_label, ActivityLocation, Colonist, ColonistState, JobPreference,
};
use crate::data::schedule::ActivityType;
use crate::ui::art::PlaceholderArt;
use crate::ui::style;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub fn draw_colonist_inspector(
    layout: &Layout,
    colonist: Option<&Colonist>,
    colonists: &[Colonist],
    current_tick: u64,
    art: &PlaceholderArt,
) {
    let Some(colonist) = colonist else {
        return;
    };

    let rail = layout.left_panel();
    let width = rail.w;
    let height = 260.0_f32.min(rail.h * 0.48);
    let x = rail.x;
    let y = rail.y + rail.h - height;
    let accent = style::mood_color(colonist.mood);

    style::draw_panel(Rect::new(x, y, width, height));
    draw_rectangle(x, y, 4.0, height, accent);

    draw_ui_text(
        &style::truncate_text(&colonist.name.to_uppercase(), 25),
        x + 14.0,
        y + 31.0,
        18.0,
        style::TEXT_PRIMARY,
    );

    let portrait_rect = Rect::new(x + 18.0, y + 54.0, 76.0, 76.0);
    draw_rectangle(
        portrait_rect.x,
        portrait_rect.y,
        portrait_rect.w,
        portrait_rect.h,
        Color::new(0.09, 0.1, 0.095, 1.0),
    );
    if let Some(texture) = art.colonist_portrait(colonist.id) {
        draw_texture_ex(
            texture,
            portrait_rect.x,
            portrait_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(portrait_rect.w, portrait_rect.h)),
                ..Default::default()
            },
        );
    } else {
        draw_circle(
            portrait_rect.x + portrait_rect.w * 0.5,
            portrait_rect.y + 30.0,
            22.0,
            Color::new(0.74, 0.62, 0.49, 1.0),
        );
        draw_rectangle(
            portrait_rect.x + 21.0,
            portrait_rect.y + 50.0,
            34.0,
            20.0,
            job_color(colonist.job_preference),
        );
    }
    draw_rectangle_lines(
        portrait_rect.x,
        portrait_rect.y,
        portrait_rect.w,
        portrait_rect.h,
        1.0,
        style::PANEL_BORDER,
    );
    draw_ui_text(
        job_label(colonist.job_preference),
        x + 112.0,
        y + 74.0,
        style::BODY_SIZE,
        style::TEXT_BODY,
    );
    draw_ui_text(
        state_label(colonist.state),
        x + 112.0,
        y + 96.0,
        style::SMALL_SIZE,
        style::BAR_GREEN,
    );
    draw_ui_text(
        &format!(
            "{} at {}",
            activity_label(&colonist.current_activity),
            style::truncate_text(&activity_location_label(&colonist.activity_location), 17)
        ),
        x + 112.0,
        y + 116.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
    draw_ui_text(
        &format!("Injury: {}", injury_label(colonist, current_tick)),
        x + 112.0,
        y + 134.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    let bars_y = y + 146.0;
    draw_labeled_bar(
        x + 18.0,
        bars_y,
        "Mood",
        colonist.mood / 100.0,
        style::BAR_GREEN,
    );
    draw_labeled_bar(x + 18.0, bars_y + 23.0, "Energy", 0.46, style::BAR_GOLD);
    draw_labeled_bar(x + 18.0, bars_y + 46.0, "Hunger", 0.58, style::BAR_RED);
    draw_labeled_bar(
        x + 18.0,
        bars_y + 69.0,
        "Health",
        if colonist.is_hurt(current_tick) {
            0.45
        } else {
            0.82
        },
        style::BAR_CYAN,
    );

    let relationship = strongest_relationship(colonist, colonists)
        .map(|(name, value)| {
            format!(
                "{} {} ({:+})",
                style::truncate_text(&name, 16),
                relationship_label(value),
                value
            )
        })
        .unwrap_or_else(|| "No strong tie yet".to_string());

    draw_ui_text(
        &format!("RELATIONSHIP  {}", style::truncate_text(&relationship, 24)),
        x + 18.0,
        y + height - 13.0,
        style::TINY_SIZE,
        style::TEXT_BODY,
    );

    draw_relationship_portraits(x + 18.0, y + height - 58.0, colonist, art);
}

fn draw_relationship_portraits(x: f32, y: f32, colonist: &Colonist, art: &PlaceholderArt) {
    let mut relationships = colonist.relationships.iter().collect::<Vec<_>>();
    relationships.sort_by_key(|(_, value)| std::cmp::Reverse(value.abs()));

    for (index, (other_id, value)) in relationships.into_iter().take(5).enumerate() {
        let px = x + index as f32 * 43.0;
        let rect = Rect::new(px, y, 34.0, 34.0);
        if let Some(texture) = art.colonist_portrait(*other_id) {
            draw_texture_ex(
                texture,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, rect.h)),
                    ..Default::default()
                },
            );
        } else {
            draw_circle(rect.x + 17.0, rect.y + 17.0, 15.0, style::PANEL_BG_DEEP);
        }
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.0,
            if *value >= 0 {
                style::BAR_GREEN
            } else {
                style::ALERT_RED
            },
        );
        let value_text = format!("{:+}", value);
        let width = measure_ui_text(&value_text, None, 9, 1.0).width;
        draw_ui_text(
            &value_text,
            rect.x + (rect.w - width) * 0.5,
            rect.y + 44.0,
            9.0,
            if *value >= 0 {
                style::BAR_GREEN
            } else {
                style::ALERT_RED
            },
        );
    }

    if colonist.relationships.is_empty() {
        draw_ui_text(
            "No ties yet",
            x,
            y + 22.0,
            style::TINY_SIZE,
            style::TEXT_MUTED,
        );
    }
}

fn draw_labeled_bar(x: f32, y: f32, label: &str, value: f32, color: Color) {
    draw_ui_text(label, x, y + 9.0, style::TINY_SIZE, style::TEXT_BODY);
    style::draw_progress_bar(Rect::new(x + 72.0, y, 150.0, 9.0), value, color);
}

fn state_label(state: ColonistState) -> &'static str {
    match state {
        ColonistState::Idle => "Idle",
        ColonistState::Moving { .. } => "Moving",
        ColonistState::Working => "Working",
        ColonistState::Eating => "Eating",
        ColonistState::Sleeping => "Sleeping",
        ColonistState::OnMission { .. } => "Mission",
    }
}

fn job_label(job: JobPreference) -> &'static str {
    match job {
        JobPreference::Explorer => "Explorer",
        JobPreference::Builder => "Builder",
        JobPreference::Cook => "Cook",
        JobPreference::Hauler => "Hauler",
        JobPreference::None => "General",
    }
}

fn activity_label(activity: &ActivityType) -> &'static str {
    match activity {
        ActivityType::Sleep => "Sleep",
        ActivityType::Work => "Work",
        ActivityType::Relax => "Recover",
        ActivityType::Eat => "Meal",
    }
}

fn activity_location_label(location: &ActivityLocation) -> String {
    match location {
        ActivityLocation::None => "open ground".to_string(),
        ActivityLocation::Ground(pos) => format!("ground {},{}", pos.x, pos.y),
        ActivityLocation::Building {
            building_id,
            building_type,
        } => format!("{} #{}", building_type.name(), building_id),
    }
}

fn injury_label(colonist: &Colonist, current_tick: u64) -> String {
    colonist
        .recovery_minutes_remaining(current_tick)
        .map(|remaining| format!("recovering {}m", remaining))
        .unwrap_or_else(|| "clear".to_string())
}

fn strongest_relationship(colonist: &Colonist, colonists: &[Colonist]) -> Option<(String, i32)> {
    colonist
        .relationships
        .iter()
        .max_by_key(|(_, value)| value.abs())
        .map(|(other_id, value)| {
            let other_name = colonists
                .iter()
                .find(|candidate| candidate.id == *other_id)
                .map(|candidate| candidate.name.clone())
                .unwrap_or_else(|| format!("Colonist {}", other_id));
            (other_name, *value)
        })
}

fn job_color(job: JobPreference) -> Color {
    match job {
        JobPreference::Explorer => style::ACCENT_BLUE,
        JobPreference::Builder => style::BAR_GOLD,
        JobPreference::Cook => style::BAR_GREEN,
        JobPreference::Hauler => style::TEXT_MUTED,
        JobPreference::None => style::TEXT_BODY,
    }
}
