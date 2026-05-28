use crate::data::assign_roster;
use crate::data::colonist::{relationship_label, Colonist, JobPreference, RelationshipBand};
use crate::data::technology::TechnologyState;
use crate::systems::relationship_directive_system::{PairDirective, RelationshipDirectiveSystem};
use crate::ui::hit_zones::{AssignRosterFilter, AssignRosterSort};

pub(super) struct AssignPairAction {
    pub(super) label: String,
    pub(super) detail: String,
    pub(super) directive: PairDirective,
}

pub(super) fn assign_roster_page_count(
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> usize {
    assign_roster::assign_roster_page_count(
        colonists,
        selected_colonist_id,
        active_filter,
        active_role_filter,
        active_building_filter,
    )
}

pub(super) fn assign_visible_colonists<'a>(
    colonists: &'a [Colonist],
    selected_colonist_id: Option<u32>,
    page: usize,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<&'a Colonist> {
    assign_roster::assign_visible_colonists(
        colonists,
        selected_colonist_id,
        page,
        active_filter,
        active_sort,
        active_role_filter,
        active_building_filter,
    )
}

pub(super) fn assign_role_filter_label(role: Option<JobPreference>) -> &'static str {
    match role {
        None => "ALL",
        Some(JobPreference::Explorer) => "EXP",
        Some(JobPreference::Builder) => "BLD",
        Some(JobPreference::Cook) => "CK",
        Some(JobPreference::Hauler) => "HL",
        Some(JobPreference::None) => "GEN",
    }
}

pub(super) fn assign_pair_action(
    colonists: &[Colonist],
    selected_id: u32,
    target_id: u32,
) -> Option<AssignPairAction> {
    let current =
        RelationshipDirectiveSystem::directive_for_pair(colonists, selected_id, target_id);
    let directive = current.or_else(|| {
        RelationshipDirectiveSystem::recommended_directive(colonists, selected_id, target_id)
    })?;
    let value =
        RelationshipDirectiveSystem::average_relationship(colonists, selected_id, target_id)
            .unwrap_or(0);
    let label = match current {
        Some(active) => format!("{} set {:+}", active.label(), value),
        None => format!("{} {:+}", directive.label(), value),
    };
    let detail = RelationshipDirectiveSystem::directive_detail(colonists, selected_id, target_id);

    Some(AssignPairAction {
        label,
        detail,
        directive,
    })
}

pub(super) fn selected_assignment_label(colonist: &Colonist) -> String {
    let home = colonist
        .assigned_habitat
        .map(|id| format!("H#{}", id))
        .unwrap_or_else(|| "H--".to_string());
    let work = colonist
        .assigned_workplace
        .map(|id| format!("W#{}", id))
        .unwrap_or_else(|| "W--".to_string());
    format!("{} {}", home, work)
}

