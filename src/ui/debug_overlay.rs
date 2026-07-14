//! Debug overlay - development information display

use crate::data::colonist::{relationship_label, ActivityLocation, Colonist, ColonistState};
use crate::data::priority::ColonyPriority;
use crate::data::resources::ResourceState;
use crate::data::scenario::ScenarioOutcome;
use crate::data::technology::{TechId, TechnologyState};
use crate::data::types::Position;
use macroquad_toolkit::debug::DebugOverlay;

/// Draw debug overlay with game state information. `overlay` owns the
/// smoothed FPS/frame-time readout and visibility toggle (F3); this function
/// supplies the colony-specific stat lines drawn beneath it.
pub fn draw_debug_overlay(
    overlay: &DebugOverlay,
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
    let mut lines = vec![
        format!("Tick: {}  Priority: {}", tick, priority.label()),
        match hovered_cell {
            Some(pos) => format!("Cell: ({}, {})", pos.x, pos.y),
            None => "Cell: --".to_string(),
        },
        format!("Buildings: {}", building_count),
        format!(
            "Resources: supplies {}/{} salvage {} meals {} need/day {} status {} | {}",
            resources.supplies,
            storage_capacity,
            resources.salvage,
            resources.prepared_meals,
            daily_supply_need,
            resources.condition.label(),
            outcome.label()
        ),
        objective.to_string(),
        format!(
            "Missions away {} | Tech {}/{} | Next {}",
            active_mission_count,
            technology.unlocked_count(),
            TechId::all().len(),
            technology
                .next_research_target()
                .map(|tech| tech.name())
                .unwrap_or("Complete")
        ),
        format!(
            "Progress: explore {}/8 workshop {}/6 kitchen {}/4 hauling {}/5",
            resources.exploration_progress,
            resources.workshop_progress,
            resources.kitchen_progress,
            resources.hauling_progress
        ),
        colonist_state_summary(colonists),
        "Colonists:".to_string(),
    ];

    for colonist in colonists.iter().take(6) {
        lines.push(colonist_line(colonist, colonists, tick));
    }

    overlay.draw(&lines);
}

fn colonist_state_summary(colonists: &[Colonist]) -> String {
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

    format!(
        "Idle {} Move {} Work {} Eat {} Sleep {} Mission {}",
        idle, moving, working, eating, sleeping, on_mission
    )
}

fn colonist_line(colonist: &Colonist, colonists: &[Colonist], tick: u64) -> String {
    let location = activity_location_label(&colonist.activity_location);
    let health = colonist
        .recovery_minutes_remaining(tick)
        .map(|remaining| format!(" hurt {}m", remaining))
        .unwrap_or_default();
    let relationship = strongest_relationship(colonist, colonists)
        .map(|(other_name, value)| {
            format!(
                " | {} {} ({:+})",
                other_name,
                relationship_label(value),
                value
            )
        })
        .unwrap_or_default();

    format!(
        "{} {:?} mood {:.0} @ {}{}{}",
        colonist.name, colonist.current_activity, colonist.mood, location, health, relationship
    )
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
