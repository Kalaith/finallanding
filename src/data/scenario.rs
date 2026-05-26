use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioOutcome {
    InProgress,
    Victory,
    Failure,
}

impl ScenarioOutcome {
    pub fn label(&self) -> &'static str {
        match self {
            ScenarioOutcome::InProgress => "In Progress",
            ScenarioOutcome::Victory => "Stable Landing",
            ScenarioOutcome::Failure => "Colony Failed",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioState {
    pub target_day: u32,
    pub required_tech_unlocks: usize,
    pub outcome: ScenarioOutcome,
    pub outcome_tick: Option<u64>,
}

impl Default for ScenarioState {
    fn default() -> Self {
        Self {
            target_day: 7,
            required_tech_unlocks: 3,
            outcome: ScenarioOutcome::InProgress,
            outcome_tick: None,
        }
    }
}

impl ScenarioState {
    pub fn is_finished(&self) -> bool {
        self.outcome != ScenarioOutcome::InProgress
    }
}
