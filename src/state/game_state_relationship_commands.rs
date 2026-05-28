use super::*;

impl GameplayState {
    pub(super) fn toggle_relationship_directive(&mut self, first_id: u32, second_id: u32) {
        let first_name = self
            .colonist_by_id(first_id)
            .map(|colonist| colonist.name.clone())
            .unwrap_or_else(|| format!("Colonist {}", first_id));
        let second_name = self
            .colonist_by_id(second_id)
            .map(|colonist| colonist.name.clone())
            .unwrap_or_else(|| format!("Colonist {}", second_id));

        let change = RelationshipDirectiveSystem::toggle_pair_directive(
            &mut self.data.colonists,
            first_id,
            second_id,
        );

        match change {
            Ok(DirectiveChange::Set(directive)) => {
                self.data.push_log(
                    LogCategory::Social,
                    directive.log_title(),
                    directive_log_detail(directive, &first_name, &second_name),
                );
            }
            Ok(DirectiveChange::Cleared(directive)) => {
                self.data.push_log(
                    LogCategory::Social,
                    "Relationship directive cleared",
                    format!(
                        "{} and {} no longer have a forced {} directive.",
                        first_name,
                        second_name,
                        directive.label().to_lowercase()
                    ),
                );
            }
            Err(_) => {
                self.data.push_log(
                    LogCategory::Social,
                    "Directive failed",
                    format!(
                        "Could not update a directive between {} and {}.",
                        first_name, second_name
                    ),
                );
            }
        }
    }
}
