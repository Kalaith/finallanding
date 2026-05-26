use crate::data::priority::ColonyPriority;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionItem {
    StructuralAlloy,
    AlienCircuit,
    MedicinalGel,
    NutrientPods,
    SalvageCache,
}

impl MissionItem {
    pub fn name(&self) -> &'static str {
        match self {
            MissionItem::StructuralAlloy => "Structural Alloy",
            MissionItem::AlienCircuit => "Alien Circuit",
            MissionItem::MedicinalGel => "Medicinal Gel",
            MissionItem::NutrientPods => "Nutrient Pods",
            MissionItem::SalvageCache => "Salvage Cache",
        }
    }

    pub fn contributes_to_technology(&self) -> bool {
        !matches!(self, MissionItem::SalvageCache)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    PerimeterScan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissionDefinition {
    pub mission_type: MissionType,
    pub name: &'static str,
    pub duration_minutes: u64,
    pub danger_percent: u32,
    pub description: &'static str,
}

impl MissionType {
    pub fn definition(&self) -> MissionDefinition {
        match self {
            MissionType::PerimeterScan => MissionDefinition {
                mission_type: MissionType::PerimeterScan,
                name: "Perimeter Scan",
                duration_minutes: 1,
                danger_percent: 25,
                description: "A quick one-minute sweep for useful wreckage and unknown materials.",
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveMission {
    pub id: u32,
    pub colonist_id: u32,
    pub mission_type: MissionType,
    pub started_tick: u64,
    pub completes_at_tick: u64,
    pub danger_percent: u32,
    pub priority: ColonyPriority,
}

impl ActiveMission {
    pub fn remaining_ticks(&self, current_tick: u64) -> u64 {
        self.completes_at_tick.saturating_sub(current_tick)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionState {
    pub active_missions: Vec<ActiveMission>,
    pub next_id: u32,
}

impl Default for MissionState {
    fn default() -> Self {
        Self {
            active_missions: Vec::new(),
            next_id: 1,
        }
    }
}

impl MissionState {
    pub fn active_count(&self) -> usize {
        self.active_missions.len()
    }
}
