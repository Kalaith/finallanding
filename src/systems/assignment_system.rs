use crate::data::colonist::{relationship_label, Colonist, JobPreference, RelationshipBand};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentPressure {
    Supported,
    Neutral,
    Tense,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleAssignmentForecast {
    pub target_role: JobPreference,
    pub pressure: AssignmentPressure,
    pub detail: String,
}

pub struct AssignmentSystem;

impl AssignmentSystem {
    pub fn forecast_role_change(
        colonists: &[Colonist],
        colonist_id: u32,
        target_role: JobPreference,
    ) -> RoleAssignmentForecast {
        let Some(colonist) = colonists
            .iter()
            .find(|candidate| candidate.id == colonist_id)
        else {
            return RoleAssignmentForecast {
                target_role,
                pressure: AssignmentPressure::Neutral,
                detail: "No survivor selected for assignment.".to_string(),
            };
        };

        let coworkers = colonists
            .iter()
            .filter(|candidate| {
                candidate.id != colonist.id && candidate.job_preference == target_role
            })
            .map(|candidate| {
                (
                    candidate.name.clone(),
                    average_relationship(colonist, candidate),
                )
            })
            .collect::<Vec<_>>();

        if coworkers.is_empty() {
            return RoleAssignmentForecast {
                target_role,
                pressure: AssignmentPressure::Neutral,
                detail: format!(
                    "{} would work this role without a current same-role partner.",
                    colonist.name
                ),
            };
        }

        let weakest = coworkers.iter().min_by_key(|(_, value)| *value);
        if let Some((name, value)) = weakest {
            if RelationshipBand::from_value(*value).is_risk() {
                return RoleAssignmentForecast {
                    target_role,
                    pressure: AssignmentPressure::Tense,
                    detail: format!(
                        "{} would share {} work with {} ({} {:+}).",
                        colonist.name,
                        target_role.label(),
                        name,
                        relationship_label(*value),
                        value
                    ),
                };
            }
        }

        let strongest = coworkers.iter().max_by_key(|(_, value)| *value);
        if let Some((name, value)) = strongest {
            if RelationshipBand::from_value(*value).is_support() {
                return RoleAssignmentForecast {
                    target_role,
                    pressure: AssignmentPressure::Supported,
                    detail: format!(
                        "{} would reinforce a {} bond with {} ({:+}).",
                        colonist.name,
                        relationship_label(*value),
                        name,
                        value
                    ),
                };
            }
        }

        RoleAssignmentForecast {
            target_role,
            pressure: AssignmentPressure::Neutral,
            detail: format!(
                "{} would join {} same-role partner(s). No strong social pressure yet.",
                colonist.name,
                coworkers.len()
            ),
        }
    }
}

fn average_relationship(first: &Colonist, second: &Colonist) -> i32 {
    let first_value = first.relationships.get(&second.id).copied().unwrap_or(0);
    let second_value = second.relationships.get(&first.id).copied().unwrap_or(0);
    (first_value + second_value) / 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{Colonist, Trait};
    use crate::data::types::Position;

    fn test_colonist(id: u32, name: &str, role: JobPreference) -> Colonist {
        Colonist::new(
            id,
            name.to_string(),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            role,
        )
    }

    #[test]
    fn test_forecast_flags_tense_same_role_assignment() {
        let mut alice = test_colonist(1, "Alice", JobPreference::Cook);
        let mut bob = test_colonist(2, "Bob", JobPreference::Builder);
        alice.relationships.insert(2, -24);
        bob.relationships.insert(1, -20);

        let forecast =
            AssignmentSystem::forecast_role_change(&[alice, bob], 1, JobPreference::Builder);

        assert_eq!(forecast.pressure, AssignmentPressure::Tense);
        assert!(forecast.detail.contains("Bob"));
        assert!(forecast.detail.contains("Tense"));
    }

    #[test]
    fn test_forecast_identifies_supportive_same_role_assignment() {
        let mut alice = test_colonist(1, "Alice", JobPreference::Cook);
        let mut bob = test_colonist(2, "Bob", JobPreference::Hauler);
        alice.relationships.insert(2, 30);
        bob.relationships.insert(1, 26);

        let forecast =
            AssignmentSystem::forecast_role_change(&[alice, bob], 1, JobPreference::Hauler);

        assert_eq!(forecast.pressure, AssignmentPressure::Supported);
        assert!(forecast.detail.contains("Bob"));
        assert!(forecast.detail.contains("Friendly"));
    }
}
