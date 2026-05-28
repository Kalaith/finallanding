pub(crate) use crate::data::assign_roster::{
    assign_roster_page_count, assign_visible_colonist_indices,
};
use crate::data::colonist::JobPreference;

pub(crate) fn next_assign_role_filter(current: Option<JobPreference>) -> Option<JobPreference> {
    match current {
        None => Some(JobPreference::Explorer),
        Some(JobPreference::Explorer) => Some(JobPreference::Builder),
        Some(JobPreference::Builder) => Some(JobPreference::Cook),
        Some(JobPreference::Cook) => Some(JobPreference::Hauler),
        Some(JobPreference::Hauler) | Some(JobPreference::None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_assign_role_filter_cycles_assignable_roles() {
        assert_eq!(next_assign_role_filter(None), Some(JobPreference::Explorer));
        assert_eq!(
            next_assign_role_filter(Some(JobPreference::Explorer)),
            Some(JobPreference::Builder)
        );
        assert_eq!(next_assign_role_filter(Some(JobPreference::Hauler)), None);
    }
}
