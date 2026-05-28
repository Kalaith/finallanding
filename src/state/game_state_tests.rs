use super::*;
use crate::data::colonist::{JobPreference, Trait};

#[test]
fn test_terrain_detail_is_deterministic_and_skips_missing_cells() {
    assert_eq!(
        terrain_detail(Some(CellType::Empty), 7, 11),
        terrain_detail(Some(CellType::Empty), 7, 11)
    );
    assert_eq!(terrain_detail(None, 7, 11), TerrainDetail::None);
}

#[test]
fn test_crash_site_detail_adds_deterministic_map_dressing() {
    assert_eq!(crash_site_detail(10, 10), Some(TerrainDetail::SupplyCrate));
    assert_eq!(crash_site_detail(15, 5), Some(TerrainDetail::SignalBeacon));
    assert_eq!(crash_site_detail(5, 11), Some(TerrainDetail::HullPanel));
    assert_eq!(crash_site_detail(13, 7), Some(TerrainDetail::FuelDrum));
    assert_eq!(crash_site_detail(8, 7), Some(TerrainDetail::Wreckage));
    assert_eq!(crash_site_detail(4, 4), Some(TerrainDetail::Track));
    assert_eq!(crash_site_detail(8, 6), Some(TerrainDetail::Cable));
    assert_eq!(crash_site_detail(0, 0), None);
}

#[test]
fn test_terrain_color_varies_without_leaving_palette() {
    let first = terrain_color(Some(CellType::Empty), 1, 1);
    let second = terrain_color(Some(CellType::Empty), 2, 1);

    assert_ne!(first, second);
    assert!((0.14..=0.22).contains(&first.r));
    assert!((0.08..=0.14).contains(&first.b));
}

#[test]
fn test_average_relationship_uses_bidirectional_values() {
    let mut first = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    let mut second = Colonist::new(
        2,
        "Bob".to_string(),
        Position::new(1, 0),
        Trait::FastWalker,
        JobPreference::Explorer,
    );

    first.relationships.insert(2, 26);
    second.relationships.insert(1, 30);

    assert_eq!(average_relationship_between(&first, &second), 28);
    assert_eq!(strongest_relationship_value(&first), Some(26));
}

#[test]
fn test_space_assignment_kind_matches_role_and_room() {
    assert_eq!(
        space_assignment_kind(JobPreference::Builder, BuildingType::Habitat),
        Some(SpaceAssignmentKind::Recovery)
    );
    assert_eq!(
        space_assignment_kind(JobPreference::Builder, BuildingType::Workshop),
        Some(SpaceAssignmentKind::Work)
    );
    assert_eq!(
        space_assignment_kind(JobPreference::Builder, BuildingType::MessHall),
        None
    );
    assert_eq!(
        space_assignment_kind(JobPreference::Cook, BuildingType::MessHall),
        Some(SpaceAssignmentKind::Work)
    );
}

#[test]
fn test_shared_social_location_requires_same_building_or_ground_cell() {
    let mut first = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    let mut second = Colonist::new(
        2,
        "Bob".to_string(),
        Position::new(1, 0),
        Trait::FastWalker,
        JobPreference::Explorer,
    );

    first.activity_location = ActivityLocation::Building {
        building_id: 7,
        building_type: BuildingType::Workshop,
    };
    second.activity_location = ActivityLocation::Building {
        building_id: 7,
        building_type: BuildingType::Workshop,
    };

    assert!(shared_social_location(&first, &second));

    second.activity_location = ActivityLocation::Ground(Position::new(2, 2));
    assert!(!shared_social_location(&first, &second));
}

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

#[test]
fn test_assign_visible_indices_pin_selected_colonist_first() {
    let colonists = (0..6)
        .map(|id| {
            Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        ),
        vec![5, 0, 1, 2, 3]
    );
    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            None,
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        ),
        vec![0, 1, 2, 3, 4]
    );
}

#[test]
fn test_assign_visible_indices_page_through_remaining_colonists() {
    let colonists = (0..8)
        .map(|id| {
            Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        assign_roster_page_count(&colonists, Some(5), AssignRosterFilter::All, None, None),
        2
    );
    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            1,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        ),
        vec![5, 4, 6, 7]
    );
    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            None,
            1,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        ),
        vec![5, 6, 7]
    );
}

