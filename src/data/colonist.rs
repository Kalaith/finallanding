use crate::data::building::BuildingType;
use crate::data::schedule::{ActivityType, Schedule};
use crate::data::types::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Colonist {
    pub id: u32,
    pub name: String,
    pub portrait_id: u32,
    pub position: Position,
    /// Visual position for smooth interpolation (in pixels)
    #[serde(skip)]
    pub visual_x: f32,
    #[serde(skip)]
    pub visual_y: f32,
    pub state: ColonistState,
    pub current_activity: ActivityType,
    pub activity_location: ActivityLocation,
    pub trait_data: Trait,
    pub job_preference: JobPreference,
    pub schedule: Schedule,
    /// Relationship values with other colonists (ID -> Value -50 to +50)
    pub relationships: HashMap<u32, i32>,
    /// Assigned Habitat Building ID
    pub assigned_habitat: Option<u32>,
    /// Assigned work building ID for the current role.
    #[serde(default)]
    pub assigned_workplace: Option<u32>,
    // Mood system (0.0 to 100.0)
    pub mood: f32,
    pub mood_modifiers: Vec<MoodModifier>,
    #[serde(default)]
    pub last_refusal_tick: u64,
    #[serde(default)]
    pub last_social_strain_tick: u64,
    #[serde(default)]
    pub injured_until_tick: Option<u64>,
    #[serde(default)]
    pub active_mission_id: Option<u32>,
    #[serde(default)]
    pub preferred_partner_id: Option<u32>,
    #[serde(default)]
    pub avoided_partner_id: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoodModifier {
    pub name: String,
    pub value: f32,
    pub duration_remaining: f32, // in game time (e.g. hours)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColonistState {
    Idle,
    Moving { target: Position },
    Working,
    Eating,
    Sleeping,
    OnMission { mission_id: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ActivityLocation {
    #[default]
    None,
    Building {
        building_id: u32,
        building_type: BuildingType,
    },
    Ground(Position),
}

impl ActivityLocation {
    pub fn building_id(&self) -> Option<u32> {
        match self {
            ActivityLocation::Building { building_id, .. } => Some(*building_id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trait {
    HardWorker,
    Lazy,
    FastWalker,
    Gourmet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobPreference {
    Explorer,
    Builder,
    Cook,
    Hauler,
    None,
}

impl JobPreference {
    pub fn all_assignable() -> &'static [JobPreference] {
        &[
            JobPreference::Explorer,
            JobPreference::Builder,
            JobPreference::Cook,
            JobPreference::Hauler,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            JobPreference::Explorer => "Explorer",
            JobPreference::Builder => "Builder",
            JobPreference::Cook => "Cook",
            JobPreference::Hauler => "Hauler",
            JobPreference::None => "General",
        }
    }

    pub fn next_assignable(self) -> JobPreference {
        let jobs = Self::all_assignable();
        let next_index = jobs
            .iter()
            .position(|job| *job == self)
            .map(|index| (index + 1) % jobs.len())
            .unwrap_or(0);
        jobs[next_index]
    }

    pub fn work_building_type(self) -> BuildingType {
        match self {
            JobPreference::Explorer => BuildingType::ExplorationGate,
            JobPreference::Builder => BuildingType::Workshop,
            JobPreference::Cook => BuildingType::MessHall,
            JobPreference::Hauler => BuildingType::Storage,
            JobPreference::None => BuildingType::Workshop,
        }
    }
}

impl Colonist {
    pub fn new(
        id: u32,
        name: String,
        position: Position,
        trait_data: Trait,
        job_preference: JobPreference,
    ) -> Self {
        Self {
            id,
            name,
            portrait_id: 0, // Placeholder
            position,
            visual_x: position.x as f32 * 32.0, // TILE_SIZE
            visual_y: position.y as f32 * 32.0,
            state: ColonistState::Idle,
            current_activity: ActivityType::Relax,
            activity_location: ActivityLocation::None,
            trait_data,
            job_preference,
            schedule: Schedule::default(),
            relationships: HashMap::new(),
            assigned_habitat: None,
            assigned_workplace: None,
            mood: 50.0,
            mood_modifiers: Vec::new(),
            last_refusal_tick: 0,
            last_social_strain_tick: 0,
            injured_until_tick: None,
            active_mission_id: None,
            preferred_partner_id: None,
            avoided_partner_id: None,
        }
    }

    pub fn is_hurt(&self, current_tick: u64) -> bool {
        self.injured_until_tick
            .is_some_and(|recovery_tick| recovery_tick > current_tick)
    }

    pub fn is_on_mission(&self) -> bool {
        self.active_mission_id.is_some() || matches!(self.state, ColonistState::OnMission { .. })
    }

    pub fn can_start_mission(&self, current_tick: u64) -> bool {
        !self.is_hurt(current_tick) && !self.is_on_mission()
    }

    pub fn recovery_minutes_remaining(&self, current_tick: u64) -> Option<u64> {
        self.injured_until_tick
            .map(|recovery_tick| recovery_tick.saturating_sub(current_tick))
            .filter(|remaining| *remaining > 0)
    }

    /// Interpolate visual position towards grid position
    pub fn update_visual_position(&mut self, speed: f32) {
        let target_x = self.position.x as f32 * 32.0;
        let target_y = self.position.y as f32 * 32.0;

        let dx = target_x - self.visual_x;
        let dy = target_y - self.visual_y;

        if dx.abs() > 0.5 {
            self.visual_x += dx.signum() * speed;
        } else {
            self.visual_x = target_x;
        }

        if dy.abs() > 0.5 {
            self.visual_y += dy.signum() * speed;
        } else {
            self.visual_y = target_y;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationshipBand {
    Hostile,
    Tense,
    Neutral,
    Friendly,
    Close,
}

impl RelationshipBand {
    pub fn from_value(value: i32) -> Self {
        if value >= 30 {
            RelationshipBand::Close
        } else if value >= 10 {
            RelationshipBand::Friendly
        } else if value <= -30 {
            RelationshipBand::Hostile
        } else if value <= -10 {
            RelationshipBand::Tense
        } else {
            RelationshipBand::Neutral
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RelationshipBand::Close => "Close",
            RelationshipBand::Friendly => "Friendly",
            RelationshipBand::Hostile => "Hostile",
            RelationshipBand::Tense => "Tense",
            RelationshipBand::Neutral => "Neutral",
        }
    }

    pub fn is_support(self) -> bool {
        matches!(self, RelationshipBand::Friendly | RelationshipBand::Close)
    }

    pub fn is_risk(self) -> bool {
        matches!(self, RelationshipBand::Tense | RelationshipBand::Hostile)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoodBand {
    Low,
    Strained,
    Steady,
}

impl MoodBand {
    pub fn from_mood(mood: f32) -> Self {
        if mood >= 65.0 {
            MoodBand::Steady
        } else if mood >= 35.0 {
            MoodBand::Strained
        } else {
            MoodBand::Low
        }
    }

    pub fn face(self) -> &'static str {
        match self {
            MoodBand::Steady => ":)",
            MoodBand::Strained => ":|",
            MoodBand::Low => ":(",
        }
    }
}

pub fn relationship_label(value: i32) -> &'static str {
    RelationshipBand::from_value(value).label()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_assignment_cycles_through_work_roles() {
        assert_eq!(
            JobPreference::Explorer.next_assignable(),
            JobPreference::Builder
        );
        assert_eq!(
            JobPreference::Builder.next_assignable(),
            JobPreference::Cook
        );
        assert_eq!(JobPreference::Cook.next_assignable(), JobPreference::Hauler);
        assert_eq!(
            JobPreference::Hauler.next_assignable(),
            JobPreference::Explorer
        );
        assert_eq!(
            JobPreference::None.next_assignable(),
            JobPreference::Explorer
        );
    }

    #[test]
    fn test_job_preferences_map_to_work_buildings() {
        assert_eq!(
            JobPreference::Explorer.work_building_type(),
            BuildingType::ExplorationGate
        );
        assert_eq!(
            JobPreference::Builder.work_building_type(),
            BuildingType::Workshop
        );
        assert_eq!(
            JobPreference::Cook.work_building_type(),
            BuildingType::MessHall
        );
        assert_eq!(
            JobPreference::Hauler.work_building_type(),
            BuildingType::Storage
        );
    }
}
