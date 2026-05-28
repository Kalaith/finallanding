use super::*;

pub(crate) fn initial_toolbar_mode() -> ToolbarMode {
    std::env::var("TFL_START_TOOLBAR_MODE")
        .ok()
        .and_then(|value| toolbar_mode_from_name(&value))
        .unwrap_or(ToolbarMode::Build)
}

pub(crate) fn initial_selected_building(toolbar_mode: ToolbarMode) -> Option<BuildingType> {
    std::env::var("TFL_START_SELECTED_BUILDING")
        .ok()
        .and_then(|value| building_type_from_name(&value))
        .filter(|building_type| {
            toolbar_mode.uses_building_choices()
                && toolbar_buildings_for_mode(toolbar_mode).contains(building_type)
        })
}

pub(crate) fn initial_capture_preview_position() -> Option<Position> {
    let x = std::env::var("TFL_PREVIEW_GRID_X")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())?;
    let y = std::env::var("TFL_PREVIEW_GRID_Y")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())?;
    Some(Position::new(x, y))
}

pub(crate) fn seed_assign_spaces_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_ASSIGN_SPACES").is_ok_and(|value| value != "0") {
        return;
    }

    let placements = [
        (BuildingType::Habitat, Position::new(3, 4)),
        (BuildingType::Habitat, Position::new(8, 4)),
        (BuildingType::Workshop, Position::new(6, 8)),
        (BuildingType::Storage, Position::new(12, 8)),
    ];

    let mut habitat_id = None;
    let mut workshop_id = None;
    for (building_type, position) in placements {
        if let PlacementResult::Success(building_id) =
            data.building_system
                .try_place_building(&mut data.grid, building_type, position)
        {
            if building_type == BuildingType::Habitat && habitat_id.is_none() {
                habitat_id = Some(building_id);
            } else if building_type == BuildingType::Workshop {
                workshop_id = Some(building_id);
            }
        }
    }

    if let Some(colonist) = data.colonists.iter_mut().find(|colonist| colonist.id == 5) {
        colonist.assigned_habitat = habitat_id;
        colonist.assigned_workplace = workshop_id;
    }
    if let Some(colonist) = data.colonists.iter_mut().find(|colonist| colonist.id == 0) {
        colonist.assigned_habitat = habitat_id;
    }
}

pub(crate) fn seed_social_history_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_SOCIAL_HISTORY").is_ok_and(|value| value != "0") {
        return;
    }

    for entry in [
        SocialHistoryEntry::new(
            0,
            "Crash night summary",
            "The first shelter line held, but Alice and Fiona carried visible tension while Charlie and Evan kept field work steady.",
            "Use Assign to keep Alice and Fiona apart until recovery space improves.",
            50.0,
            1.0,
            2,
            1,
        ),
        SocialHistoryEntry::new(
            1,
            "Mess routine settled",
            "Shared meals improved morale around Bob and Diana, but the workshop queue still created late shifts.",
            "Keep cooks near supportive partners and reduce workshop crowding.",
            58.0,
            6.0,
            2,
            0,
        ),
        SocialHistoryEntry::new(
            2,
            "Workshop strain returned",
            "Alice and Fiona overlapped at the stockpile again, cutting into the recovery gains from yesterday.",
            "Separate tense workers before assigning the next salvage push.",
            47.0,
            -4.0,
            1,
            1,
        ),
        SocialHistoryEntry::new(
            3,
            "Habitat pairs adjusted",
            "Room pins gave Charlie and Evan a reliable recovery loop while Alice took quieter repair shifts.",
            "Protect the supportive pair and avoid crowding the west habitat.",
            61.0,
            8.0,
            2,
            0,
        ),
        SocialHistoryEntry::new(
            4,
            "Late repair friction",
            "The workshop recovered output, but Diana and Fiona clashed during the evening tool handoff.",
            "Move one of them to field prep before the next high-pressure day.",
            53.0,
            -7.0,
            1,
            1,
        ),
    ] {
        data.push_social_history(entry);
    }
}

pub(crate) fn seed_activity_poses_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_ACTIVITY_POSES").is_ok_and(|value| value != "0") {
        return;
    }

    data.time.speed = TimeSpeed::Paused;
    let pose_layout = [
        (0, Position::new(3, 7), ColonistState::Idle),
        (
            1,
            Position::new(6, 7),
            ColonistState::Moving {
                target: Position::new(7, 7),
            },
        ),
        (2, Position::new(9, 7), ColonistState::Working),
        (3, Position::new(12, 7), ColonistState::Eating),
        (4, Position::new(15, 7), ColonistState::Sleeping),
    ];

    for (index, position, state) in pose_layout {
        if let Some(colonist) = data.colonists.get_mut(index) {
            colonist.position = position;
            colonist.visual_x = position.x as f32 * 32.0;
            colonist.visual_y = position.y as f32 * 32.0;
            colonist.state = state;
        }
    }
}

pub(crate) fn initial_selected_colonist_id(
    data: &GameState,
    toolbar_mode: ToolbarMode,
) -> Option<u32> {
    std::env::var("TFL_START_SELECTED_COLONIST")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|id| data.colonists.iter().any(|colonist| colonist.id == *id))
        .or_else(|| {
            (toolbar_mode == ToolbarMode::Assign)
                .then(|| data.colonists.first().map(|colonist| colonist.id))
                .flatten()
        })
}

pub(crate) fn initial_selected_social_history_day(data: &GameState) -> Option<u32> {
    std::env::var("TFL_START_SOCIAL_HISTORY_DAY")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|day| data.social_history.iter().any(|entry| entry.day == *day))
}

pub(crate) fn toolbar_mode_from_name(value: &str) -> Option<ToolbarMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "build" => Some(ToolbarMode::Build),
        "rooms" => Some(ToolbarMode::Rooms),
        "objects" => Some(ToolbarMode::Objects),
        "colony" => Some(ToolbarMode::Colony),
        "research" => Some(ToolbarMode::Research),
        "assign" => Some(ToolbarMode::Assign),
        "log" => Some(ToolbarMode::Log),
        _ => None,
    }
}

pub(crate) fn building_type_from_name(value: &str) -> Option<BuildingType> {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "habitat" => Some(BuildingType::Habitat),
        "mess_hall" | "messhall" => Some(BuildingType::MessHall),
        "workshop" => Some(BuildingType::Workshop),
        "storage" => Some(BuildingType::Storage),
        "exploration_gate" | "explorationgate" | "gate" => Some(BuildingType::ExplorationGate),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_mode_from_name_accepts_capture_modes() {
        assert_eq!(toolbar_mode_from_name("assign"), Some(ToolbarMode::Assign));
        assert_eq!(
            toolbar_mode_from_name(" Research "),
            Some(ToolbarMode::Research)
        );
        assert_eq!(toolbar_mode_from_name("missing"), None);
    }

    #[test]
    fn test_building_type_from_name_accepts_capture_names() {
        assert_eq!(
            building_type_from_name("mess hall"),
            Some(BuildingType::MessHall)
        );
        assert_eq!(
            building_type_from_name("exploration-gate"),
            Some(BuildingType::ExplorationGate)
        );
        assert_eq!(building_type_from_name("missing"), None);
    }
}
