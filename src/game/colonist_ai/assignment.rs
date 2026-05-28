use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, Colonist, ColonistState, JobPreference};
use crate::data::event_log::LogCategory;
use crate::data::schedule::ActivityType;
use crate::game::colonist_ai::social::social_score_for_building;
use crate::game::colonist_ai::types::{BuildingSnapshot, PendingLog, SocialLocation};
use crate::game::colonist_ai::REFUSAL_LOG_COOLDOWN_TICKS;
use std::collections::HashMap;

pub(super) fn find_or_assign_habitat(
    colonist: &mut Colonist,
    buildings: &[BuildingSnapshot],
    building_occupancy: &mut HashMap<u32, u32>,
    habitat_capacity: u32,
    social_locations: &[SocialLocation],
    pending_logs: &mut Vec<PendingLog>,
    current_tick: u64,
) -> Option<BuildingType> {
    if let Some(bid) = colonist.assigned_habitat {
        if building_matches_type(buildings, bid, BuildingType::Habitat) {
            return Some(BuildingType::Habitat);
        }

        colonist.assigned_habitat = None;
    }

    let mut best_habitat: Option<(u32, i32, u32)> = None;
    for (id, b_type, _, _) in buildings {
        if *b_type != BuildingType::Habitat {
            continue;
        }

        let count = *building_occupancy.get(id).unwrap_or(&0);
        if count >= habitat_capacity {
            continue;
        }

        let social_score = social_score_for_building(colonist, *id, social_locations);
        let candidate = (*id, social_score, count);
        if best_habitat
            .map(|best| better_habitat_candidate(candidate, best))
            .unwrap_or(true)
        {
            best_habitat = Some(candidate);
        }
    }

    if let Some((building_id, _, _)) = best_habitat {
        colonist.assigned_habitat = Some(building_id);
        *building_occupancy.entry(building_id).or_default() += 1;
        return Some(BuildingType::Habitat);
    }

    colonist.state = ColonistState::Sleeping;
    colonist.current_activity = ActivityType::Sleep;
    colonist.activity_location = ActivityLocation::Ground(colonist.position);

    if current_tick.saturating_sub(colonist.last_refusal_tick) >= REFUSAL_LOG_COOLDOWN_TICKS {
        colonist.last_refusal_tick = current_tick;
        pending_logs.push((
            LogCategory::Mood,
            format!("{} slept without a habitat", colonist.name),
            "Recovery is weaker when there are not enough usable habitats.".to_string(),
        ));
    }

    None
}

pub(super) fn building_type_for_activity(
    activity: &ActivityType,
    job_preference: JobPreference,
) -> Option<BuildingType> {
    match activity {
        ActivityType::Sleep => Some(BuildingType::Habitat),
        ActivityType::Work => Some(job_preference.work_building_type()),
        ActivityType::Eat => Some(BuildingType::MessHall),
        ActivityType::Relax => None,
    }
}

pub(super) fn specific_target_for_activity(
    colonist: &Colonist,
    activity: &ActivityType,
    building_type: BuildingType,
    buildings: &[BuildingSnapshot],
) -> Option<u32> {
    match activity {
        ActivityType::Sleep => colonist.assigned_habitat.filter(|building_id| {
            building_matches_type(buildings, *building_id, BuildingType::Habitat)
        }),
        ActivityType::Work => colonist
            .assigned_workplace
            .filter(|building_id| building_matches_type(buildings, *building_id, building_type)),
        ActivityType::Eat | ActivityType::Relax => None,
    }
}

pub(super) fn building_matches_type(
    buildings: &[BuildingSnapshot],
    building_id: u32,
    building_type: BuildingType,
) -> bool {
    buildings
        .iter()
        .any(|(id, candidate_type, _, _)| *id == building_id && *candidate_type == building_type)
}

fn better_habitat_candidate(candidate: (u32, i32, u32), best: (u32, i32, u32)) -> bool {
    let (candidate_id, candidate_score, candidate_count) = candidate;
    let (best_id, best_score, best_count) = best;

    candidate_score > best_score
        || (candidate_score == best_score && candidate_count < best_count)
        || (candidate_score == best_score
            && candidate_count == best_count
            && candidate_id < best_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::Trait;
    use crate::data::types::Position;

    #[test]
    fn test_workplace_assignment_ignores_wrong_building_type() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.assigned_workplace = Some(20);

        let buildings = vec![(20, BuildingType::MessHall, Position::new(12, 12), (3, 2))];

        assert_eq!(
            specific_target_for_activity(
                &colonist,
                &ActivityType::Work,
                BuildingType::Workshop,
                &buildings,
            ),
            None
        );
    }

    #[test]
    fn test_habitat_assignment_prefers_supportive_roommate() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.relationships.insert(2, -26);
        colonist.relationships.insert(3, 22);

        let buildings = vec![
            (10, BuildingType::Habitat, Position::new(1, 1), (2, 2)),
            (20, BuildingType::Habitat, Position::new(10, 10), (2, 2)),
        ];
        let social_locations = vec![
            (
                2,
                ActivityLocation::Building {
                    building_id: 10,
                    building_type: BuildingType::Habitat,
                },
            ),
            (
                3,
                ActivityLocation::Building {
                    building_id: 20,
                    building_type: BuildingType::Habitat,
                },
            ),
        ];
        let mut building_occupancy = HashMap::new();
        let mut pending_logs = Vec::new();

        let target = find_or_assign_habitat(
            &mut colonist,
            &buildings,
            &mut building_occupancy,
            2,
            &social_locations,
            &mut pending_logs,
            420,
        );

        assert_eq!(target, Some(BuildingType::Habitat));
        assert_eq!(colonist.assigned_habitat, Some(20));
    }
}
