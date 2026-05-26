use crate::data::colonist::{Colonist, ColonistState};
use crate::data::priority::ColonyPriority;

pub fn update_mood(colonist: &mut Colonist, elapsed_ticks: u64, priority: ColonyPriority) {
    let minutes = elapsed_ticks as f32;
    let delta = match colonist.state {
        ColonistState::Working => -0.05 * minutes * work_pressure_multiplier(priority),
        ColonistState::Idle => 0.02 * minutes * recovery_multiplier(priority),
        ColonistState::Eating => 0.2 * minutes * recovery_multiplier(priority),
        ColonistState::Sleeping => 0.1 * minutes * recovery_multiplier(priority),
        ColonistState::OnMission { .. } => -0.03 * minutes * mission_pressure_multiplier(priority),
        _ => 0.0,
    };

    // 1. Decay based on Needs (Simplification: Decay over time if not eating/sleeping)
    // In a full game, we'd check Hunger/Energy.
    // For MVP, if they are "Working" too long, mood drops.
    // If they are Idle/Relaxing, mood recovers.

    colonist.mood += delta;

    // 2. Modifiers (e.g. "Ate without table")
    // colonist.mood_modifiers.retain(...)

    // Clamp
    colonist.mood = colonist.mood.clamp(0.0, 100.0);
}

fn work_pressure_multiplier(priority: ColonyPriority) -> f32 {
    match priority {
        ColonyPriority::Recovery => 0.55,
        ColonyPriority::Stockpile => 1.1,
        ColonyPriority::Survey => 1.15,
    }
}

fn recovery_multiplier(priority: ColonyPriority) -> f32 {
    match priority {
        ColonyPriority::Recovery => 1.35,
        ColonyPriority::Stockpile => 1.0,
        ColonyPriority::Survey => 0.9,
    }
}

fn mission_pressure_multiplier(priority: ColonyPriority) -> f32 {
    match priority {
        ColonyPriority::Recovery => 0.75,
        ColonyPriority::Stockpile => 1.0,
        ColonyPriority::Survey => 1.2,
    }
}
