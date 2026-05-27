use super::Layout;
use crate::data::colonist::{
    relationship_label, ActivityLocation, Colonist, ColonistState, JobPreference,
};
use crate::data::schedule::ActivityType;
use macroquad::prelude::*;

pub fn draw_colonist_inspector(
    layout: &Layout,
    colonist: Option<&Colonist>,
    colonists: &[Colonist],
    current_tick: u64,
) {
    let Some(colonist) = colonist else {
        return;
    };

    let game_area = layout.game_area();
    let width = (game_area.w - 28.0).clamp(300.0, 370.0);
    let height = 126.0;
    let x = game_area.x + 14.0;
    let y = game_area.y + 14.0;
    let accent = mood_color(colonist.mood);

    draw_rectangle(x, y, width, height, Color::new(0.035, 0.042, 0.048, 0.92));
    draw_rectangle(x, y, 4.0, height, accent);
    draw_rectangle_lines(x, y, width, height, 1.0, Color::new(0.48, 0.54, 0.58, 0.85));

    draw_text(
        &truncate(&colonist.name, 25),
        x + 14.0,
        y + 24.0,
        17.0,
        WHITE,
    );
    draw_text(
        state_label(colonist.state),
        x + width - 96.0,
        y + 24.0,
        13.0,
        LIGHTGRAY,
    );

    draw_line(
        x + 12.0,
        y + 33.0,
        x + width - 12.0,
        y + 33.0,
        1.0,
        Color::new(0.35, 0.39, 0.42, 0.85),
    );

    draw_text(
        &format!(
            "Mood {:.0} | Job {}",
            colonist.mood,
            job_label(colonist.job_preference)
        ),
        x + 14.0,
        y + 53.0,
        13.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!(
            "Activity: {} at {}",
            activity_label(&colonist.current_activity),
            truncate(&activity_location_label(&colonist.activity_location), 26)
        ),
        x + 14.0,
        y + 74.0,
        12.0,
        Color::new(0.75, 0.78, 0.8, 1.0),
    );
    draw_text(
        &format!("Injury: {}", injury_label(colonist, current_tick)),
        x + 14.0,
        y + 94.0,
        12.0,
        Color::new(0.72, 0.75, 0.77, 1.0),
    );

    let relationship = strongest_relationship(colonist, colonists)
        .map(|(name, value)| {
            format!(
                "{} {} ({:+})",
                truncate(&name, 16),
                relationship_label(value),
                value
            )
        })
        .unwrap_or_else(|| "No strong tie yet".to_string());

    draw_text(
        &format!("Strongest relationship: {}", truncate(&relationship, 28)),
        x + 14.0,
        y + 114.0,
        12.0,
        Color::new(0.8, 0.82, 0.84, 1.0),
    );
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

fn mood_color(mood: f32) -> Color {
    if mood >= 65.0 {
        GREEN
    } else if mood >= 35.0 {
        YELLOW
    } else {
        ORANGE
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
