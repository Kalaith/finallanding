use super::*;

pub(crate) fn directive_log_detail(
    directive: PairDirective,
    first_name: &str,
    second_name: &str,
) -> String {
    match directive {
        PairDirective::Pair => format!(
            "{} and {} will prefer the same work and recovery spaces when the settlement has a choice.",
            first_name, second_name
        ),
        PairDirective::Separate => format!(
            "{} and {} will avoid sharing work and recovery spaces when another option exists.",
            first_name, second_name
        ),
    }
}