#[test]
fn test_assign_visible_indices_filter_and_sort_pressure() {
    let mut colonists = (0..6)
        .map(|id| {
            Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            )
        })
        .collect::<Vec<_>>();
    colonists[1].relationships.insert(2, -12);
    colonists[3].relationships.insert(4, -34);
    colonists[4].relationships.insert(3, 22);

    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::Risk,
            AssignRosterSort::Bond,
            None,
            None,
        ),
        vec![5, 3, 1]
    );
}

#[test]
fn test_assign_visible_indices_filter_role() {
    let mut colonists = (0..6)
        .map(|id| {
            Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            )
        })
        .collect::<Vec<_>>();
    colonists[1].job_preference = JobPreference::Cook;
    colonists[4].job_preference = JobPreference::Cook;

    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            Some(JobPreference::Cook),
            None,
        ),
        vec![5, 1, 4]
    );
}

#[test]
fn test_assign_visible_indices_filter_building_instance() {
    let mut colonists = (0..6)
        .map(|id| {
            Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            )
        })
        .collect::<Vec<_>>();
    colonists[1].assigned_habitat = Some(7);
    colonists[3].assigned_workplace = Some(7);
    colonists[4].assigned_habitat = Some(8);

    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            Some(7),
        ),
        vec![5, 1, 3]
    );
}

#[test]
fn test_next_assign_role_filter_cycles_assignable_roles() {
    assert_eq!(next_assign_role_filter(None), Some(JobPreference::Explorer));
    assert_eq!(
        next_assign_role_filter(Some(JobPreference::Explorer)),
        Some(JobPreference::Builder)
    );
    assert_eq!(next_assign_role_filter(Some(JobPreference::Hauler)), None);
}

#[test]
fn test_social_archive_markdown_exports_latest_report_first() {
    let history = vec![
        SocialHistoryEntry::new(
            1,
            "Early friction",
            "Alice and Fiona need space.",
            "Use Apart before the next work block.",
            46.0,
            -8.0,
            0,
            1,
        ),
        SocialHistoryEntry::new(
            2,
            "Shared meal",
            "Bob and Diana stabilized dinner.",
            "Keep the supportive pair together.",
            62.0,
            12.0,
            1,
            0,
        ),
    ];

    let export = social_archive_markdown(&history);

    assert!(export.contains("# The Final Landing Social Archive"));
    assert!(export.contains("Reports: 2"));
    assert!(export.find("Day 2").unwrap() < export.find("Day 1").unwrap());
    assert!(export.contains("Recommendation: Keep the supportive pair together."));
}

#[test]
fn test_batch_home_pin_respects_visible_page_and_capacity() {
    let mut colonists = (0..5)
        .map(|id| {
            Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            )
        })
        .collect::<Vec<_>>();
    colonists[0].assigned_habitat = Some(7);

    let assigned = apply_batch_home_pin(&mut colonists, 0, 7, &[0, 1, 2, 3], 2);

    assert_eq!(assigned, vec!["Colonist 1".to_string()]);
    assert_eq!(colonists[1].assigned_habitat, Some(7));
    assert_eq!(colonists[2].assigned_habitat, None);
}

#[test]
fn test_batch_work_pin_only_copies_to_compatible_visible_roles() {
    let mut colonists = vec![
        Colonist::new(
            0,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        ),
        Colonist::new(
            1,
            "Bob".to_string(),
            Position::new(1, 0),
            Trait::FastWalker,
            JobPreference::Builder,
        ),
        Colonist::new(
            2,
            "Diana".to_string(),
            Position::new(2, 0),
            Trait::Gourmet,
            JobPreference::Cook,
        ),
    ];
    colonists[0].assigned_workplace = Some(9);
    colonists[1].state = ColonistState::Working;
    colonists[1].activity_location = ActivityLocation::Building {
        building_id: 3,
        building_type: BuildingType::Workshop,
    };

    let assigned = apply_batch_work_pin(&mut colonists, 0, 9, BuildingType::Workshop, &[0, 1, 2]);

    assert_eq!(assigned, vec!["Bob".to_string()]);
    assert_eq!(colonists[1].assigned_workplace, Some(9));
    assert_eq!(colonists[1].state, ColonistState::Idle);
    assert_eq!(colonists[1].activity_location, ActivityLocation::None);
    assert_eq!(colonists[2].assigned_workplace, None);
}

