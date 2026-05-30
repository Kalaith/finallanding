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
    TriageProtocols,
    DroneSurvey,
    NutrientCulture,
    HullRetrofits,
    FabricationJigs,
}

impl TechId {
    pub fn all() -> &'static [TechId] {
        &[
            TechId::FieldMedicine,
            TechId::SurveyScanners,
            TechId::ModularHabitats,
            TechId::HydroponicPlanning,
            TechId::StorageLattice,
            TechId::TriageProtocols,
            TechId::DroneSurvey,
            TechId::NutrientCulture,
            TechId::HullRetrofits,
            TechId::FabricationJigs,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            TechId::FieldMedicine => "Field Medicine",
            TechId::SurveyScanners => "Survey Scanners",
            TechId::ModularHabitats => "Modular Habitats",
            TechId::HydroponicPlanning => "Hydroponic Planning",
            TechId::StorageLattice => "Storage Lattice",
            TechId::TriageProtocols => "Triage Protocols",
            TechId::DroneSurvey => "Drone Survey",
            TechId::NutrientCulture => "Nutrient Culture",
            TechId::HullRetrofits => "Hull Retrofits",
            TechId::FabricationJigs => "Fabrication Jigs",
        }
    }

    pub fn effect_text(&self) -> &'static str {
        match self {
            TechId::FieldMedicine => "Mission injuries recover faster.",
            TechId::SurveyScanners => "Mission danger is reduced.",
            TechId::ModularHabitats => "Each Habitat can support one more sleeper.",
            TechId::HydroponicPlanning => "Daily supply need is reduced by one.",
            TechId::StorageLattice => "Storage capacity increases.",
            TechId::TriageProtocols => "Mission injuries recover even faster.",
            TechId::DroneSurvey => "Mission danger and regroup time are reduced.",
            TechId::NutrientCulture => "Daily supply need is reduced further.",
            TechId::HullRetrofits => "Habitats and storage use wreckage more efficiently.",
            TechId::FabricationJigs => "Workshop and hauling salvage recovery improves.",
        }
    }

    pub fn item_requirements(&self) -> Vec<(MissionItem, u32)> {
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
            TechId::TriageProtocols => {
                vec![
                    (MissionItem::MedicinalGel, 2),
                    (MissionItem::AlienCircuit, 1),
                ]
            }
            TechId::DroneSurvey => {
                vec![
                    (MissionItem::AlienCircuit, 2),
                    (MissionItem::StructuralAlloy, 1),
                ]
            }
            TechId::NutrientCulture => {
                vec![
                    (MissionItem::NutrientPods, 2),
                    (MissionItem::MedicinalGel, 1),
                ]
            }
            TechId::HullRetrofits => {
                vec![
                    (MissionItem::StructuralAlloy, 2),
                    (MissionItem::AlienCircuit, 1),
                ]
            }
            TechId::FabricationJigs => {
                vec![
                    (MissionItem::StructuralAlloy, 3),
                    (MissionItem::AlienCircuit, 2),
                ]
            }
        }
    }

    pub fn prerequisite_tech(&self) -> Vec<TechId> {
        match self {
            TechId::TriageProtocols => vec![TechId::FieldMedicine],
            TechId::DroneSurvey => vec![TechId::SurveyScanners],
            TechId::NutrientCulture => vec![TechId::HydroponicPlanning],
            TechId::HullRetrofits => vec![TechId::ModularHabitats, TechId::StorageLattice],
            TechId::FabricationJigs => vec![TechId::StorageLattice],
            _ => Vec::new(),
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

    pub fn next_research_target(&self) -> Option<TechId> {
        TechId::all()
            .iter()
            .copied()
            .find(|tech_id| !self.has(*tech_id) && self.prerequisites_met(*tech_id))
            .or_else(|| self.next_locked_tech())
    }

    pub fn visible_research_targets(&self, limit: usize) -> Vec<TechId> {
        TechId::all()
            .iter()
            .copied()
            .filter(|tech_id| !self.has(*tech_id))
            .filter(|tech_id| self.prerequisites_met(*tech_id))
            .take(limit)
            .collect()
    }

    pub fn requirement_progress_text(&self, tech_id: TechId) -> String {
        let mut missing = Vec::new();

        for prerequisite in tech_id.prerequisite_tech() {
            if !self.has(prerequisite) {
                missing.push(format!("need {}", prerequisite.name()));
            }
        }

        for (item, required) in tech_id.item_requirements() {
            let count = self.item_count(item);
            if count < required {
                missing.push(format!("{} {}/{}", item.short_name(), count, required));
            }
        }

        if missing.is_empty() {
            "ready to unlock".to_string()
        } else {
            missing.join(" | ")
        }
    }

    pub fn mission_danger_reduction(&self) -> u32 {
        if self.has(TechId::DroneSurvey) {
            18
        } else if self.has(TechId::SurveyScanners) {
            10
        } else {
            0
        }
    }

    pub fn mission_cooldown_reduction(&self) -> u64 {
        if self.has(TechId::DroneSurvey) {
            10
        } else {
            0
        }
    }

    pub fn injury_duration_ticks(&self) -> u64 {
        if self.has(TechId::TriageProtocols) {
            40
        } else if self.has(TechId::FieldMedicine) {
            60
        } else {
            150
        }
    }

    pub fn habitat_capacity_bonus(&self) -> u32 {
        if self.has(TechId::HullRetrofits) {
            2
        } else if self.has(TechId::ModularHabitats) {
            1
        } else {
            0
        }
    }

    pub fn daily_supply_reduction(&self) -> i32 {
        if self.has(TechId::NutrientCulture) {
            2
        } else if self.has(TechId::HydroponicPlanning) {
            1
        } else {
            0
        }
    }

    pub fn storage_capacity_bonus(&self) -> i32 {
        if self.has(TechId::HullRetrofits) {
            35
        } else if self.has(TechId::StorageLattice) {
            20
        } else {
            0
        }
    }

    pub fn salvage_recovery_bonus(&self) -> i32 {
        if self.has(TechId::FabricationJigs) {
            1
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
        self.prerequisites_met(tech_id)
            && tech_id
                .item_requirements()
                .iter()
                .all(|(item, required)| self.item_count(*item) >= *required)
    }

    pub fn prerequisites_met(&self, tech_id: TechId) -> bool {
        tech_id
            .prerequisite_tech()
            .iter()
            .all(|prerequisite| self.has(*prerequisite))
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

    #[test]
    fn test_advanced_technology_requires_prerequisite_and_extra_items() {
        let mut technology = TechnologyState::default();
        technology.add_item(MissionItem::MedicinalGel);
        assert!(technology.has(TechId::FieldMedicine));
        assert!(!technology.has(TechId::TriageProtocols));

        let unlocked = technology.add_item(MissionItem::AlienCircuit);
        assert!(!unlocked.contains(&TechId::TriageProtocols));

        let unlocked = technology.add_item(MissionItem::MedicinalGel);
        assert!(unlocked.contains(&TechId::TriageProtocols));
        assert_eq!(technology.injury_duration_ticks(), 40);
    }

    #[test]
    fn test_research_target_and_progress_text_show_available_tree_work() {
        let mut technology = TechnologyState::default();

        assert_eq!(
            technology.next_research_target(),
            Some(TechId::FieldMedicine)
        );
        assert_eq!(
            technology.requirement_progress_text(TechId::FieldMedicine),
            "Gel 0/1"
        );

        technology.add_item(MissionItem::MedicinalGel);
        assert_eq!(
            technology.next_research_target(),
            Some(TechId::SurveyScanners)
        );
        assert!(technology
            .visible_research_targets(5)
            .contains(&TechId::TriageProtocols));
    }

    #[test]
    fn test_advanced_effects_stack_into_existing_system_modifiers() {
        let mut technology = TechnologyState::default();
        technology.add_item(MissionItem::AlienCircuit);
        technology.add_item(MissionItem::AlienCircuit);
        technology.add_item(MissionItem::StructuralAlloy);

        assert!(technology.has(TechId::DroneSurvey));
        assert_eq!(technology.mission_danger_reduction(), 18);
        assert_eq!(technology.mission_cooldown_reduction(), 10);

        technology.add_item(MissionItem::StructuralAlloy);
        assert!(technology.has(TechId::HullRetrofits));
        assert_eq!(technology.habitat_capacity_bonus(), 2);
        assert_eq!(technology.storage_capacity_bonus(), 35);
    }
}
