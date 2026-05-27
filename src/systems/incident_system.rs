use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::incident::IncidentType;
use crate::systems::resource_system::ResourceSystem;
use crate::systems::summary_system::SummarySystem;
use crate::systems::time_system::TimeSystem;

const INCIDENT_ADVISOR_DURATION: u64 = TimeSystem::TICKS_PER_HOUR * 10;

pub struct IncidentSystem;

impl IncidentSystem {
    pub fn process_hourly_incidents(state: &mut GameState) {
        state.incidents.clear_expired(state.tick);

        let (day, hour, _) = TimeSystem::get_time_of_day(state.tick);
        let Some(incident_type) = scheduled_incident(day, hour) else {
            return;
        };

        if state.incidents.has_triggered(incident_type) {
            return;
        }

        state.incidents.mark_triggered(incident_type);
        state
            .incidents
            .activate(incident_type, state.tick, INCIDENT_ADVISOR_DURATION);
        Self::apply_incident(state, incident_type);
        ResourceSystem::update_condition(state);
    }

    fn apply_incident(state: &mut GameState, incident_type: IncidentType) {
        match incident_type {
            IncidentType::RationSpoilage => Self::ration_spoilage(state),
            IncidentType::HabitatConflict => Self::habitat_conflict(state),
            IncidentType::ToolBreakage => Self::tool_breakage(state),
            IncidentType::MoraleDip => Self::morale_dip(state),
            IncidentType::StormWarning => Self::storm_warning(state),
        }
    }

    fn ration_spoilage(state: &mut GameState) {
        let loss = (ResourceSystem::daily_supply_need(state).max(1) + 1).min(4);
        let lost = state.resources.supplies.min(loss);
        state.resources.supplies -= lost;
        state.push_log(
            LogCategory::Resource,
            IncidentType::RationSpoilage.title(),
            format!(
                "Moisture ruined {} supplies. The colony needs replacement food before dawn.",
                lost
            ),
        );
    }

    fn habitat_conflict(state: &mut GameState) {
        for colonist in &mut state.colonists {
            colonist.mood = (colonist.mood - 3.0).clamp(0.0, 100.0);
        }

        if let Some(pair) = SummarySystem::colony_pressure_summary(state).weakest_pair {
            Self::shift_relationship(state, &pair.first_name, &pair.second_name, -4);
            state.push_log(
                LogCategory::Social,
                IncidentType::HabitatConflict.title(),
                format!(
                    "{} and {} argued over recovery space. Relationship pressure rose and mood fell.",
                    pair.first_name, pair.second_name
                ),
            );
        } else {
            state.push_log(
                LogCategory::Social,
                IncidentType::HabitatConflict.title(),
                "Crowded recovery space lowered mood across the settlement.",
            );
        }
    }

    fn tool_breakage(state: &mut GameState) {
        let loss = state.resources.salvage.min(3);
        state.resources.salvage -= loss;
        state.resources.workshop_progress = 0;
        state.push_log(
            LogCategory::Resource,
            IncidentType::ToolBreakage.title(),
            format!(
                "A field kit cracked during repair work. {} salvage was spent replacing tools.",
                loss
            ),
        );
    }

    fn morale_dip(state: &mut GameState) {
        for colonist in &mut state.colonists {
            colonist.mood = (colonist.mood - 6.0).clamp(0.0, 100.0);
        }

        state.push_log(
            LogCategory::Mood,
            IncidentType::MoraleDip.title(),
            "The crash anniversary hit hard. Recovery priority can soften the next work block.",
        );
    }

    fn storm_warning(state: &mut GameState) {
        for colonist in &mut state.colonists {
            colonist.mood = (colonist.mood - 2.0).clamp(0.0, 100.0);
        }

        state.push_log(
            LogCategory::Colony,
            IncidentType::StormWarning.title(),
            "Static in the relay points to a rough night. Avoid sending a weak crew unless supplies are secure.",
        );
    }

    fn shift_relationship(state: &mut GameState, name_a: &str, name_b: &str, delta: i32) {
        let Some(id_a) = state
            .colonists
            .iter()
            .find(|colonist| colonist.name == name_a)
            .map(|colonist| colonist.id)
        else {
            return;
        };
        let Some(id_b) = state
            .colonists
            .iter()
            .find(|colonist| colonist.name == name_b)
            .map(|colonist| colonist.id)
        else {
            return;
        };

        for (from_id, to_id) in [(id_a, id_b), (id_b, id_a)] {
            if let Some(colonist) = state
                .colonists
                .iter_mut()
                .find(|colonist| colonist.id == from_id)
            {
                let value = colonist.relationships.entry(to_id).or_insert(0);
                *value = (*value + delta).clamp(-50, 50);
            }
        }
    }
}

fn scheduled_incident(day: u32, hour: u32) -> Option<IncidentType> {
    match (day, hour) {
        (2, 12) => Some(IncidentType::RationSpoilage),
        (3, 21) => Some(IncidentType::HabitatConflict),
        (4, 10) => Some(IncidentType::ToolBreakage),
        (5, 15) => Some(IncidentType::MoraleDip),
        (6, 18) => Some(IncidentType::StormWarning),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{Colonist, JobPreference, Trait};
    use crate::data::types::Position;

    #[test]
    fn test_ration_spoilage_uses_existing_supplies_and_advisor_state() {
        let mut state = GameState::new();
        state.tick = TimeSystem::TICKS_PER_DAY + TimeSystem::TICKS_PER_HOUR * 12;
        state.resources.supplies = 10;
        state.colonists.push(Colonist::new(
            1,
            "Ari".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ));

        IncidentSystem::process_hourly_incidents(&mut state);

        assert!(state.resources.supplies < 10);
        assert!(state.incidents.has_triggered(IncidentType::RationSpoilage));
        assert_eq!(
            state.incidents.active_incident(state.tick),
            Some(IncidentType::RationSpoilage)
        );
        assert!(state
            .event_log
            .iter()
            .any(|entry| entry.title == IncidentType::RationSpoilage.title()));
    }
}
