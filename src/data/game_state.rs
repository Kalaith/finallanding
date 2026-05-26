use super::grid::Grid;
use crate::data::event_log::{ColonyLogEntry, LogCategory};
use crate::data::mission::MissionState;
use crate::data::priority::PriorityState;
use crate::data::resources::ResourceState;
use crate::data::scenario::ScenarioState;
use crate::data::technology::TechnologyState;
use crate::game::building_system::BuildingSystem;
use crate::systems::time_system::TimeSystem;
use serde::{Deserialize, Serialize};

const MAX_EVENT_LOG_ENTRIES: usize = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSpeed {
    Paused = 0,
    Normal = 1,
    Fast = 2,
    SuperFast = 4,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeState {
    pub speed: TimeSpeed,
    pub day_length_ticks: u64,
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            speed: TimeSpeed::Normal,
            day_length_ticks: 1440, // 24 hours * 60 minutes
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameState {
    // Other agents will add fields here (e.g., world map, entities)
    pub tick: u64,
    pub time: TimeState,
    pub grid: Grid,

    pub colonists: Vec<crate::data::colonist::Colonist>,
    pub event_log: Vec<ColonyLogEntry>,
    pub resources: ResourceState,
    pub missions: MissionState,
    pub priority: PriorityState,
    pub technology: TechnologyState,
    pub scenario: ScenarioState,

    /// Building placement system
    #[serde(skip)]
    pub building_system: BuildingSystem,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            time: TimeState::default(),
            grid: Grid::default(),

            colonists: Vec::new(),
            event_log: Vec::new(),
            resources: ResourceState::default(),
            missions: MissionState::default(),
            priority: PriorityState::default(),
            technology: TechnologyState::default(),
            scenario: ScenarioState::default(),
            building_system: BuildingSystem::new(),
        }
    }

    pub fn push_log(
        &mut self,
        category: LogCategory,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) {
        let (day, hour, minute) = TimeSystem::get_time_of_day(self.tick);
        self.event_log.push(ColonyLogEntry::new(
            day,
            hour,
            minute,
            category,
            title.into(),
            detail.into(),
        ));

        if self.event_log.len() > MAX_EVENT_LOG_ENTRIES {
            let overflow = self.event_log.len() - MAX_EVENT_LOG_ENTRIES;
            self.event_log.drain(0..overflow);
        }
    }
}