#[test]
fn test_batch_assignment_log_names_all_colony_scope() {
    let (_title, detail) = batch_assignment_log(
        "Batch work pins",
        "Alice",
        "W",
        9,
        BatchAssignmentScope::All,
        vec!["Bob".to_string(), "Charlie".to_string()],
    );

    assert!(detail.contains("all compatible survivors"));
    assert!(detail.contains("Bob, Charlie"));
}

#[test]
fn test_sprite_pose_tracks_colonist_state() {
    assert_eq!(sprite_pose_for_state(ColonistState::Idle), SpritePose::Idle);
    assert_eq!(
        sprite_pose_for_state(ColonistState::Moving {
            target: Position::new(1, 1)
        }),
        SpritePose::Moving
    );
    assert_eq!(
        sprite_pose_for_state(ColonistState::Working),
        SpritePose::Working
    );
    assert_eq!(
        sprite_pose_for_state(ColonistState::Eating),
        SpritePose::Eating
    );
    assert_eq!(
        sprite_pose_for_state(ColonistState::Sleeping),
        SpritePose::Sleeping
    );
}

#[test]
fn test_social_body_language_overrides_idle_pose() {
    let colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );

    assert_eq!(
        sprite_pose_for_colonist(&colonist, Some(SocialBodyLanguage::Tense(-24))),
        SpritePose::Tense
    );
    assert_eq!(
        sprite_pose_for_colonist(&colonist, Some(SocialBodyLanguage::Supported(28))),
        SpritePose::Supported
    );
}

#[test]
fn test_social_body_language_cycles_alternate_pose_frames() {
    let colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );

    assert_eq!(
        sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Supported(28)), 45),
        SpritePose::SupportedReach
    );
    assert_eq!(
        sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 45),
        SpritePose::TenseGuarded
    );
    assert_eq!(
        sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 90),
        SpritePose::Tense
    );
}

#[test]
fn test_shared_assignment_and_adjacency_drive_social_contact() {
    let mut first = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(4, 4),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    let mut second = Colonist::new(
        2,
        "Bob".to_string(),
        Position::new(5, 4),
        Trait::FastWalker,
        JobPreference::Builder,
    );

    assert!(adjacent_positions(first.position, second.position));
    first.assigned_workplace = Some(9);
    second.assigned_workplace = Some(9);
    assert!(shared_assignment_pin(&first, &second));
    second.assigned_workplace = Some(10);
    assert!(!shared_assignment_pin(&first, &second));
}

#[test]
fn test_building_outline_style_prioritizes_hover_over_assignment() {
    let hovered = building_outline_style(true, Some(style::BAR_GREEN)).unwrap();
    let assigned = building_outline_style(false, Some(style::BAR_GREEN)).unwrap();

    assert_eq!(hovered.1, 3.0);
    assert_eq!(assigned.1, 2.0);
    assert!(hovered.0.r > assigned.0.r);
    assert!(building_outline_style(false, None).is_none());
}

#[test]
fn test_assignment_marker_with_filter_adds_filter_without_replacing_assignment() {
    let filtered = assignment_marker_with_filter(None, true).unwrap();
    assert_eq!(filtered.0, "FILTER");
    assert_eq!(filtered.1.r, style::ACCENT_GOLD.r);
    assert_eq!(filtered.1.g, style::ACCENT_GOLD.g);
    assert_eq!(filtered.1.b, style::ACCENT_GOLD.b);

    let assigned = assignment_marker_with_filter(Some(("HOME", style::BAR_GREEN)), true)
        .expect("assignment marker should remain visible");
    assert_eq!(assigned.0, "HOME");
    assert_eq!(assigned.1.r, style::BAR_GREEN.r);
    assert!(assignment_marker_with_filter(None, false).is_none());
}

#[test]
fn test_assign_filter_outline_uses_gold_room_highlight() {
    let filtered =
        building_outline_style_for_assign_filter(true, Some(style::BAR_GREEN), true).unwrap();
    assert_eq!(filtered.0.r, style::ACCENT_GOLD.r);
    assert_eq!(filtered.0.g, style::ACCENT_GOLD.g);
    assert_eq!(filtered.0.b, style::ACCENT_GOLD.b);
    assert_eq!(filtered.1, 3.0);

    let assigned =
        building_outline_style_for_assign_filter(false, Some(style::BAR_GREEN), false).unwrap();
    assert_eq!(assigned.1, 2.0);
    assert!(building_outline_style_for_assign_filter(false, None, false).is_none());
}
