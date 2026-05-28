use crate::data::resources::ColonyCondition;
use crate::data::scenario::ScenarioOutcome;
use crate::systems::playtest_strategy::PlaytestStrategyKind;
use crate::systems::time_system::TimeSystem;

#[derive(Clone, Debug)]
pub struct PlaytestReport {
    pub start_tick: u64,
    pub strategy: PlaytestStrategyKind,
    pub end_tick: u64,
    pub estimated_normal_minutes: f32,
    pub outcome: ScenarioOutcome,
    pub condition: ColonyCondition,
    pub average_mood: f32,
    pub supplies: i32,
    pub salvage: i32,
    pub daily_supply_need: i32,
    pub technologies_unlocked: usize,
    pub required_technologies: usize,
    pub missions_completed: u32,
    pub buildings_placed: usize,
    pub incidents_triggered: usize,
}

impl PlaytestReport {
    pub fn proves_reference_run(&self) -> bool {
        self.outcome == ScenarioOutcome::Victory
            && (30.0..=40.0).contains(&self.estimated_normal_minutes)
            && self.condition != ColonyCondition::Critical
            && self.condition != ColonyCondition::Collapsed
            && self.supplies >= self.daily_supply_need.max(1)
            && self.technologies_unlocked >= self.required_technologies
            && self.missions_completed >= self.required_technologies as u32
            && self.buildings_placed >= 5
            && self.incidents_triggered >= 1
    }

    pub fn outcome_band(&self) -> PlaytestOutcomeBand {
        let reached_target_day = self.end_tick >= TimeSystem::TICKS_PER_DAY * 6;
        if self.outcome == ScenarioOutcome::Victory
            && self.condition == ColonyCondition::Stable
            && self.supplies >= self.daily_supply_need.max(1) * 2
            && self.average_mood >= 50.0
        {
            PlaytestOutcomeBand::Win
        } else if reached_target_day
            && self.supplies >= self.daily_supply_need.max(1)
            && self.technologies_unlocked >= self.required_technologies
            && (self.condition == ColonyCondition::Strained || self.average_mood < 50.0)
        {
            PlaytestOutcomeBand::Limp
        } else {
            PlaytestOutcomeBand::Fail
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaytestOutcomeBand {
    Win,
    Limp,
    Fail,
}

impl PlaytestOutcomeBand {
    pub fn label(self) -> &'static str {
        match self {
            PlaytestOutcomeBand::Win => "Win",
            PlaytestOutcomeBand::Limp => "Limp",
            PlaytestOutcomeBand::Fail => "Fail",
        }
    }
}
