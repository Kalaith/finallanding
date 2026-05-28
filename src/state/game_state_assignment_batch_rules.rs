use super::*;

pub(crate) fn apply_batch_home_pin(
    colonists: &mut [Colonist],
    selected_id: u32,
    habitat_id: u32,
    visible_indices: &[usize],
    capacity: u32,
) -> Vec<String> {
    let mut assigned_count = colonists
        .iter()
        .filter(|colonist| colonist.assigned_habitat == Some(habitat_id))
        .count() as u32;
    let mut assigned = Vec::new();

    for index in visible_indices {
        if assigned_count >= capacity {
            break;
        }

        let Some(colonist) = colonists.get_mut(*index) else {
            continue;
        };
        if colonist.id == selected_id || colonist.assigned_habitat == Some(habitat_id) {
            continue;
        }

        colonist.assigned_habitat = Some(habitat_id);
        assigned_count += 1;
        assigned.push(colonist.name.clone());
    }

    assigned
}

pub(crate) fn apply_batch_work_pin(
    colonists: &mut [Colonist],
    selected_id: u32,
    workplace_id: u32,
    building_type: BuildingType,
    target_indices: &[usize],
) -> Vec<String> {
    let mut assigned = Vec::new();

    for index in target_indices {
        let Some(colonist) = colonists.get_mut(*index) else {
            continue;
        };
        if colonist.id == selected_id
            || colonist.assigned_workplace == Some(workplace_id)
            || colonist.job_preference.work_building_type() != building_type
        {
            continue;
        }

        colonist.assigned_workplace = Some(workplace_id);
        if matches!(
            colonist.state,
            ColonistState::Working | ColonistState::Moving { .. }
        ) {
            colonist.state = ColonistState::Idle;
            colonist.activity_location = ActivityLocation::None;
        }
        assigned.push(colonist.name.clone());
    }

    assigned
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BatchAssignmentScope {
    Page,
    All,
}

impl BatchAssignmentScope {
    fn label(self) -> &'static str {
        match self {
            BatchAssignmentScope::Page => "visible roster",
            BatchAssignmentScope::All => "all compatible survivors",
        }
    }
}

pub(crate) fn batch_assignment_log(
    title: &'static str,
    source_name: &str,
    pin_prefix: &str,
    building_id: u32,
    scope: BatchAssignmentScope,
    assigned: Vec<String>,
) -> (String, String) {
    let detail = if assigned.is_empty() {
        format!(
            "{} had no compatible survivors in {} to copy {}#{} to.",
            source_name,
            scope.label(),
            pin_prefix,
            building_id
        )
    } else {
        format!(
            "Copied {}#{} from {} to {} in {}.",
            pin_prefix,
            building_id,
            source_name,
            truncate_text(&assigned.join(", "), 45),
            scope.label()
        )
    };

    (title.to_string(), detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::Trait;

    fn builder(id: u32, name: &str, position: Position) -> Colonist {
        Colonist::new(
            id,
            name.to_string(),
            position,
            Trait::HardWorker,
            JobPreference::Builder,
        )
    }

    #[test]
    fn test_batch_home_pin_respects_visible_page_and_capacity() {
        let mut colonists = (0..5)
            .map(|id| builder(id, &format!("Colonist {}", id), Position::new(id as i32, 0)))
            .collect::<Vec<_>>();
        colonists[0].assigned_habitat = Some(7);

        let assigned = apply_batch_home_pin(&mut colonists, 0, 7, &[0, 1, 2, 3], 2);

        assert_eq!(assigned, vec!["Colonist 1".to_string()]);
        assert_eq!(colonists[1].assigned_habitat, Some(7));
        assert_eq!(colonists[2].assigned_habitat, None);
    }

    #[test]
    fn test_batch_work_pin_only_copies_to_compatible_visible_roles() {
        let mut colonists = vec![
            builder(0, "Alice", Position::new(0, 0)),
            builder(1, "Bob", Position::new(1, 0)),
            Colonist::new(
                2,
                "Diana".to_string(),
                Position::new(2, 0),
                Trait::Gourmet,
                JobPreference::Cook,
            ),
        ];
        colonists[0].assigned_workplace = Some(9);
        colonists[1].state = ColonistState::Working;
        colonists[1].activity_location = ActivityLocation::Building {
            building_id: 3,
            building_type: BuildingType::Workshop,
        };

        let assigned =
            apply_batch_work_pin(&mut colonists, 0, 9, BuildingType::Workshop, &[0, 1, 2]);

        assert_eq!(assigned, vec!["Bob".to_string()]);
        assert_eq!(colonists[1].assigned_workplace, Some(9));
        assert_eq!(colonists[1].state, ColonistState::Idle);
        assert_eq!(colonists[1].activity_location, ActivityLocation::None);
        assert_eq!(colonists[2].assigned_workplace, None);
    }

    #[test]
    fn test_batch_assignment_log_names_all_colony_scope() {
        let (_title, detail) = batch_assignment_log(
            "Batch work pins",
            "Alice",
            "W",
            9,
            BatchAssignmentScope::All,
            vec!["Bob".to_string(), "Charlie".to_string()],
        );

        assert!(detail.contains("all compatible survivors"));
        assert!(detail.contains("Bob, Charlie"));
    }
}
