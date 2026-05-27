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
    fn positive_detail(self) -> &'static str {
        match self {
            RelationshipSource::Work => {
                "Working the same station built trust; compatible crews do better together."
            }
            RelationshipSource::Meal => "A shared meal gave them neutral ground to reconnect.",
            RelationshipSource::SharedHabitat => {
                "Recovering in the same habitat made the space feel safer."
            }
        }
    }

    fn negative_detail(self) -> &'static str {
        match self {
            RelationshipSource::Work => {
                "Low mood or a disliked coworker made the shift feel forced."
            }
            RelationshipSource::Meal => {
                "Stress followed them into the Mess Hall; the meal did not clear the tension."
            }
            RelationshipSource::SharedHabitat => {
                "Cramped recovery space kept them close to someone they already mistrust."
            }
        }
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
        let old_value = Self::relationship_value(state, id_a, id_b);
        let old_label = relationship_label(old_value);

        let new_a = Self::apply_one_way_update(state, id_a, id_b, change);
        let new_b = Self::apply_one_way_update(state, id_b, id_a, change);
        let new_value = (new_a + new_b) / 2;
        let new_label = relationship_label(new_value);

        if old_label != new_label {
            let direction = if change > 0 { "improved" } else { "worsened" };
            let detail = if change > 0 {
                source.positive_detail()
            } else {
                source.negative_detail()
            };

            state.push_log(
                LogCategory::Social,
                format!("{} and {} are now {}", name_a, name_b, new_label),
                format!(
                    "Their relationship {} from {} to {} ({:+}). {}",
                    direction, old_label, new_label, change, detail
                ),
            );
        }
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
}
