use crate::data::mission::MissionItem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechId {
    FieldMedicine,
    SurveyScanners,
    ModularHabitats,
    HydroponicPlanning,
    StorageLattice,
}

impl TechId {
    pub fn all() -> &'static [TechId] {
        &[
            TechId::FieldMedicine,
            TechId::SurveyScanners,
            TechId::ModularHabitats,
            TechId::HydroponicPlanning,
            TechId::StorageLattice,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            TechId::FieldMedicine => "Field Medicine",
            TechId::SurveyScanners => "Survey Scanners",
            TechId::ModularHabitats => "Modular Habitats",
            TechId::HydroponicPlanning => "Hydroponic Planning",
            TechId::StorageLattice => "Storage Lattice",
        }
    }

    pub fn effect_text(&self) -> &'static str {
        match self {
            TechId::FieldMedicine => "Mission injuries recover faster.",
            TechId::SurveyScanners => "Mission danger is reduced.",
            TechId::ModularHabitats => "Each Habitat can support one more sleeper.",
            TechId::HydroponicPlanning => "Daily supply need is reduced by one.",
            TechId::StorageLattice => "Storage capacity increases.",
        }
    }

    pub fn requirements(&self) -> Vec<(MissionItem, u32)> {
        match self {
            TechId::FieldMedicine => vec![(MissionItem::MedicinalGel, 1)],
            TechId::SurveyScanners => vec![(MissionItem::AlienCircuit, 1)],
            TechId::ModularHabitats => vec![(MissionItem::StructuralAlloy, 1)],
            TechId::HydroponicPlanning => vec![(MissionItem::NutrientPods, 1)],
            TechId::StorageLattice => {
                vec![
                    (MissionItem::StructuralAlloy, 1),
                    (MissionItem::AlienCircuit, 1),
                ]
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TechnologyState {
    pub item_counts: HashMap<MissionItem, u32>,
    pub unlocked: Vec<TechId>,
}

impl Default for TechnologyState {
    fn default() -> Self {
        Self {
            item_counts: HashMap::new(),
            unlocked: Vec::new(),
        }
    }
}

impl TechnologyState {
    pub fn add_item(&mut self, item: MissionItem) -> Vec<TechId> {
        if item.contributes_to_technology() {
            *self.item_counts.entry(item).or_insert(0) += 1;
        }

        self.unlock_available()
    }

    pub fn has(&self, tech_id: TechId) -> bool {
        self.unlocked.contains(&tech_id)
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }

    pub fn next_locked_tech(&self) -> Option<TechId> {
        TechId::all()
            .iter()
            .copied()
            .find(|tech_id| !self.has(*tech_id))
    }

    pub fn mission_danger_reduction(&self) -> u32 {
        if self.has(TechId::SurveyScanners) {
            10
        } else {
            0
        }
    }

    pub fn injury_duration_ticks(&self) -> u64 {
        if self.has(TechId::FieldMedicine) {
            60
        } else {
            150
        }
    }

    pub fn habitat_capacity_bonus(&self) -> u32 {
        if self.has(TechId::ModularHabitats) {
            1
        } else {
            0
        }
    }

    pub fn daily_supply_reduction(&self) -> i32 {
        if self.has(TechId::HydroponicPlanning) {
            1
        } else {
            0
        }
    }

    pub fn storage_capacity_bonus(&self) -> i32 {
        if self.has(TechId::StorageLattice) {
            20
        } else {
            0
        }
    }

    fn unlock_available(&mut self) -> Vec<TechId> {
        let mut newly_unlocked = Vec::new();

        for tech_id in TechId::all() {
            if self.has(*tech_id) {
                continue;
            }

            if self.meets_requirements(*tech_id) {
                self.unlocked.push(*tech_id);
                newly_unlocked.push(*tech_id);
            }
        }

        newly_unlocked
    }

    fn meets_requirements(&self, tech_id: TechId) -> bool {
        tech_id
            .requirements()
            .iter()
            .all(|(item, required)| self.item_count(*item) >= *required)
    }

    pub fn item_count(&self, item: MissionItem) -> u32 {
        self.item_counts.get(&item).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_items_unlock_technology() {
        let mut technology = TechnologyState::default();

        let unlocked = technology.add_item(MissionItem::MedicinalGel);

        assert_eq!(unlocked, vec![TechId::FieldMedicine]);
        assert!(technology.has(TechId::FieldMedicine));
    }

    #[test]
    fn test_combined_requirements_unlock_storage_lattice() {
        let mut technology = TechnologyState::default();
        technology.add_item(MissionItem::StructuralAlloy);

        let unlocked = technology.add_item(MissionItem::AlienCircuit);

        assert!(unlocked.contains(&TechId::StorageLattice));
        assert!(technology.has(TechId::StorageLattice));
    }
}
