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
    // Mood system (0.0 to 100.0)
    pub mood: f32,
    pub mood_modifiers: Vec<MoodModifier>,
    #[serde(default)]
    pub last_refusal_tick: u64,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityLocation {
    None,
    Building {
        building_id: u32,
        building_type: BuildingType,
    },
    Ground(Position),
}

impl Default for ActivityLocation {
    fn default() -> Self {
        Self::None
    }
}

impl ActivityLocation {
    pub fn building_id(&self) -> Option<u32> {
        match self {
            ActivityLocation::Building { building_id, .. } => Some(*building_id),
            _ => None,
        }
    }

    pub fn building_type(&self) -> Option<BuildingType> {
        match self {
            ActivityLocation::Building { building_type, .. } => Some(*building_type),
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
            mood: 50.0,
            mood_modifiers: Vec::new(),
            last_refusal_tick: 0,
        }
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

pub fn relationship_label(value: i32) -> &'static str {
    if value >= 30 {
        "Close"
    } else if value >= 10 {
        "Friendly"
    } else if value <= -30 {
        "Hostile"
    } else if value <= -10 {
        "Tense"
    } else {
        "Neutral"
    }
}
