mod assignment;
mod behavior;
mod movement;
mod social;
mod targeting;
mod types;

use crate::data::game_state::GameState;
use crate::data::types::Position;
use crate::game::colonist_ai::types::{BuildingSnapshot, PendingLog, SocialLocation};
use crate::systems::mood_system::update_mood;
use crate::systems::time_system::TimeSystem;
use std::collections::HashMap;

/// Movement speed for visual interpolation (pixels per frame)
const VISUAL_MOVE_SPEED: f32 = 2.0;
pub(super) const REFUSAL_LOG_COOLDOWN_TICKS: u64 = 60;
pub(super) const SOCIAL_STRAIN_LOG_COOLDOWN_TICKS: u64 = 120;

pub fn update_colonists(state: &mut GameState, elapsed_ticks: u64) {
    if elapsed_ticks == 0 {
        for colonist in &mut state.colonists {
            colonist.update_visual_position(VISUAL_MOVE_SPEED);
        }
        return;
    }

    let (_, hour, _) = TimeSystem::get_time_of_day(state.tick);

    let occupied: HashMap<Position, u32> = state
        .colonists
        .iter()
        .filter(|c| !c.is_on_mission())
        .map(|c| (c.position, c.id))
        .collect();
    let colonist_names: HashMap<u32, String> = state
        .colonists
        .iter()
        .map(|colonist| (colonist.id, colonist.name.clone()))
        .collect();
    let social_locations: Vec<SocialLocation> = state
        .colonists
        .iter()
        .map(|colonist| (colonist.id, colonist.activity_location.clone()))
        .collect();

    let mut building_occupancy: HashMap<u32, u32> = HashMap::new();
    for c in &state.colonists {
        if c.is_on_mission() {
            continue;
        }

        if let Some(bid) = c.assigned_habitat {
            *building_occupancy.entry(bid).or_default() += 1;
        }
    }

    let buildings: Vec<BuildingSnapshot> = state
        .building_system
        .buildings()
        .iter()
        .map(|b| (b.id, b.building_type, b.position, b.size()))
        .collect();
    let habitat_capacity = 2 + state.technology.habitat_capacity_bonus();
    let priority = state.priority.active;

    let mut pending_logs: Vec<PendingLog> = Vec::new();

    for i in 0..state.colonists.len() {
        let scheduled_activity = state.colonists[i].schedule.get_activity_for_hour(hour);

        behavior::update_colonist_ai(
            &mut state.colonists[i],
            &scheduled_activity,
            &occupied,
            &colonist_names,
            &social_locations,
            &state.grid,
            &mut state.rng,
            &buildings,
            &mut building_occupancy,
            habitat_capacity,
            state.tick,
            &mut pending_logs,
        );

        state.colonists[i].update_visual_position(VISUAL_MOVE_SPEED);
        update_mood(&mut state.colonists[i], elapsed_ticks, priority);
    }

    for (category, title, detail) in pending_logs {
        state.push_log(category, title, detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{Colonist, ColonistState, JobPreference, Trait};
    use crate::data::event_log::LogCategory;

    #[test]
    fn test_strained_proximity_logs_visible_mood_drop() {
        let mut state = GameState::new();
        state.tick = 420;

        let mut alice = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Explorer,
        );
        alice.state = ColonistState::Moving {
            target: Position::new(1, 0),
        };
        alice.relationships.insert(2, -30);
        alice.mood = 50.0;

        let mut bob = Colonist::new(
            2,
            "Bob".to_string(),
            Position::new(1, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        bob.relationships.insert(1, -30);

        state.colonists.push(alice);
        state.colonists.push(bob);

        update_colonists(&mut state, 1);

        let alice = &state.colonists[0];
        assert_eq!(alice.position, Position::new(0, 0));
        assert!(alice.mood <= 45.1);

        let log = state
            .event_log
            .iter()
            .find(|entry| entry.category == LogCategory::Social)
            .expect("strained proximity should create a social log");
        assert_eq!(log.title, "Alice avoided Bob");
        assert!(log.detail.contains("strained relationship"));
        assert!(log.detail.contains("Mood dropped"));
    }
}
