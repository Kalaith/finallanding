use crate::data::colonist::{Colonist, JobPreference, RelationshipBand};

pub const ASSIGN_ROSTER_SLOT_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignRosterFilter {
    All,
    Risk,
    Support,
    Pinned,
}

impl AssignRosterFilter {
    pub fn all() -> &'static [AssignRosterFilter] {
        &[
            AssignRosterFilter::All,
            AssignRosterFilter::Risk,
            AssignRosterFilter::Support,
            AssignRosterFilter::Pinned,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            AssignRosterFilter::All => "ALL",
            AssignRosterFilter::Risk => "RISK",
            AssignRosterFilter::Support => "SUP",
            AssignRosterFilter::Pinned => "PIN",
        }
    }

    pub fn tooltip_title(self) -> &'static str {
        match self {
            AssignRosterFilter::All => "All survivors",
            AssignRosterFilter::Risk => "Relationship risks",
            AssignRosterFilter::Support => "Support ties",
            AssignRosterFilter::Pinned => "Pinned spaces",
        }
    }

    pub fn tooltip_body(self) -> &'static str {
        match self {
            AssignRosterFilter::All => "Show the full roster around the selected survivor.",
            AssignRosterFilter::Risk => {
                "Show survivors with tense or hostile relationship pressure."
            }
            AssignRosterFilter::Support => "Show survivors with friendly or close support ties.",
            AssignRosterFilter::Pinned => "Show survivors with manual Habitat or work pins.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignRosterSort {
    Roster,
    Mood,
    Bond,
}

impl AssignRosterSort {
    pub fn all() -> &'static [AssignRosterSort] {
        &[
            AssignRosterSort::Roster,
            AssignRosterSort::Mood,
            AssignRosterSort::Bond,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            AssignRosterSort::Roster => "ORD",
            AssignRosterSort::Mood => "MOOD",
            AssignRosterSort::Bond => "BOND",
        }
    }

    pub fn tooltip_title(self) -> &'static str {
        match self {
            AssignRosterSort::Roster => "Roster order",
            AssignRosterSort::Mood => "Low mood first",
            AssignRosterSort::Bond => "Strongest ties first",
        }
    }

    pub fn tooltip_body(self) -> &'static str {
        match self {
            AssignRosterSort::Roster => "Use the colony's original survivor order.",
            AssignRosterSort::Mood => "Put survivors with lower mood first.",
            AssignRosterSort::Bond => {
                "Put survivors with the strongest relationship pressure or support first."
            }
        }
    }
}

