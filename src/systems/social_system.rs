use crate::data::building::BuildingType;
use crate::data::colonist::{relationship_label, ActivityLocation, ColonistState};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::priority::ColonyPriority;
use std::collections::HashMap;

pub struct SocialSystem;

#[derive(Clone, Copy, Debug)]
enum RelationshipSource {
    Work,
    Meal,
    SharedHabitat,
}

impl RelationshipSource {
    fn label(self) -> &'static str {
        match self {
            RelationshipSource::Work => "Work",
            RelationshipSource::Meal => "Meal",
            RelationshipSource::SharedHabitat => "Habitat",
        }
    }

    fn positive_detail(self, old_value: i32, new_value: i32) -> &'static str {
        let options = match self {
            RelationshipSource::Work => &[
                "compatible crews built trust at the same station",
                "one covered the other's weak spot during the shift",
                "shared repair pressure turned into practical respect",
            ][..],
            RelationshipSource::Meal => &[
                "a shared meal gave them neutral ground to reconnect",
                "meal time let the tension drain out of the conversation",
                "they found a small routine worth repeating over dinner",
            ],
            RelationshipSource::SharedHabitat => &[
                "recovering together made the habitat feel safer",
                "quiet rest helped them see each other as reliable",
                "shared shelter turned into a brief moment of trust",
            ],
        };

        options[relationship_detail_index(old_value, new_value, options.len())]
    }

    fn negative_detail(self, old_value: i32, new_value: i32) -> &'static str {
        let options = match self {
            RelationshipSource::Work => &[
                "low mood or a disliked coworker made the shift feel forced",
                "the work pace left too little patience for compromise",
                "a bad handoff made the whole station feel personal",
            ][..],
            RelationshipSource::Meal => &[
                "stress followed them into the Mess Hall",
                "the table felt too small for the argument they carried in",
                "they ate together but left less willing to cooperate",
            ],
            RelationshipSource::SharedHabitat => &[
                "cramped recovery space kept them close to someone they mistrust",
                "resting near unresolved tension made recovery shallow",
                "sleeping arrangements turned small irritation into resentment",
            ],
        };

        options[relationship_detail_index(old_value, new_value, options.len())]
    }
}

impl SocialSystem {
    /// Checks if colonists are working together in the same building.
    /// Should be called periodically, usually once per in-game hour.
    pub fn check_working_together(state: &mut GameState) {
        let mut workplace_map: HashMap<u32, Vec<u32>> = HashMap::new();

        for colonist in &state.colonists {
            if colonist.state != ColonistState::Working {
                continue;
            }

            if let ActivityLocation::Building { building_id, .. } = colonist.activity_location {
                workplace_map
                    .entry(building_id)
                    .or_default()
                    .push(colonist.id);
            }
        }

        Self::apply_group_relationships(state, &workplace_map, RelationshipSource::Work);
    }

    /// Checks if colonists are eating together in the Mess Hall.
    /// Should be called periodically, usually once per in-game hour.
    pub fn check_eating_together(state: &mut GameState) {
        let mut mess_hall_map: HashMap<u32, Vec<u32>> = HashMap::new();

        for colonist in &state.colonists {
            if colonist.state != ColonistState::Eating {
                continue;
            }

            if let ActivityLocation::Building {
                building_id,
                building_type,
            } = colonist.activity_location
            {
                if building_type == BuildingType::MessHall {
                    mess_hall_map
                        .entry(building_id)
                        .or_default()
                        .push(colonist.id);
                }
            }
        }

        Self::apply_group_relationships(state, &mess_hall_map, RelationshipSource::Meal);
    }

    pub fn apply_shared_habitat_relationships(
        state: &mut GameState,
        habitat_groups: &HashMap<u32, Vec<u32>>,
    ) {
        Self::apply_group_relationships(state, habitat_groups, RelationshipSource::SharedHabitat);
    }

    fn apply_group_relationships(
        state: &mut GameState,
        groups: &HashMap<u32, Vec<u32>>,
        source: RelationshipSource,
    ) {
        let mut pair_updates: Vec<(u32, u32, i32)> = Vec::new();

        for group in groups.values() {
            if group.len() < 2 {
                continue;
            }

            for i in 0..group.len() {
                for j in (i + 1)..group.len() {
                    let id_a = group[i];
                    let id_b = group[j];
                    pair_updates.push((
                        id_a,
                        id_b,
                        Self::relationship_delta(state, id_a, id_b, source),
                    ));
                }
            }
        }

        for (id_a, id_b, change) in pair_updates {
            Self::apply_pair_update(state, id_a, id_b, change, source);
        }
    }

    fn relationship_delta(
        state: &GameState,
        id_a: u32,
        id_b: u32,
        source: RelationshipSource,
    ) -> i32 {
        let mood_a = Self::colonist_mood(state, id_a);
        let mood_b = Self::colonist_mood(state, id_b);
        let avg_mood = (mood_a + mood_b) * 0.5;
        let relationship = Self::relationship_value(state, id_a, id_b);

        let base_delta = match source {
            RelationshipSource::Work => {
                if avg_mood < 35.0 || relationship < -20 {
                    -1
                } else {
                    1
                }
            }
            RelationshipSource::Meal => {
                if avg_mood < 25.0 && relationship < -10 {
                    -1
                } else {
                    1
                }
            }
            RelationshipSource::SharedHabitat => {
                if avg_mood < 30.0 || relationship < -20 {
                    -1
                } else {
                    2
                }
            }
        };

        Self::priority_relationship_delta(state.priority.active, source, base_delta)
    }

