use crate::data::colonist::{relationship_label, RelationshipBand};
use crate::data::event_log::{LogCategory, SocialHistoryEntry};
use crate::data::game_state::GameState;
use crate::systems::resource_system::ResourceSystem;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyStoryReport {
    pub title: String,
    pub detail: String,
    pub recommendation: String,
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

                let relationship_band = RelationshipBand::from_value(average_value);
                if relationship_band.is_support() {
                    close_pairs += 1;
                    connected_pairs.push(pair.clone());
                }

                if relationship_band.is_risk() {
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

        let report = Self::previous_day_report(state, new_day);
        let summary = Self::colony_pressure_summary(state);
        let previous_day = new_day.saturating_sub(1).max(1);
        state.push_social_history(SocialHistoryEntry::new(
            previous_day,
            report.title.clone(),
            report.detail.clone(),
            report.recommendation.clone(),
            summary.average_mood,
            summary.average_relationship,
            summary.close_pairs,
            summary.strained_pairs,
        ));
        state.push_log(LogCategory::Colony, report.title, report.detail);
    }

    pub fn previous_day_report(state: &GameState, new_day: u32) -> DailyStoryReport {
        let previous_day = new_day.saturating_sub(1).max(1);
        let summary = Self::colony_pressure_summary(state);

        let title = if summary.average_mood < 30.0 || summary.strained_pairs > 2 {
            "Colony strain rose"
        } else if summary.average_mood > 65.0 && summary.average_relationship > 8.0 {
            "The colony found rhythm"
        } else {
            "The colony held together"
        };

        let best = summary
            .strongest_pair
            .as_ref()
            .map(|pair| {
                format!(
                    "{} and {} were the strongest bond ({} {:+}).",
                    pair.first_name, pair.second_name, pair.label, pair.value
                )
            })
            .unwrap_or_else(|| "No clear bond stood out yet.".to_string());
        let worst = summary
            .weakest_pair
            .as_ref()
            .map(|pair| {
                format!(
                    "{} and {} carried the sharpest tension ({} {:+}).",
                    pair.first_name, pair.second_name, pair.label, pair.value
                )
            })
            .unwrap_or_else(|| "No visible conflict stood out yet.".to_string());
        let recommendation = Self::pressure_recommendation(state, &summary);

        DailyStoryReport {
            title: format!("Day {} summary", previous_day),
            detail: format!(
                "{}. Mood {:.0}, relationship average {:+.0}, strained pairs {}. {} {} Next pressure: {}",
                title,
                summary.average_mood,
                summary.average_relationship,
                summary.strained_pairs,
                best,
                worst,
                recommendation
            ),
            recommendation,
        }
    }

    fn pressure_recommendation(state: &GameState, summary: &ColonyPressureSummary) -> String {
        let daily_need = ResourceSystem::daily_supply_need(state).max(1);

        if state.resources.supplies < daily_need * 2 {
            return format!(
                "raise supplies above {} before the next ration check.",
                daily_need * 2
            );
        }

        let habitat_capacity = ResourceSystem::habitat_capacity(state);
        if habitat_capacity < state.colonists.len() as u32 {
            return "add habitat capacity so recovery does not create nightly pressure."
                .to_string();
        }

        if summary.average_mood < 40.0 {
            return "use Recovery priority before low mood turns into refusals.".to_string();
        }

        if summary.strained_pairs > 0 {
            return "separate tense pairs with recovery time or different work stations."
                .to_string();
        }

        if state.technology.unlocked_count() < state.scenario.required_tech_unlocks {
            return "keep scouting until field technology is secure.".to_string();
        }

        "maintain the supply buffer and protect the strongest social bonds.".to_string()
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

    #[test]
    fn test_daily_story_report_names_best_worst_and_pressure() {
        let mut state = GameState::new();
        state.resources.supplies = 1;
        state.colonists.push(test_colonist(1, "Alice"));
        state.colonists.push(test_colonist(2, "Bob"));
        state.colonists.push(test_colonist(3, "Charlie"));
        state.colonists[0].relationships.insert(2, 24);
        state.colonists[1].relationships.insert(1, 20);
        state.colonists[0].relationships.insert(3, -18);
        state.colonists[2].relationships.insert(1, -14);

        let report = SummarySystem::previous_day_report(&state, 3);

        assert_eq!(report.title, "Day 2 summary");
        assert!(report
            .detail
            .contains("Alice and Bob were the strongest bond"));
        assert!(report
            .detail
            .contains("Alice and Charlie carried the sharpest tension"));
        assert!(report.detail.contains("Next pressure: raise supplies"));
        assert!(report.recommendation.contains("raise supplies"));
    }

    #[test]
    fn test_summarize_previous_day_preserves_social_history() {
        let mut state = GameState::new();
        state.resources.supplies = 1;
        state.colonists.push(test_colonist(1, "Alice"));
        state.colonists.push(test_colonist(2, "Bob"));
        state.colonists[0].relationships.insert(2, -20);
        state.colonists[1].relationships.insert(1, -24);

        SummarySystem::summarize_previous_day(&mut state, 2);

        let entry = state
            .social_history
            .last()
            .expect("daily summary should be preserved as history");
        assert_eq!(entry.day, 1);
        assert_eq!(entry.title, "Day 1 summary");
        assert_eq!(entry.strained_pairs, 1);
        assert!(entry.detail.contains("sharpest tension"));
        assert!(state
            .event_log
            .iter()
            .any(|entry| entry.title == "Day 1 summary"));
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
