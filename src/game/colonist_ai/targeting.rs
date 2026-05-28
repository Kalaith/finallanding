use crate::data::building::BuildingType;
use crate::data::colonist::Colonist;
use crate::data::grid::Grid;
use crate::data::types::Position;
use crate::game::colonist_ai::social::social_score_for_building;
use crate::game::colonist_ai::types::{BuildingSnapshot, BuildingTarget, SocialLocation};

pub(super) fn find_building_entrance(
    from: Position,
    building_type: BuildingType,
    buildings: &[BuildingSnapshot],
    specific_target: Option<u32>,
    colonist: &Colonist,
    social_locations: &[SocialLocation],
    grid: &Grid,
) -> Option<BuildingTarget> {
    let mut best_target: Option<(BuildingTarget, i32, i32)> = None;

    for (id, bt, pos, (width, height)) in buildings.iter() {
        if *bt != building_type {
            continue;
        }

        if let Some(target_id) = specific_target {
            if *id != target_id {
                continue;
            }
        }

        let Some((entrance, distance)) = best_building_entrance(from, *pos, *width, *height, grid)
        else {
            continue;
        };

        let target = BuildingTarget {
            building_id: *id,
            building_type: *bt,
            entrance,
        };
        let social_score = social_score_for_building(colonist, *id, social_locations);
        let candidate = (target, social_score, distance);

        if best_target
            .as_ref()
            .map(|best| better_target_candidate(&candidate, best))
            .unwrap_or(true)
        {
            best_target = Some(candidate);
        }
    }

    best_target.map(|(target, _, _)| target)
}

pub(super) fn find_adjacent_building(
    pos: Position,
    building_type: BuildingType,
    buildings: &[BuildingSnapshot],
    specific_target: Option<u32>,
) -> Option<(u32, BuildingType)> {
    for (id, bt, bpos, (width, height)) in buildings {
        if *bt != building_type {
            continue;
        }

        if let Some(target_id) = specific_target {
            if *id != target_id {
                continue;
            }
        }

        if is_position_adjacent_to_building(pos, *bpos, *width, *height) {
            return Some((*id, *bt));
        }
    }

    None
}

pub(super) fn is_adjacent_to_building(
    pos: Position,
    building_type: BuildingType,
    buildings: &[BuildingSnapshot],
    specific_target: Option<u32>,
) -> bool {
    find_adjacent_building(pos, building_type, buildings, specific_target).is_some()
}

fn best_building_entrance(
    from: Position,
    pos: Position,
    width: u32,
    height: u32,
    grid: &Grid,
) -> Option<(Position, i32)> {
    let mut best_target: Option<Position> = None;
    let mut best_distance = i32::MAX;

    for dx in -1..=(width as i32) {
        for dy in -1..=(height as i32) {
            let check_x = pos.x + dx;
            let check_y = pos.y + dy;

            if dx >= 0 && dx < width as i32 && dy >= 0 && dy < height as i32 {
                continue;
            }

            if !grid
                .get_cell(check_x, check_y)
                .is_some_and(|cell| cell.is_walkable())
            {
                continue;
            }

            let candidate = Position::new(check_x, check_y);
            let dist = (from.x - check_x).abs() + (from.y - check_y).abs();

            if dist < best_distance {
                best_distance = dist;
                best_target = Some(candidate);
            }
        }
    }

    best_target.map(|entrance| (entrance, best_distance))
}

fn better_target_candidate(
    candidate: &(BuildingTarget, i32, i32),
    best: &(BuildingTarget, i32, i32),
) -> bool {
    let (candidate_target, candidate_score, candidate_distance) = candidate;
    let (best_target, best_score, best_distance) = best;

    candidate_score > best_score
        || (candidate_score == best_score && candidate_distance < best_distance)
        || (candidate_score == best_score
            && candidate_distance == best_distance
            && candidate_target.building_id < best_target.building_id)
}

fn is_position_adjacent_to_building(
    pos: Position,
    building_pos: Position,
    width: u32,
    height: u32,
) -> bool {
    let min_x = building_pos.x - 1;
    let max_x = building_pos.x + width as i32;
    let min_y = building_pos.y - 1;
    let max_y = building_pos.y + height as i32;

    let on_perimeter = pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y;
    let inside = pos.x >= building_pos.x
        && pos.x < building_pos.x + width as i32
        && pos.y >= building_pos.y
        && pos.y < building_pos.y + height as i32;

    on_perimeter && !inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{ActivityLocation, JobPreference, Trait};
    use crate::data::schedule::ActivityType;
    use crate::game::colonist_ai::assignment::specific_target_for_activity;

    #[test]
    fn test_work_target_prefers_supportive_occupied_building() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.relationships.insert(2, -35);
        colonist.relationships.insert(3, 30);

        let buildings = vec![
            (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
            (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
        ];
        let social_locations = vec![
            (
                2,
                ActivityLocation::Building {
                    building_id: 10,
                    building_type: BuildingType::Workshop,
                },
            ),
            (
                3,
                ActivityLocation::Building {
                    building_id: 20,
                    building_type: BuildingType::Workshop,
                },
            ),
        ];
        let grid = Grid::default();

        let target = find_building_entrance(
            colonist.position,
            BuildingType::Workshop,
            &buildings,
            None,
            &colonist,
            &social_locations,
            &grid,
        )
        .expect("workshop target should be found");

        assert_eq!(target.building_id, 20);
    }

    #[test]
    fn test_work_target_honors_keep_apart_directive() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.relationships.insert(2, 30);
        colonist.avoided_partner_id = Some(2);

        let buildings = vec![
            (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
            (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
        ];
        let social_locations = vec![(
            2,
            ActivityLocation::Building {
                building_id: 10,
                building_type: BuildingType::Workshop,
            },
        )];
        let grid = Grid::default();

        let target = find_building_entrance(
            colonist.position,
            BuildingType::Workshop,
            &buildings,
            None,
            &colonist,
            &social_locations,
            &grid,
        )
        .expect("workshop target should be found");

        assert_eq!(target.building_id, 20);
    }

    #[test]
    fn test_work_target_honors_manual_workplace_assignment() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        );
        colonist.assigned_workplace = Some(20);

        let buildings = vec![
            (10, BuildingType::Workshop, Position::new(1, 1), (2, 2)),
            (20, BuildingType::Workshop, Position::new(12, 12), (2, 2)),
        ];
        let grid = Grid::default();
        let target = find_building_entrance(
            colonist.position,
            BuildingType::Workshop,
            &buildings,
            specific_target_for_activity(
                &colonist,
                &ActivityType::Work,
                BuildingType::Workshop,
                &buildings,
            ),
            &colonist,
            &[],
            &grid,
        )
        .expect("assigned workshop target should be found");

        assert_eq!(target.building_id, 20);
    }
}
