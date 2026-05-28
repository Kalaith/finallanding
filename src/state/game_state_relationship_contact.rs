use super::*;

pub(crate) fn shared_assignment_pin(first: &Colonist, second: &Colonist) -> bool {
    first
        .assigned_habitat
        .is_some_and(|id| second.assigned_habitat == Some(id))
        || first
            .assigned_workplace
            .is_some_and(|id| second.assigned_workplace == Some(id))
}

pub(crate) fn adjacent_positions(first: Position, second: Position) -> bool {
    (first.x - second.x).abs() + (first.y - second.y).abs() <= 1
}

pub(crate) fn average_relationship_between(first: &Colonist, second: &Colonist) -> i32 {
    let first_value = first.relationships.get(&second.id).copied().unwrap_or(0);
    let second_value = second.relationships.get(&first.id).copied().unwrap_or(0);

    if first_value == 0 {
        second_value
    } else if second_value == 0 {
        first_value
    } else {
        (first_value + second_value) / 2
    }
}

pub(crate) fn shared_social_location(first: &Colonist, second: &Colonist) -> bool {
    match (&first.activity_location, &second.activity_location) {
        (
            ActivityLocation::Building {
                building_id: first_id,
                ..
            },
            ActivityLocation::Building {
                building_id: second_id,
                ..
            },
        ) => first_id == second_id,
        (ActivityLocation::Ground(first_pos), ActivityLocation::Ground(second_pos)) => {
            first_pos == second_pos
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{JobPreference, Trait};

    fn test_colonist(id: u32, position: Position) -> Colonist {
        Colonist::new(
            id,
            format!("Colonist {}", id),
            position,
            Trait::HardWorker,
            JobPreference::Builder,
        )
    }

    #[test]
    fn test_average_relationship_uses_bidirectional_values() {
        let mut first = test_colonist(1, Position::new(0, 0));
        let mut second = test_colonist(2, Position::new(1, 0));

        first.relationships.insert(2, 26);
        second.relationships.insert(1, 30);

        assert_eq!(average_relationship_between(&first, &second), 28);
    }

    #[test]
    fn test_shared_social_location_requires_same_building_or_ground_cell() {
        let mut first = test_colonist(1, Position::new(0, 0));
        let mut second = test_colonist(2, Position::new(1, 0));

        first.activity_location = ActivityLocation::Building {
            building_id: 7,
            building_type: BuildingType::Workshop,
        };
        second.activity_location = ActivityLocation::Building {
            building_id: 7,
            building_type: BuildingType::Workshop,
        };

        assert!(shared_social_location(&first, &second));

        second.activity_location = ActivityLocation::Ground(Position::new(2, 2));
        assert!(!shared_social_location(&first, &second));
    }

    #[test]
    fn test_shared_assignment_and_adjacency_drive_social_contact() {
        let mut first = test_colonist(1, Position::new(4, 4));
        let mut second = test_colonist(2, Position::new(5, 4));

        assert!(adjacent_positions(first.position, second.position));
        first.assigned_workplace = Some(9);
        second.assigned_workplace = Some(9);
        assert!(shared_assignment_pin(&first, &second));
        second.assigned_workplace = Some(10);
        assert!(!shared_assignment_pin(&first, &second));
    }
}