pub(super) fn selected_assignment_detail(
    colonist: &Colonist,
    colonists: &[Colonist],
    technology: &TechnologyState,
) -> String {
    let base = format!(
        "Click this card to cycle role. Click a compatible map building to pin or clear recovery/work space. Current pins: {}.",
        selected_assignment_label(colonist)
    );
    assignment_pin_warning(colonist, colonists, technology)
        .map(|warning| format!("{} {}", base, warning.detail))
        .unwrap_or(base)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AssignmentPinWarning {
    pub(super) label: String,
    pub(super) detail: String,
}

pub(super) fn assignment_pin_warning(
    colonist: &Colonist,
    colonists: &[Colonist],
    technology: &TechnologyState,
) -> Option<AssignmentPinWarning> {
    if let Some(habitat_id) = colonist.assigned_habitat {
        let count = colonists
            .iter()
            .filter(|candidate| candidate.assigned_habitat == Some(habitat_id))
            .count() as u32;
        let capacity = 2 + technology.habitat_capacity_bonus();
        if count > capacity {
            return Some(AssignmentPinWarning {
                label: "CAP".to_string(),
                detail: format!(
                    "Habitat #{} over capacity: {}/{} pinned survivors.",
                    habitat_id, count, capacity
                ),
            });
        }

        if let Some((name, value)) = first_assignment_conflict(
            colonist,
            colonists,
            AssignmentPinLocation::Habitat(habitat_id),
        ) {
            return Some(AssignmentPinWarning {
                label: "TENSE".to_string(),
                detail: format!(
                    "{}: {} {:+} in H#{}. Pin another room or use Apart.",
                    name,
                    relationship_label(value),
                    value,
                    habitat_id
                ),
            });
        }
    }

    if let Some(workplace_id) = colonist.assigned_workplace {
        if let Some((name, value)) = first_assignment_conflict(
            colonist,
            colonists,
            AssignmentPinLocation::Work(workplace_id),
        ) {
            return Some(AssignmentPinWarning {
                label: "TENSE".to_string(),
                detail: format!(
                    "{}: {} {:+} at W#{}. Pin another space or use Apart.",
                    name,
                    relationship_label(value),
                    value,
                    workplace_id
                ),
            });
        }
    }

    None
}

#[derive(Clone, Copy)]
pub(super) enum AssignmentPinLocation {
    Habitat(u32),
    Work(u32),
}

pub(super) fn first_assignment_conflict(
    colonist: &Colonist,
    colonists: &[Colonist],
    location: AssignmentPinLocation,
) -> Option<(String, i32)> {
    colonists
        .iter()
        .filter(|candidate| candidate.id != colonist.id)
        .filter(|candidate| match location {
            AssignmentPinLocation::Habitat(id) => candidate.assigned_habitat == Some(id),
            AssignmentPinLocation::Work(id) => candidate.assigned_workplace == Some(id),
        })
        .filter_map(|candidate| {
            let value = RelationshipDirectiveSystem::average_relationship(
                colonists,
                colonist.id,
                candidate.id,
            )
            .unwrap_or(0);
            RelationshipBand::from_value(value)
                .is_risk()
                .then(|| (candidate.name.clone(), value))
        })
        .min_by_key(|(_, value)| *value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::Trait;
    use crate::data::types::Position;

    fn test_colonist(id: u32) -> Colonist {
        Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        )
    }

    #[test]
    fn test_assign_visible_colonists_pin_selected_first() {
        let colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        let visible = assign_visible_colonists(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 0, 1, 2, 3]);
    }

    #[test]
    fn test_assign_visible_colonists_page_through_roster() {
        let colonists = (0..8).map(test_colonist).collect::<Vec<_>>();
        let page = assign_visible_colonists(
            &colonists,
            Some(5),
            1,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(
            assign_roster_page_count(&colonists, Some(5), AssignRosterFilter::All, None, None),
            2
        );
        assert_eq!(page, vec![5, 4, 6, 7]);
    }

    #[test]
    fn test_assign_visible_colonists_filter_pinned_and_sort_mood() {
        let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        colonists[1].assigned_habitat = Some(3);
        colonists[1].mood = 42.0;
        colonists[4].assigned_workplace = Some(8);
        colonists[4].mood = 21.0;

        let visible = assign_visible_colonists(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::Pinned,
            AssignRosterSort::Mood,
            None,
            None,
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 4, 1]);
    }

    #[test]
    fn test_assign_visible_colonists_filter_role() {
        let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        colonists[1].job_preference = JobPreference::Cook;
        colonists[3].job_preference = JobPreference::Cook;
        colonists[4].job_preference = JobPreference::Explorer;

        let visible = assign_visible_colonists(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            Some(JobPreference::Cook),
            None,
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 1, 3]);
    }

    #[test]
    fn test_assign_visible_colonists_filter_building_instance() {
        let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        colonists[1].assigned_habitat = Some(7);
        colonists[3].assigned_workplace = Some(7);
        colonists[4].assigned_habitat = Some(8);

        let visible = assign_visible_colonists(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            Some(7),
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 1, 3]);
    }

    #[test]
    fn test_assign_pair_action_reports_active_directive() {
        let mut colonists = vec![test_colonist(1), test_colonist(2)];
        colonists[0].relationships.insert(2, -24);
        colonists[1].relationships.insert(1, -20);
        colonists[0].avoided_partner_id = Some(2);
        colonists[1].avoided_partner_id = Some(1);

        let action = assign_pair_action(&colonists, 1, 2).unwrap();

        assert_eq!(action.directive, PairDirective::Separate);
        assert_eq!(action.label, "Apart set -22");
    }

    #[test]
    fn test_selected_assignment_label_reports_room_pins() {
        let mut colonist = test_colonist(1);
        assert_eq!(selected_assignment_label(&colonist), "H-- W--");

        colonist.assigned_habitat = Some(3);
        colonist.assigned_workplace = Some(8);

        assert_eq!(selected_assignment_label(&colonist), "H#3 W#8");
        assert!(selected_assignment_detail(
            &colonist,
            &[colonist.clone()],
            &TechnologyState::default()
        )
        .contains("H#3 W#8"));
    }

    #[test]
    fn test_assignment_pin_warning_flags_over_capacity_habitat() {
        let mut colonists = vec![test_colonist(1), test_colonist(2), test_colonist(3)];
        for colonist in &mut colonists {
            colonist.assigned_habitat = Some(7);
        }

        let warning =
            assignment_pin_warning(&colonists[0], &colonists, &TechnologyState::default()).unwrap();

        assert_eq!(warning.label, "CAP");
        assert!(warning.detail.contains("3/2"));
    }

    #[test]
    fn test_assignment_pin_warning_flags_tense_shared_workplace() {
        let mut colonists = vec![test_colonist(1), test_colonist(2)];
        colonists[0].assigned_workplace = Some(9);
        colonists[1].assigned_workplace = Some(9);
        colonists[0].relationships.insert(2, -24);
        colonists[1].relationships.insert(1, -20);

        let warning =
            assignment_pin_warning(&colonists[0], &colonists, &TechnologyState::default()).unwrap();

        assert_eq!(warning.label, "TENSE");
        assert!(warning.detail.contains("Colonist 2"));
        assert!(warning.detail.contains("W#9"));
    }
}
