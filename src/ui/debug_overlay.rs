//! Debug overlay - development information display

use crate::data::colonist::{relationship_label, ActivityLocation, Colonist, ColonistState};
use crate::data::priority::ColonyPriority;
use crate::data::resources::ResourceState;
use crate::data::scenario::ScenarioOutcome;
use crate::data::technology::{TechId, TechnologyState};
use crate::data::types::Position;
use macroquad::prelude::*;

/// Draw debug overlay with game state information
pub fn draw_debug_overlay(
    tick: u64,
    colonists: &[Colonist],
    hovered_cell: Option<Position>,
    building_count: usize,
    resources: &ResourceState,
    storage_capacity: i32,
    daily_supply_need: i32,
    objective: &str,
    outcome: ScenarioOutcome,
    active_mission_count: usize,
    technology: &TechnologyState,
    priority: ColonyPriority,
) {
    let x = 10.0;
    let y = 60.0; // Below top bar
    let line_height = 18.0;

    // Semi-transparent background
    draw_rectangle(
        x - 5.0,
        y - 15.0,
        430.0,
        430.0,
        Color::new(0.0, 0.0, 0.0, 0.7),
    );
    draw_rectangle_lines(x - 5.0, y - 15.0, 430.0, 430.0, 1.0, YELLOW);

    // FPS
    let fps = get_fps();
    let fps_color = if fps >= 55 {
        GREEN
    } else if fps >= 30 {
        YELLOW
    } else {
        RED
    };
    draw_text(&format!("FPS: {}", fps), x, y, 16.0, fps_color);

    // Tick
    draw_text(&format!("Tick: {}", tick), x, y + line_height, 16.0, WHITE);
    draw_text(
        &format!("Priority: {}", priority.label()),
        x + 130.0,
        y + line_height,
        16.0,
        LIGHTGRAY,
    );

    // Hovered cell
    if let Some(pos) = hovered_cell {
        draw_text(
            &format!("Cell: ({}, {})", pos.x, pos.y),
            x,
            y + line_height * 2.0,
            16.0,
            YELLOW,
        );
    } else {
        draw_text("Cell: --", x, y + line_height * 2.0, 16.0, GRAY);
    }

    // Building count
    draw_text(
        &format!("Buildings: {}", building_count),
        x,
        y + line_height * 3.0,
        16.0,
        WHITE,
    );
    draw_text(
        &format!(
            "Resources: supplies {}/{} salvage {} meals {} need/day {} status {} | {}",
            resources.supplies,
            storage_capacity,
            resources.salvage,
            resources.prepared_meals,
            daily_supply_need,
            resources.condition.label(),
            outcome.label()
        ),
        x,
        y + line_height * 4.0,
        13.0,
        LIGHTGRAY,
    );
    draw_text(objective, x, y + line_height * 6.4, 12.0, LIGHTGRAY);
    draw_text(
        &format!(
            "Missions away {} | Tech {}/{} | Next {}",
            active_mission_count,
            technology.unlocked_count(),
            TechId::all().len(),
            technology
                .next_locked_tech()
                .map(|tech| tech.name())
                .unwrap_or("Complete")
        ),
        x,
        y + line_height * 5.6,
        12.0,
        GRAY,
    );
    draw_text(
        &format!(
            "Progress: explore {}/8 workshop {}/6 kitchen {}/4 hauling {}/5",
            resources.exploration_progress,
            resources.workshop_progress,
            resources.kitchen_progress,
            resources.hauling_progress
        ),
        x,
        y + line_height * 4.8,
        12.0,
        GRAY,
    );

    // Colonist states
    let mut idle = 0;
    let mut moving = 0;
    let mut working = 0;
    let mut eating = 0;
    let mut sleeping = 0;
    let mut on_mission = 0;

    for colonist in colonists {
        match colonist.state {
            ColonistState::Idle => idle += 1,
            ColonistState::Moving { .. } => moving += 1,
            ColonistState::Working => working += 1,
            ColonistState::Eating => eating += 1,
            ColonistState::Sleeping => sleeping += 1,
            ColonistState::OnMission { .. } => on_mission += 1,
        }
    }

    draw_text("Colonist States:", x, y + line_height * 7.8, 14.0, WHITE);
    draw_text(
        &format!("  Idle: {}  Moving: {}", idle, moving),
        x,
        y + line_height * 8.8,
        14.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!(
            "  Work: {}  Eat: {}  Sleep: {}  Mission: {}",
            working, eating, sleeping, on_mission
        ),
        x,
        y + line_height * 9.8,
        14.0,
        LIGHTGRAY,
    );

    draw_text("Colonists:", x, y + line_height * 11.3, 14.0, WHITE);

    for (i, colonist) in colonists.iter().take(6).enumerate() {
        let row_y = y + line_height * 12.3 + i as f32 * 30.0;
        let location = activity_location_label(&colonist.activity_location);
        let health = colonist
            .recovery_minutes_remaining(tick)
            .map(|remaining| format!(" hurt {}m", remaining))
            .unwrap_or_default();
        draw_text(
            &format!(
                "{} {:?} mood {:.0} @ {}{}",
                colonist.name, colonist.current_activity, colonist.mood, location, health
            ),
            x,
            row_y,
            12.0,
            LIGHTGRAY,
        );

        if let Some((other_name, value)) = strongest_relationship(colonist, colonists) {
            draw_text(
                &format!(
                    "  notable: {} {} ({:+})",
                    other_name,
                    relationship_label(value),
                    value
                ),
                x,
                row_y + 13.0,
                11.0,
                GRAY,
            );
        }
    }

    draw_text("[F3] to hide debug", x, y + 405.0, 12.0, GRAY);
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
