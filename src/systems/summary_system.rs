use crate::data::colonist::relationship_label;
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;

pub struct SummarySystem;

#[derive(Clone, Debug)]
pub struct RelationshipPairSummary {
    pub first_name: String,
    pub second_name: String,
    pub value: i32,
    pub label: &'static str,
}

#[derive(Clone, Debug)]
pub struct ColonyPressureSummary {
    pub average_mood: f32,
    pub average_relationship: f32,
    pub close_pairs: u32,
    pub strained_pairs: u32,
    pub connected_pairs: Vec<RelationshipPairSummary>,
    pub tense_pairs: Vec<RelationshipPairSummary>,
    pub strongest_pair: Option<RelationshipPairSummary>,
    pub weakest_pair: Option<RelationshipPairSummary>,
}

impl SummarySystem {
    pub fn colony_pressure_summary(state: &GameState) -> ColonyPressureSummary {
        let average_mood = if state.colonists.is_empty() {
            0.0
        } else {
            state.colonists.iter().map(|c| c.mood).sum::<f32>() / state.colonists.len() as f32
        };

        let mut relationship_total = 0;
        let mut relationship_count = 0;
        let mut close_pairs = 0;
        let mut strained_pairs = 0;
        let mut connected_pairs = Vec::new();
        let mut tense_pairs = Vec::new();
        let mut strongest_pair: Option<RelationshipPairSummary> = None;
        let mut weakest_pair: Option<RelationshipPairSummary> = None;

        for i in 0..state.colonists.len() {
            for j in (i + 1)..state.colonists.len() {
                let first = &state.colonists[i];
                let second = &state.colonists[j];
                let value_a = first.relationships.get(&second.id).copied().unwrap_or(0);
                let value_b = second.relationships.get(&first.id).copied().unwrap_or(0);
                let average_value = (value_a + value_b) / 2;
                let pair = RelationshipPairSummary {
                    first_name: first.name.clone(),
                    second_name: second.name.clone(),
                    value: average_value,
                    label: relationship_label(average_value),
                };

                relationship_total += average_value;
                relationship_count += 1;

                if average_value >= 10 {
                    close_pairs += 1;
                    connected_pairs.push(pair.clone());
                }

                if average_value <= -10 {
                    strained_pairs += 1;
                    tense_pairs.push(pair.clone());
                }

                if strongest_pair
                    .as_ref()
                    .is_none_or(|current| average_value > current.value)
                {
                    strongest_pair = Some(pair.clone());
                }

                if weakest_pair
                    .as_ref()
                    .is_none_or(|current| average_value < current.value)
                {
                    weakest_pair = Some(pair);
                }
            }
        }

        let average_relationship = if relationship_count > 0 {
            relationship_total as f32 / relationship_count as f32
        } else {
            0.0
        };

        connected_pairs.sort_by(|left, right| right.value.cmp(&left.value));
        tense_pairs.sort_by(|left, right| left.value.cmp(&right.value));
        connected_pairs.truncate(3);
        tense_pairs.truncate(3);

        ColonyPressureSummary {
            average_mood,
            average_relationship,
            close_pairs,
            strained_pairs,
            connected_pairs,
            tense_pairs,
            strongest_pair,
            weakest_pair,
        }
    }

