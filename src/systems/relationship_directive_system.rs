use crate::data::colonist::{relationship_label, Colonist, RelationshipBand};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairDirective {
    Pair,
    Separate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectiveChange {
    Set(PairDirective),
    Cleared(PairDirective),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectiveError {
    MissingColonist,
    SameColonist,
}

pub struct RelationshipDirectiveSystem;

impl RelationshipDirectiveSystem {
    pub fn directive_for_pair(
        colonists: &[Colonist],
        first_id: u32,
        second_id: u32,
    ) -> Option<PairDirective> {
        let first = colonist_by_id(colonists, first_id)?;
        let second = colonist_by_id(colonists, second_id)?;

        if first.preferred_partner_id == Some(second_id)
            && second.preferred_partner_id == Some(first_id)
        {
            Some(PairDirective::Pair)
        } else if first.avoided_partner_id == Some(second_id)
            && second.avoided_partner_id == Some(first_id)
        {
            Some(PairDirective::Separate)
        } else {
            None
        }
    }

    pub fn recommended_directive(
        colonists: &[Colonist],
        first_id: u32,
        second_id: u32,
    ) -> Option<PairDirective> {
        let value = Self::average_relationship(colonists, first_id, second_id)?;
        if RelationshipBand::from_value(value).is_risk() {
            Some(PairDirective::Separate)
        } else {
            Some(PairDirective::Pair)
        }
    }

    pub fn average_relationship(
        colonists: &[Colonist],
        first_id: u32,
        second_id: u32,
    ) -> Option<i32> {
        let first = colonist_by_id(colonists, first_id)?;
        let second = colonist_by_id(colonists, second_id)?;
        let first_value = first.relationships.get(&second_id).copied().unwrap_or(0);
        let second_value = second.relationships.get(&first_id).copied().unwrap_or(0);

        Some(if first_value == 0 {
            second_value
        } else if second_value == 0 {
            first_value
        } else {
            (first_value + second_value) / 2
        })
    }

    pub fn toggle_pair_directive(
        colonists: &mut [Colonist],
        first_id: u32,
        second_id: u32,
    ) -> Result<DirectiveChange, DirectiveError> {
        if first_id == second_id {
            return Err(DirectiveError::SameColonist);
        }

        if colonist_by_id(colonists, first_id).is_none()
            || colonist_by_id(colonists, second_id).is_none()
        {
            return Err(DirectiveError::MissingColonist);
        }

        let next_directive = Self::recommended_directive(colonists, first_id, second_id)
            .ok_or(DirectiveError::MissingColonist)?;

        if Self::directive_for_pair(colonists, first_id, second_id) == Some(next_directive) {
            apply_directive(colonists, first_id, second_id, None);
            Ok(DirectiveChange::Cleared(next_directive))
        } else {
            apply_directive(colonists, first_id, second_id, Some(next_directive));
            Ok(DirectiveChange::Set(next_directive))
        }
    }

    pub fn directive_detail(colonists: &[Colonist], first_id: u32, second_id: u32) -> String {
        let Some(first) = colonist_by_id(colonists, first_id) else {
            return "Missing survivor.".to_string();
        };
        let Some(second) = colonist_by_id(colonists, second_id) else {
            return "Missing survivor.".to_string();
        };
        let value = Self::average_relationship(colonists, first_id, second_id).unwrap_or(0);
        let label = relationship_label(value);
        let current = Self::directive_for_pair(colonists, first_id, second_id);
        let action =
            match current.or_else(|| Self::recommended_directive(colonists, first_id, second_id)) {
                Some(PairDirective::Pair) => "Pair for shared work and recovery choices",
                Some(PairDirective::Separate) => {
                    "Keep apart when another room or workplace is available"
                }
                None => "No directive available",
            };

        format!(
            "{} and {} are {} ({:+}). {}.",
            first.name, second.name, label, value, action
        )
    }
}

impl PairDirective {
    pub fn label(self) -> &'static str {
        match self {
            PairDirective::Pair => "Pair",
            PairDirective::Separate => "Apart",
        }
    }

    pub fn log_title(self) -> &'static str {
        match self {
            PairDirective::Pair => "Pair directive",
            PairDirective::Separate => "Space directive",
        }
    }
}

fn colonist_by_id(colonists: &[Colonist], id: u32) -> Option<&Colonist> {
    colonists.iter().find(|colonist| colonist.id == id)
}

fn apply_directive(
    colonists: &mut [Colonist],
    first_id: u32,
    second_id: u32,
    directive: Option<PairDirective>,
) {
    for colonist in colonists {
        let partner_id = if colonist.id == first_id {
            Some(second_id)
        } else if colonist.id == second_id {
            Some(first_id)
        } else {
            None
        };

        let Some(partner_id) = partner_id else {
            continue;
        };

        if colonist.preferred_partner_id == Some(partner_id) {
            colonist.preferred_partner_id = None;
        }
        if colonist.avoided_partner_id == Some(partner_id) {
            colonist.avoided_partner_id = None;
        }

        match directive {
            Some(PairDirective::Pair) => colonist.preferred_partner_id = Some(partner_id),
            Some(PairDirective::Separate) => colonist.avoided_partner_id = Some(partner_id),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{JobPreference, Trait};
    use crate::data::types::Position;

    fn test_colonist(id: u32, name: &str) -> Colonist {
        Colonist::new(
            id,
            name.to_string(),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        )
    }

    #[test]
    fn test_toggle_pair_directive_pairs_supportive_colonists() {
        let mut colonists = vec![test_colonist(1, "Alice"), test_colonist(2, "Bob")];
        colonists[0].relationships.insert(2, 22);
        colonists[1].relationships.insert(1, 18);

        let change =
            RelationshipDirectiveSystem::toggle_pair_directive(&mut colonists, 1, 2).unwrap();

        assert_eq!(change, DirectiveChange::Set(PairDirective::Pair));
        assert_eq!(colonists[0].preferred_partner_id, Some(2));
        assert_eq!(colonists[1].preferred_partner_id, Some(1));
        assert_eq!(
            RelationshipDirectiveSystem::directive_for_pair(&colonists, 1, 2),
            Some(PairDirective::Pair)
        );
    }

    #[test]
    fn test_toggle_pair_directive_separates_tense_colonists() {
        let mut colonists = vec![test_colonist(1, "Alice"), test_colonist(2, "Bob")];
        colonists[0].relationships.insert(2, -24);
        colonists[1].relationships.insert(1, -18);

        let change =
            RelationshipDirectiveSystem::toggle_pair_directive(&mut colonists, 1, 2).unwrap();

        assert_eq!(change, DirectiveChange::Set(PairDirective::Separate));
        assert_eq!(colonists[0].avoided_partner_id, Some(2));
        assert_eq!(colonists[1].avoided_partner_id, Some(1));
    }

    #[test]
    fn test_toggle_pair_directive_clears_existing_directive() {
        let mut colonists = vec![test_colonist(1, "Alice"), test_colonist(2, "Bob")];
        colonists[0].relationships.insert(2, 12);
        colonists[1].relationships.insert(1, 16);

        RelationshipDirectiveSystem::toggle_pair_directive(&mut colonists, 1, 2).unwrap();
        let change =
            RelationshipDirectiveSystem::toggle_pair_directive(&mut colonists, 1, 2).unwrap();

        assert_eq!(change, DirectiveChange::Cleared(PairDirective::Pair));
        assert_eq!(colonists[0].preferred_partner_id, None);
        assert_eq!(colonists[1].preferred_partner_id, None);
    }
}