    fn priority_relationship_delta(
        priority: ColonyPriority,
        source: RelationshipSource,
        base_delta: i32,
    ) -> i32 {
        match (priority, source, base_delta) {
            (ColonyPriority::Recovery, RelationshipSource::Meal, value) if value > 0 => value + 1,
            (ColonyPriority::Recovery, RelationshipSource::SharedHabitat, value) if value > 0 => {
                value + 1
            }
            (ColonyPriority::Recovery, RelationshipSource::Work, value) if value < 0 => 0,
            (ColonyPriority::Stockpile, RelationshipSource::Work, value) if value > 0 => value + 1,
            (ColonyPriority::Survey, RelationshipSource::Work, value) if value < 0 => value - 1,
            _ => base_delta,
        }
    }

    fn apply_pair_update(
        state: &mut GameState,
        id_a: u32,
        id_b: u32,
        change: i32,
        source: RelationshipSource,
    ) {
        let name_a = Self::colonist_name(state, id_a);
        let name_b = Self::colonist_name(state, id_b);
        let old_a = Self::relationship_value(state, id_a, id_b);
        let old_b = Self::relationship_value(state, id_b, id_a);
        let old_value = (old_a + old_b) / 2;
        let old_label = relationship_label(old_value);

        let new_a = Self::apply_one_way_update(state, id_a, id_b, change);
        let new_b = Self::apply_one_way_update(state, id_b, id_a, change);
        let new_value = (new_a + new_b) / 2;
        let new_label = relationship_label(new_value);

        if Self::should_log_relationship_shift(change, old_value, new_value, old_label, new_label) {
            let detail = if change > 0 {
                source.positive_detail(old_value, new_value)
            } else {
                source.negative_detail(old_value, new_value)
            };
            let title = if old_label != new_label {
                format!("{} and {} are now {}", name_a, name_b, new_label)
            } else if change > 0 {
                format!("{} and {} connected", name_a, name_b)
            } else {
                format!("{} and {} clashed", name_a, name_b)
            };

            state.push_log(
                LogCategory::Social,
                title,
                format!(
                    "{}: {}. {} -> {} ({:+}, now {:+}).",
                    source.label(),
                    detail,
                    old_label,
                    new_label,
                    change,
                    new_value
                ),
            );
        }
    }

    fn should_log_relationship_shift(
        change: i32,
        old_value: i32,
        new_value: i32,
        old_label: &str,
        new_label: &str,
    ) -> bool {
        if change == 0 || old_value == new_value {
            return false;
        }

        old_label != new_label
            || old_value == 0
            || old_value.signum() != new_value.signum()
            || new_value.abs() % 10 == 0
    }

    fn apply_one_way_update(state: &mut GameState, from_id: u32, to_id: u32, change: i32) -> i32 {
        if let Some(colonist) = state.colonists.iter_mut().find(|c| c.id == from_id) {
            let value = colonist.relationships.entry(to_id).or_insert(0);
            *value = (*value + change).clamp(-50, 50);
            *value
        } else {
            0
        }
    }

    fn relationship_value(state: &GameState, from_id: u32, to_id: u32) -> i32 {
        state
            .colonists
            .iter()
            .find(|c| c.id == from_id)
            .and_then(|c| c.relationships.get(&to_id).copied())
            .unwrap_or(0)
    }

    fn colonist_mood(state: &GameState, colonist_id: u32) -> f32 {
        state
            .colonists
            .iter()
            .find(|c| c.id == colonist_id)
            .map(|c| c.mood)
            .unwrap_or(50.0)
    }

    fn colonist_name(state: &GameState, colonist_id: u32) -> String {
        state
            .colonists
            .iter()
            .find(|c| c.id == colonist_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("Colonist {}", colonist_id))
    }
}

fn relationship_detail_index(old_value: i32, new_value: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }

    (old_value.abs() as usize + new_value.abs() as usize) % len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{ActivityLocation, Colonist, JobPreference, Trait};
    use crate::data::priority::ColonyPriority;
    use crate::data::types::Position;

    #[test]
    fn test_recovery_priority_strengthens_shared_meals() {
        let mut state = GameState::new();
        state.priority.active = ColonyPriority::Recovery;

        for id in 1..=2 {
            let mut colonist = Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::Gourmet,
                JobPreference::Cook,
            );
            colonist.state = ColonistState::Eating;
            colonist.activity_location = ActivityLocation::Building {
                building_id: 7,
                building_type: BuildingType::MessHall,
            };
            colonist.mood = 60.0;
            state.colonists.push(colonist);
        }

        SocialSystem::check_eating_together(&mut state);

        assert_eq!(state.colonists[0].relationships.get(&2), Some(&2));
        assert_eq!(state.colonists[1].relationships.get(&1), Some(&2));
    }

    #[test]
    fn test_first_work_relationship_change_logs_visible_reason() {
        let mut state = GameState::new();
        state.priority.active = ColonyPriority::Recovery;

        for id in 1..=2 {
            let mut colonist = Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            );
            colonist.state = ColonistState::Working;
            colonist.activity_location = ActivityLocation::Building {
                building_id: 12,
                building_type: BuildingType::Workshop,
            };
            colonist.mood = 60.0;
            state.colonists.push(colonist);
        }

        SocialSystem::check_working_together(&mut state);

        let log = state
            .event_log
            .iter()
            .find(|entry| entry.category == LogCategory::Social)
            .expect("first relationship shift should be logged");
        assert_eq!(log.title, "Colonist 1 and Colonist 2 connected");
        assert!(log.detail.contains("Work:"));
        assert!(log.detail.contains("Neutral -> Neutral"));
        assert!(log.detail.contains("now +1"));
    }
}
