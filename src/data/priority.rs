use super::building::BuildingType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColonyPriority {
    Recovery,
    #[default]
    Stockpile,
    Survey,
}

impl ColonyPriority {
    pub fn all() -> &'static [ColonyPriority] {
        &[
            ColonyPriority::Recovery,
            ColonyPriority::Stockpile,
            ColonyPriority::Survey,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ColonyPriority::Recovery => "Recovery",
            ColonyPriority::Stockpile => "Stockpile",
            ColonyPriority::Survey => "Survey",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            ColonyPriority::Recovery => "Care",
            ColonyPriority::Stockpile => "Stock",
            ColonyPriority::Survey => "Survey",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            ColonyPriority::Recovery => "1",
            ColonyPriority::Stockpile => "2",
            ColonyPriority::Survey => "3",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ColonyPriority::Recovery => {
                "Care routines ease mood strain, strengthen meals and lower mission danger."
            }
            ColonyPriority::Stockpile => {
                "Material crews focus on salvage, storage and meal throughput."
            }
            ColonyPriority::Survey => {
                "Scouts push harder for exploration progress and research items."
            }
        }
    }

    pub fn adjust_work_output(&self, building_type: BuildingType, base_output: u32) -> u32 {
        match self {
            ColonyPriority::Recovery => match building_type {
                BuildingType::MessHall => base_output + 1,
                BuildingType::Workshop | BuildingType::Storage | BuildingType::ExplorationGate => {
                    base_output.saturating_sub(1).max(1)
                }
                BuildingType::Habitat => base_output,
            },
            ColonyPriority::Stockpile => match building_type {
                BuildingType::MessHall | BuildingType::Workshop | BuildingType::Storage => {
                    base_output + 1
                }
                BuildingType::Habitat | BuildingType::ExplorationGate => base_output,
            },
            ColonyPriority::Survey => match building_type {
                BuildingType::ExplorationGate => base_output + 2,
                BuildingType::Habitat
                | BuildingType::MessHall
                | BuildingType::Workshop
                | BuildingType::Storage => base_output,
            },
        }
    }

    pub fn adjust_mission_danger(&self, base_danger: u32) -> u32 {
        let adjustment = match self {
            ColonyPriority::Recovery => -10,
            ColonyPriority::Stockpile => 0,
            ColonyPriority::Survey => 5,
        };

        (base_danger as i32 + adjustment).clamp(0, 100) as u32
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PriorityState {
    pub active: ColonyPriority,
}
