use crate::data::building::BuildingType;
use crate::data::colonist::{ActivityLocation, ColonistState, JobPreference};
use crate::data::event_log::LogCategory;
use crate::data::game_state::GameState;
use crate::data::mission::{ActiveMission, MissionType};
use crate::systems::mission_system::planning::MissionPlanning;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaunchMissionError {
    NoExplorationGate,
    NoAvailableColonist,
    MissionCooldown { remaining_ticks: u64 },
}

pub(super) struct MissionLaunch;

impl MissionLaunch {
    pub(super) fn launch_mission(
        state: &mut GameState,
        mission_type: MissionType,
    ) -> Result<u32, LaunchMissionError> {
        if !state
            .building_system
            .buildings()
            .iter()
            .any(|building| building.building_type == BuildingType::ExplorationGate)
        {
            return Err(LaunchMissionError::NoExplorationGate);
        }

        let cooldown_remaining = state.missions.cooldown_remaining(state.tick);
        if cooldown_remaining > 0 {
            return Err(LaunchMissionError::MissionCooldown {
                remaining_ticks: cooldown_remaining,
            });
        }

        let Some(colonist_index) = Self::find_available_mission_colonist(state) else {
            return Err(LaunchMissionError::NoAvailableColonist);
        };

        let definition = mission_type.definition();
        let mission_id = state.missions.next_id;
        state.missions.next_id += 1;
        let danger_percent = MissionPlanning::mission_danger_percent(state, mission_type);
        let completes_at_tick = state.tick + definition.duration_minutes;
        let priority = state.priority.active;

        let colonist_name = state.colonists[colonist_index].name.clone();
        state.colonists[colonist_index].state = ColonistState::OnMission { mission_id };
        state.colonists[colonist_index].current_activity =
            crate::data::schedule::ActivityType::Work;
        state.colonists[colonist_index].activity_location = ActivityLocation::None;
        state.colonists[colonist_index].active_mission_id = Some(mission_id);

        state.missions.active_missions.push(ActiveMission {
            id: mission_id,
            colonist_id: state.colonists[colonist_index].id,
            mission_type,
            started_tick: state.tick,
            completes_at_tick,
            danger_percent,
            priority,
        });
        state.missions.next_launch_tick = state.tick + definition.cooldown_minutes;

        state.push_log(
            LogCategory::Mission,
            format!("{} started {}", colonist_name, definition.name),
            format!(
                "{} Duration {}m, danger {}% after {} priority. Crew regroups for {}m.",
                definition.reward_profile,
                definition.duration_minutes,
                danger_percent,
                priority.label(),
                definition.cooldown_minutes
            ),
        );

        Ok(mission_id)
    }

    fn find_available_mission_colonist(state: &GameState) -> Option<usize> {
        state
            .colonists
            .iter()
            .position(|colonist| {
                colonist.job_preference == JobPreference::Explorer
                    && colonist.can_start_mission(state.tick)
            })
            .or_else(|| {
                state
                    .colonists
                    .iter()
                    .position(|colonist| colonist.can_start_mission(state.tick))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::MissionSystem;
    use super::*;
    use crate::data::colonist::{Colonist, Trait};
    use crate::data::types::Position;

    fn add_gate(state: &mut GameState) {
        state.building_system.try_place_building(
            &mut state.grid,
            BuildingType::ExplorationGate,
            Position::new(0, 0),
        );
    }

    #[test]
    fn test_launch_creates_typed_mission_and_starts_cooldown() {
        let mut state = GameState::new();
        add_gate(&mut state);
        state.colonists.push(Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        ));

        let mission_id =
            MissionSystem::launch_mission(&mut state, MissionType::DeepSurvey).unwrap();

        assert_eq!(mission_id, 1);
        assert_eq!(
            state.missions.active_missions[0].mission_type,
            MissionType::DeepSurvey
        );
        assert_eq!(
            state.missions.active_missions[0].remaining_ticks(state.tick),
            MissionType::DeepSurvey.definition().duration_minutes
        );
        assert_eq!(
            state.missions.cooldown_remaining(state.tick),
            MissionType::DeepSurvey.definition().cooldown_minutes
        );
        assert!(state.colonists[0].is_on_mission());
    }

    #[test]
    fn test_hurt_colonist_cannot_launch() {
        let mut state = GameState::new();
        add_gate(&mut state);
        let mut colonist = Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        );
        colonist.injured_until_tick = Some(10);
        state.colonists.push(colonist);

        assert_eq!(
            MissionSystem::launch_mission(&mut state, MissionType::PerimeterScan),
            Err(LaunchMissionError::NoAvailableColonist)
        );
    }

    #[test]
    fn test_mission_cooldown_blocks_rapid_relaunch() {
        let mut state = GameState::new();
        add_gate(&mut state);
        state.colonists.push(Colonist::new(
            1,
            "Scout".to_string(),
            Position::new(3, 3),
            Trait::FastWalker,
            JobPreference::Explorer,
        ));
        state.colonists.push(Colonist::new(
            2,
            "Backup".to_string(),
            Position::new(4, 3),
            Trait::HardWorker,
            JobPreference::Builder,
        ));

        MissionSystem::launch_mission(&mut state, MissionType::SupplyRun).unwrap();

        assert_eq!(
            MissionSystem::launch_mission(&mut state, MissionType::PerimeterScan),
            Err(LaunchMissionError::MissionCooldown {
                remaining_ticks: MissionType::SupplyRun.definition().cooldown_minutes
            })
        );
    }
}
