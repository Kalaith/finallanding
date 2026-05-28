use super::*;

impl GameplayState {
    pub(super) fn update_assign_space_click(&mut self) {
        let Some(building) = self.building_at_mouse().cloned() else {
            return;
        };

        let Some(colonist_id) = self.selected_colonist_id else {
            return;
        };

        self.assign_selected_colonist_to_building(colonist_id, &building);
    }

    pub(super) fn assign_selected_colonist_to_building(
        &mut self,
        colonist_id: u32,
        building: &Building,
    ) {
        let Some(colonist_index) = self
            .data
            .colonists
            .iter()
            .position(|colonist| colonist.id == colonist_id)
        else {
            return;
        };

        let name = self.data.colonists[colonist_index].name.clone();
        let job = self.data.colonists[colonist_index].job_preference;
        let Some(kind) = space_assignment_kind(job, building.building_type) else {
            self.data.push_log(
                LogCategory::Social,
                "Room assignment blocked",
                format!(
                    "{} cannot pin {} #{} while assigned {}. Retask first or choose a compatible space.",
                    name,
                    building.building_type.name(),
                    building.id,
                    job.label()
                ),
            );
            return;
        };

        let colonist = &mut self.data.colonists[colonist_index];
        let (title, detail) = match kind {
            SpaceAssignmentKind::Recovery => {
                if colonist.assigned_habitat == Some(building.id) {
                    colonist.assigned_habitat = None;
                    (
                        "Recovery room pin cleared".to_string(),
                        format!("{} can choose any available Habitat again.", name),
                    )
                } else {
                    colonist.assigned_habitat = Some(building.id);
                    (
                        "Recovery room pinned".to_string(),
                        format!(
                            "{} will prefer Habitat #{} for sleep and recovery.",
                            name, building.id
                        ),
                    )
                }
            }
            SpaceAssignmentKind::Work => {
                if colonist.assigned_workplace == Some(building.id) {
                    colonist.assigned_workplace = None;
                    (
                        "Work space pin cleared".to_string(),
                        format!(
                            "{} can choose any compatible {} space again.",
                            name,
                            job.label()
                        ),
                    )
                } else {
                    colonist.assigned_workplace = Some(building.id);
                    if matches!(
                        colonist.state,
                        ColonistState::Working | ColonistState::Moving { .. }
                    ) {
                        colonist.state = ColonistState::Idle;
                        colonist.activity_location = ActivityLocation::None;
                    }
                    (
                        "Work space pinned".to_string(),
                        format!(
                            "{} will prefer {} #{} while assigned {}.",
                            name,
                            building.building_type.name(),
                            building.id,
                            job.label()
                        ),
                    )
                }
            }
        };

        self.data.push_log(LogCategory::Social, title, detail);
    }

    pub(super) fn clear_building_assignments(&mut self, building_id: u32) -> Vec<String> {
        let mut cleared = Vec::new();
        if self.assign_building_filter == Some(building_id) {
            self.assign_building_filter = None;
        }
        for colonist in &mut self.data.colonists {
            let mut changed = false;
            if colonist.assigned_habitat == Some(building_id) {
                colonist.assigned_habitat = None;
                changed = true;
            }
            if colonist.assigned_workplace == Some(building_id) {
                colonist.assigned_workplace = None;
                changed = true;
            }
            if changed {
                cleared.push(colonist.name.clone());
            }
        }

        cleared
    }
}