    pub fn summarize_previous_day(state: &mut GameState, new_day: u32) {
        if state.colonists.is_empty() {
            return;
        }

        let previous_day = new_day.saturating_sub(1).max(1);
        let summary = Self::colony_pressure_summary(state);

        let title = if summary.average_mood < 30.0 || summary.strained_pairs > 2 {
            "Colony strain is rising"
        } else if summary.average_mood > 65.0 && summary.average_relationship > 8.0 {
            "The colony feels connected"
        } else {
            "The colony held together"
        };

        state.push_log(
            LogCategory::Colony,
            format!("Day {} summary", previous_day),
            format!(
                "{}. Average mood {:.0}, average relationship {:+.0}, strained pairs {}.",
                title, summary.average_mood, summary.average_relationship, summary.strained_pairs
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{Colonist, JobPreference, Trait};
    use crate::data::types::Position;

    #[test]
    fn test_colony_pressure_summary_finds_best_and_worst_relationships() {
        let mut state = GameState::new();
        state.colonists.push(test_colonist(1, "Alice"));
        state.colonists.push(test_colonist(2, "Bob"));
        state.colonists.push(test_colonist(3, "Charlie"));

        state.colonists[0].relationships.insert(2, 24);
        state.colonists[1].relationships.insert(1, 20);
        state.colonists[0].relationships.insert(3, -18);
        state.colonists[2].relationships.insert(1, -14);

        let summary = SummarySystem::colony_pressure_summary(&state);

        assert_eq!(summary.close_pairs, 1);
        assert_eq!(summary.strained_pairs, 1);
        assert_eq!(
            summary.strongest_pair.as_ref().map(|pair| pair.value),
            Some(22)
        );
        assert_eq!(
            summary.weakest_pair.as_ref().map(|pair| pair.value),
            Some(-16)
        );
    }

    #[test]
    fn test_colony_pressure_summary_keeps_neutral_pairs_clear() {
        let mut state = GameState::new();
        state.colonists.push(test_colonist(1, "Alice"));
        state.colonists.push(test_colonist(2, "Bob"));
        state.colonists.push(test_colonist(3, "Charlie"));

        let summary = SummarySystem::colony_pressure_summary(&state);

        assert_eq!(summary.close_pairs, 0);
        assert_eq!(summary.strained_pairs, 0);
        assert_eq!(summary.average_relationship, 0.0);
        assert!(summary.connected_pairs.is_empty());
        assert!(summary.tense_pairs.is_empty());
        assert_eq!(
            summary.strongest_pair.as_ref().map(|pair| pair.label),
            Some("Neutral")
        );
        assert_eq!(
            summary.weakest_pair.as_ref().map(|pair| pair.value),
            Some(0)
        );
    }

    #[test]
    fn test_colony_pressure_summary_counts_multiple_hostile_pairs() {
        let mut state = GameState::new();
        state.colonists.push(test_colonist(1, "Alice"));
        state.colonists.push(test_colonist(2, "Bob"));
        state.colonists.push(test_colonist(3, "Charlie"));

        state.colonists[0].relationships.insert(2, -34);
        state.colonists[1].relationships.insert(1, -30);
        state.colonists[0].relationships.insert(3, -16);
        state.colonists[2].relationships.insert(1, -14);

        let summary = SummarySystem::colony_pressure_summary(&state);

        assert_eq!(summary.close_pairs, 0);
        assert_eq!(summary.strained_pairs, 2);
        assert_eq!(summary.tense_pairs.len(), 2);
        assert_eq!(summary.tense_pairs[0].value, -32);
        assert_eq!(summary.tense_pairs[1].value, -15);
        assert_eq!(
            summary.weakest_pair.as_ref().map(|pair| pair.value),
            Some(-32)
        );
        assert_eq!(
            summary.weakest_pair.as_ref().map(|pair| pair.label),
            Some("Hostile")
        );
    }

    #[test]
    fn test_colony_pressure_summary_finds_one_strong_positive_pair() {
        let mut state = GameState::new();
        state.colonists.push(test_colonist(1, "Alice"));
        state.colonists.push(test_colonist(2, "Bob"));
        state.colonists.push(test_colonist(3, "Charlie"));

        state.colonists[0].relationships.insert(2, 42);
        state.colonists[1].relationships.insert(1, 38);

        let summary = SummarySystem::colony_pressure_summary(&state);

        assert_eq!(summary.close_pairs, 1);
        assert_eq!(summary.strained_pairs, 0);
        assert_eq!(summary.connected_pairs.len(), 1);
        assert_eq!(summary.connected_pairs[0].value, 40);
        assert_eq!(
            summary.strongest_pair.as_ref().map(|pair| pair.value),
            Some(40)
        );
        assert_eq!(
            summary.strongest_pair.as_ref().map(|pair| pair.label),
            Some("Close")
        );
    }

    fn test_colonist(id: u32, name: &str) -> Colonist {
        Colonist::new(
            id,
            name.to_string(),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        )
    }
}
