use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;

pub struct SummarySystem;

impl SummarySystem {
    pub fn summarize_previous_day(state: &mut GameState, new_day: u32) {
        if state.colonists.is_empty() {
            return;
        }

        let previous_day = new_day.saturating_sub(1).max(1);
        let average_mood =
            state.colonists.iter().map(|c| c.mood).sum::<f32>() / state.colonists.len() as f32;

        let mut relationship_total = 0;
        let mut relationship_count = 0;
        let mut strained_pairs = 0;

        for i in 0..state.colonists.len() {
            for j in (i + 1)..state.colonists.len() {
                let id_a = state.colonists[i].id;
                let id_b = state.colonists[j].id;
                let value_a = state.colonists[i]
                    .relationships
                    .get(&id_b)
                    .copied()
                    .unwrap_or(0);
                let value_b = state.colonists[j]
                    .relationships
                    .get(&id_a)
                    .copied()
                    .unwrap_or(0);
                let average_value = (value_a + value_b) / 2;

                relationship_total += average_value;
                relationship_count += 1;

                if average_value <= -10 {
                    strained_pairs += 1;
                }
            }
        }

        let average_relationship = if relationship_count > 0 {
            relationship_total as f32 / relationship_count as f32
        } else {
            0.0
        };

        let title = if average_mood < 30.0 || strained_pairs > 2 {
            "Colony strain is rising"
        } else if average_mood > 65.0 && average_relationship > 8.0 {
            "The colony feels connected"
        } else {
            "The colony held together"
        };

        state.push_log(
            LogCategory::Colony,
            format!("Day {} summary", previous_day),
            format!(
                "{}. Average mood {:.0}, average relationship {:+.0}, strained pairs {}.",
                title, average_mood, average_relationship, strained_pairs
            ),
        );
    }
}
