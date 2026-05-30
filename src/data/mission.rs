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

    pub fn short_name(&self) -> &'static str {
        match self {
            MissionItem::StructuralAlloy => "Alloy",
            MissionItem::AlienCircuit => "Circuit",
            MissionItem::MedicinalGel => "Gel",
            MissionItem::NutrientPods => "Pods",
            MissionItem::SalvageCache => "Cache",
        }
    }

    pub fn contributes_to_technology(&self) -> bool {
        !matches!(self, MissionItem::SalvageCache)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    SupplyRun,
    PerimeterScan,
    DeepSurvey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissionDefinition {
    pub mission_type: MissionType,
    pub name: &'static str,
    pub short_name: &'static str,
    pub duration_minutes: u64,
    pub danger_percent: u32,
    pub cooldown_minutes: u64,
    pub description: &'static str,
    pub reward_profile: &'static str,
}

impl MissionType {
    pub fn all() -> &'static [MissionType] {
        &[
            MissionType::SupplyRun,
            MissionType::PerimeterScan,
            MissionType::DeepSurvey,
        ]
    }

    pub fn definition(&self) -> MissionDefinition {
        match self {
            MissionType::SupplyRun => MissionDefinition {
                mission_type: MissionType::SupplyRun,
                name: "Supply Run",
                short_name: "Supply",
                duration_minutes: 45,
                danger_percent: 14,
                cooldown_minutes: 20,
                description: "A fast salvage loop for food reserves and small wreckage caches.",
                reward_profile: "Supplies, salvage, low tech chance",
            },
            MissionType::PerimeterScan => MissionDefinition {
                mission_type: MissionType::PerimeterScan,
                name: "Perimeter Scan",
                short_name: "Scout",
                duration_minutes: 90,
                danger_percent: 22,
                cooldown_minutes: 35,
                description: "A cautious sweep that balances resources, mapping, and discoveries.",
                reward_profile: "Balanced resources and tech",
            },
            MissionType::DeepSurvey => MissionDefinition {
                mission_type: MissionType::DeepSurvey,
                name: "Deep Survey",
                short_name: "Tech",
                duration_minutes: 180,
                danger_percent: 38,
                cooldown_minutes: 60,
                description: "A long push past the wreck perimeter for stronger research finds.",
                reward_profile: "High tech chance, higher danger",
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
    #[cfg(test)]
    pub fn remaining_ticks(&self, current_tick: u64) -> u64 {
        self.completes_at_tick.saturating_sub(current_tick)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionState {
    pub active_missions: Vec<ActiveMission>,
    pub next_id: u32,
    #[serde(default)]
    pub next_launch_tick: u64,
}

impl Default for MissionState {
    fn default() -> Self {
        Self {
            active_missions: Vec::new(),
            next_id: 1,
            next_launch_tick: 0,
        }
    }
}

impl MissionState {
    pub fn active_count(&self) -> usize {
        self.active_missions.len()
    }

    pub fn cooldown_remaining(&self, current_tick: u64) -> u64 {
        self.next_launch_tick.saturating_sub(current_tick)
    }
}
