use super::*;

impl GameplayState {
    pub(super) fn launch_recommended_mission(&mut self) {
        let mission_type = MissionSystem::recommended_mission_type(&self.data);
        self.launch_mission(mission_type);
    }

    pub(super) fn launch_mission(&mut self, mission_type: MissionType) {
        if let Err(error) = MissionSystem::launch_mission(&mut self.data, mission_type) {
            let definition = mission_type.definition();
            let (title, detail) = match error {
                crate::systems::mission_system::LaunchMissionError::NoExplorationGate => (
                    "No Exploration Gate",
                    format!(
                        "Build an Exploration Gate before sending {}.",
                        definition.name
                    ),
                ),
                crate::systems::mission_system::LaunchMissionError::NoAvailableColonist => (
                    "No available mission crew",
                    format!(
                        "{} needs a colonist who is not away or hurt.",
                        definition.name
                    ),
                ),
                crate::systems::mission_system::LaunchMissionError::MissionCooldown {
                    remaining_ticks,
                } => (
                    "Mission crew regrouping",
                    format!(
                        "Wait {} more minutes before launching another mission.",
                        remaining_ticks
                    ),
                ),
            };

            self.data.push_log(LogCategory::Mission, title, detail);
        }
    }

    pub(super) fn handle_research_toolbar_click(
        &mut self,
        context: Rect,
        mouse_x: f32,
        mouse_y: f32,
    ) {
        if let Some(mission_type) = toolbar_mission_at(context, mouse_x, mouse_y) {
            self.launch_mission(mission_type);
        }
    }
}
