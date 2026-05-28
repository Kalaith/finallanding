mod launch;
mod planning;
mod resolution;

use crate::data::game_state::GameState;
use crate::data::mission::MissionType;

pub use launch::LaunchMissionError;
pub use planning::MissionPlan;

pub struct MissionSystem;

impl MissionSystem {
    pub fn mission_plans(state: &GameState) -> Vec<MissionPlan> {
        planning::MissionPlanning::mission_plans(state)
    }

    pub fn recommended_mission_type(state: &GameState) -> MissionType {
        planning::MissionPlanning::recommended_mission_type(state)
    }

    pub fn launch_mission(
        state: &mut GameState,
        mission_type: MissionType,
    ) -> Result<u32, LaunchMissionError> {
        launch::MissionLaunch::launch_mission(state, mission_type)
    }

    pub fn process_completed_missions(state: &mut GameState) {
        resolution::MissionResolution::process_completed_missions(state);
    }

    pub fn recover_injured_colonists(state: &mut GameState) {
        resolution::MissionResolution::recover_injured_colonists(state);
    }
}
