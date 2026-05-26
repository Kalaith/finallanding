use crate::data::colonist::{Colonist, ColonistState};

pub fn update_mood(colonist: &mut Colonist, elapsed_ticks: u64) {
    let minutes = elapsed_ticks as f32;

    // 1. Decay based on Needs (Simplification: Decay over time if not eating/sleeping)
    // In a full game, we'd check Hunger/Energy.
    // For MVP, if they are "Working" too long, mood drops.
    // If they are Idle/Relaxing, mood recovers.

    match colonist.state {
        ColonistState::Working => {
            colonist.mood -= 0.05 * minutes; // Drop ~3 points per hour
        }
        ColonistState::Idle => {
            colonist.mood += 0.02 * minutes; // Slow recovery
        }
        ColonistState::Eating => {
            colonist.mood += 0.2 * minutes; // Fast recovery
        }
        ColonistState::Sleeping => {
            colonist.mood += 0.1 * minutes; // Recovery
        }
        _ => {}
    }

    // 2. Modifiers (e.g. "Ate without table")
    // colonist.mood_modifiers.retain(...)

    // Clamp
    colonist.mood = colonist.mood.clamp(0.0, 100.0);
}
