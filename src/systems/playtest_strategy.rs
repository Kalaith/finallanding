use crate::data::schedule::ActivityType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaytestStrategyKind {
    Reference,
    Conservative,
    SurveyHeavy,
    RecoveryHeavy,
    NoFood,
    NoHabitats,
    NoMissions,
}

impl PlaytestStrategyKind {
    pub fn report_set() -> &'static [PlaytestStrategyKind] {
        &[
            PlaytestStrategyKind::Reference,
            PlaytestStrategyKind::Conservative,
            PlaytestStrategyKind::SurveyHeavy,
            PlaytestStrategyKind::RecoveryHeavy,
            PlaytestStrategyKind::NoFood,
            PlaytestStrategyKind::NoHabitats,
            PlaytestStrategyKind::NoMissions,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            PlaytestStrategyKind::Reference => "Reference",
            PlaytestStrategyKind::Conservative => "Conservative",
            PlaytestStrategyKind::SurveyHeavy => "Survey heavy",
            PlaytestStrategyKind::RecoveryHeavy => "Recovery heavy",
            PlaytestStrategyKind::NoFood => "No food",
            PlaytestStrategyKind::NoHabitats => "No habitats",
            PlaytestStrategyKind::NoMissions => "No missions",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ReferenceStrategy {
    pub(super) kind: PlaytestStrategyKind,
    pub(super) missions_completed: u32,
    pub(super) next_mission_tick: u64,
}

impl ReferenceStrategy {
    pub(super) fn new(kind: PlaytestStrategyKind) -> Self {
        Self {
            kind,
            missions_completed: 0,
            next_mission_tick: 0,
        }
    }
}

pub(super) fn activity_for_hour(hour: u32) -> ActivityType {
    match hour {
        6 | 20 | 21 => ActivityType::Eat,
        7..=17 => ActivityType::Work,
        22..=23 | 0..=5 => ActivityType::Sleep,
        _ => ActivityType::Relax,
    }
}
