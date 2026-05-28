use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpaceAssignmentKind {
    Recovery,
    Work,
}

pub(crate) fn space_assignment_kind(
    job_preference: crate::data::colonist::JobPreference,
    building_type: BuildingType,
) -> Option<SpaceAssignmentKind> {
    if building_type == BuildingType::Habitat {
        return Some(SpaceAssignmentKind::Recovery);
    }

    (building_type == job_preference.work_building_type()).then_some(SpaceAssignmentKind::Work)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::JobPreference;

    #[test]
    fn test_space_assignment_kind_matches_role_and_room() {
        assert_eq!(
            space_assignment_kind(JobPreference::Builder, BuildingType::Habitat),
            Some(SpaceAssignmentKind::Recovery)
        );
        assert_eq!(
            space_assignment_kind(JobPreference::Builder, BuildingType::Workshop),
            Some(SpaceAssignmentKind::Work)
        );
        assert_eq!(
            space_assignment_kind(JobPreference::Builder, BuildingType::MessHall),
            None
        );
        assert_eq!(
            space_assignment_kind(JobPreference::Cook, BuildingType::MessHall),
            Some(SpaceAssignmentKind::Work)
        );
    }
}