pub fn assign_roster_page_count(
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> usize {
    let selected_exists = selected_colonist_id
        .and_then(|id| colonists.iter().position(|colonist| colonist.id == id))
        .is_some();
    let other_count = (0..colonists.len())
        .filter(|index| Some(colonists[*index].id) != selected_colonist_id)
        .filter(|index| {
            assign_roster_filter_matches(&colonists[*index], active_filter, active_role_filter)
                && assign_building_filter_matches(&colonists[*index], active_building_filter)
        })
        .count();
    let page_size = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(usize::from(selected_exists));

    ((other_count + page_size - 1) / page_size).max(1)
}

pub fn assign_visible_colonist_indices(
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    page: usize,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<usize> {
    let mut indices = Vec::new();

    let selected_index =
        selected_colonist_id.and_then(|id| colonists.iter().position(|colonist| colonist.id == id));

    if let Some(index) = selected_index {
        indices.push(index);
    }

    let open_slots = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(indices.len());
    let page = page.min(
        assign_roster_page_count(
            colonists,
            selected_colonist_id,
            active_filter,
            active_role_filter,
            active_building_filter,
        ) - 1,
    );

    let roster = assign_sorted_roster_indices(
        colonists,
        selected_index,
        active_filter,
        active_sort,
        active_role_filter,
        active_building_filter,
    );
    indices.extend(roster.into_iter().skip(page * open_slots).take(open_slots));

    indices
}

pub fn assign_visible_colonists<'a>(
    colonists: &'a [Colonist],
    selected_colonist_id: Option<u32>,
    page: usize,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<&'a Colonist> {
    assign_visible_colonist_indices(
        colonists,
        selected_colonist_id,
        page,
        active_filter,
        active_sort,
        active_role_filter,
        active_building_filter,
    )
    .into_iter()
    .map(|index| &colonists[index])
    .collect()
}

pub fn assign_sorted_roster_indices(
    colonists: &[Colonist],
    selected_index: Option<usize>,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<usize> {
    let mut indices = (0..colonists.len())
        .filter(|index| Some(*index) != selected_index)
        .filter(|index| {
            assign_roster_filter_matches(&colonists[*index], active_filter, active_role_filter)
                && assign_building_filter_matches(&colonists[*index], active_building_filter)
        })
        .collect::<Vec<_>>();

    match active_sort {
        AssignRosterSort::Roster => {}
        AssignRosterSort::Mood => indices.sort_by(|left, right| {
            colonists[*left]
                .mood
                .partial_cmp(&colonists[*right].mood)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| colonists[*left].id.cmp(&colonists[*right].id))
        }),
        AssignRosterSort::Bond => indices.sort_by(|left, right| {
            relationship_pressure_score(&colonists[*right])
                .cmp(&relationship_pressure_score(&colonists[*left]))
                .then_with(|| colonists[*left].id.cmp(&colonists[*right].id))
        }),
    }

    indices
}

pub fn assign_roster_filter_matches(
    colonist: &Colonist,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
) -> bool {
    let relationship_match = match active_filter {
        AssignRosterFilter::All => true,
        AssignRosterFilter::Risk => colonist
            .relationships
            .values()
            .any(|value| RelationshipBand::from_value(*value).is_risk()),
        AssignRosterFilter::Support => colonist
            .relationships
            .values()
            .any(|value| RelationshipBand::from_value(*value).is_support()),
        AssignRosterFilter::Pinned => {
            colonist.assigned_habitat.is_some() || colonist.assigned_workplace.is_some()
        }
    };
    relationship_match && active_role_filter.is_none_or(|role| colonist.job_preference == role)
}

pub fn relationship_pressure_score(colonist: &Colonist) -> i32 {
    colonist
        .relationships
        .values()
        .map(|value| value.abs())
        .max()
        .unwrap_or(0)
}

pub fn assign_building_filter_matches(colonist: &Colonist, building_id: Option<u32>) -> bool {
    building_id.is_none_or(|id| {
        colonist.assigned_habitat == Some(id) || colonist.assigned_workplace == Some(id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::Trait;
    use crate::data::types::Position;

    fn roster_colonists(count: u32) -> Vec<Colonist> {
        (0..count)
            .map(|id| {
                Colonist::new(
                    id,
                    format!("Colonist {}", id),
                    Position::new(id as i32, 0),
                    Trait::HardWorker,
                    JobPreference::Builder,
                )
            })
            .collect()
    }

    #[test]
    fn test_assign_visible_indices_pin_selected_colonist_first() {
        let colonists = roster_colonists(6);

        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                0,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                None,
            ),
            vec![5, 0, 1, 2, 3]
        );
        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                None,
                0,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                None,
            ),
            vec![0, 1, 2, 3, 4]
        );
    }

    #[test]
    fn test_assign_visible_indices_page_through_remaining_colonists() {
        let colonists = roster_colonists(8);

        assert_eq!(
            assign_roster_page_count(&colonists, Some(5), AssignRosterFilter::All, None, None),
            2
        );
        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                1,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                None,
            ),
            vec![5, 4, 6, 7]
        );
        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                None,
                1,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                None,
            ),
            vec![5, 6, 7]
        );
    }

    #[test]
    fn test_assign_visible_indices_filter_and_sort_pressure() {
        let mut colonists = roster_colonists(6);
        colonists[1].relationships.insert(2, -12);
        colonists[3].relationships.insert(4, -34);
        colonists[4].relationships.insert(3, 22);

        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                0,
                AssignRosterFilter::Risk,
                AssignRosterSort::Bond,
                None,
                None,
            ),
            vec![5, 3, 1]
        );
    }

    #[test]
    fn test_assign_visible_indices_filter_role() {
        let mut colonists = roster_colonists(6);
        colonists[1].job_preference = JobPreference::Cook;
        colonists[4].job_preference = JobPreference::Cook;

        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                0,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                Some(JobPreference::Cook),
                None,
            ),
            vec![5, 1, 4]
        );
    }

    #[test]
    fn test_assign_visible_indices_filter_building_instance() {
        let mut colonists = roster_colonists(6);
        colonists[1].assigned_habitat = Some(7);
        colonists[3].assigned_workplace = Some(7);
        colonists[4].assigned_habitat = Some(8);

        assert_eq!(
            assign_visible_colonist_indices(
                &colonists,
                Some(5),
                0,
                AssignRosterFilter::All,
                AssignRosterSort::Roster,
                None,
                Some(7),
            ),
            vec![5, 1, 3]
        );
    }
}
