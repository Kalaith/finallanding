use super::Layout;
use crate::data::building::BuildingType;
use crate::data::colonist::{relationship_label, Colonist, JobPreference};
use crate::data::event_log::{ColonyLogEntry, LogCategory, SocialHistoryEntry};
use crate::data::priority::ColonyPriority;
use crate::data::resources::ResourceState;
use crate::data::technology::{TechId, TechnologyState};
use crate::systems::assignment_system::{AssignmentPressure, AssignmentSystem};
use crate::systems::mission_system::MissionPlan;
use crate::systems::relationship_directive_system::{PairDirective, RelationshipDirectiveSystem};
use crate::systems::summary_system::{ColonyPressureSummary, RelationshipPairSummary};
use crate::ui::font::draw_text;
use crate::ui::hit_zones::{
    assign_batch_rect, assign_filter_rect, assign_page_next_rect, assign_page_previous_rect,
    assign_role_filter_rect, assign_sort_rect, log_filter_rect, log_page_next_rect,
    log_page_previous_rect, log_search_clear_rect, log_search_export_rect, log_search_rect,
    log_timeline_row_rect, toolbar_buildings_for_mode, toolbar_context_item_rect,
    toolbar_context_rect, toolbar_list_item_rect, AssignBatchAction, AssignRosterFilter,
    AssignRosterSort, LogFilter, ToolbarMode,
};
use crate::ui::style;
use crate::ui::tooltip::draw_tooltip_near_mouse;
use macroquad::prelude::*;

pub const SOCIAL_TIMELINE_PAGE_SIZE: usize = 3;
const ASSIGN_ROSTER_SLOT_COUNT: usize = 5;

mod assign;
mod log;

use assign::*;
use log::*;

pub use log::{social_history_page_count, social_timeline_day_at};

pub struct ToolbarPanelData<'a> {
    pub mode: ToolbarMode,
    pub selected_building: Option<BuildingType>,
    pub resources: &'a ResourceState,
    pub active_priority: ColonyPriority,
    pub research: ToolbarResearchData<'a>,
    pub assign: ToolbarAssignData<'a>,
    pub log: ToolbarLogData<'a>,
}

pub struct ToolbarResearchData<'a> {
    pub mission_plans: &'a [MissionPlan],
    pub technology: &'a TechnologyState,
    pub active_mission_count: usize,
}

pub struct ToolbarAssignData<'a> {
    pub colonists: &'a [Colonist],
    pub selected_colonist_id: Option<u32>,
    pub roster_page: usize,
    pub roster_filter: AssignRosterFilter,
    pub roster_sort: AssignRosterSort,
    pub role_filter: Option<JobPreference>,
    pub building_filter: Option<u32>,
    pub technology: &'a TechnologyState,
}

pub struct ToolbarLogData<'a> {
    pub logs: &'a [ColonyLogEntry],
    pub social_history: &'a [SocialHistoryEntry],
    pub page: usize,
    pub filter: LogFilter,
    pub query: &'a str,
    pub search_active: bool,
    pub selected_day: Option<u32>,
    pub colony_summary: &'a ColonyPressureSummary,
}

pub fn draw_toolbar_context_panel(layout: &Layout, panel: ToolbarPanelData<'_>) {
    let context = toolbar_context_rect(layout.bottom_toolbar());
    style::draw_panel(context);
    style::draw_section_title(
        panel.mode.label().to_uppercase().as_str(),
        context.x + 14.0,
        context.y + 27.0,
    );

    match panel.mode {
        ToolbarMode::Build | ToolbarMode::Rooms | ToolbarMode::Objects => draw_build_context(
            context,
            panel.mode,
            panel.selected_building,
            panel.resources,
        ),
        ToolbarMode::Colony => draw_colony_context(context, panel.active_priority),
        ToolbarMode::Research => draw_research_context(
            context,
            panel.research.mission_plans,
            panel.research.technology,
            panel.research.active_mission_count,
        ),
        ToolbarMode::Assign => draw_assign_context(
            context,
            panel.assign.colonists,
            panel.assign.selected_colonist_id,
            panel.assign.roster_page,
            panel.assign.roster_filter,
            panel.assign.roster_sort,
            panel.assign.role_filter,
            panel.assign.building_filter,
            panel.assign.technology,
        ),
        ToolbarMode::Log => draw_log_context(
            context,
            panel.log.logs,
            panel.log.social_history,
            panel.log.page,
            panel.log.filter,
            panel.log.query,
            panel.log.search_active,
            panel.log.selected_day,
            panel.log.colony_summary,
        ),
    }
}

fn draw_build_context(
    context: Rect,
    mode: ToolbarMode,
    selected_building: Option<BuildingType>,
    resources: &ResourceState,
) {
    let mut hovered_building = None;
    for (index, building_type) in toolbar_buildings_for_mode(mode).iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let can_afford = resources.salvage >= building_type.salvage_cost();
        let selected = selected_building == Some(*building_type);
        let hovered = rect.contains(mouse_position().into());
        if hovered {
            hovered_building = Some(*building_type);
        }
        style::draw_button(rect, selected, hovered);
        let (r, g, b) = building_type.color();
        draw_rectangle(
            rect.x + 9.0,
            rect.y + 9.0,
            14.0,
            14.0,
            Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0),
        );
        draw_text(
            &style::truncate_text(building_type.name(), 12),
            rect.x + 30.0,
            rect.y + 18.0,
            style::TINY_SIZE,
            if can_afford {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_MUTED
            },
        );
        draw_text(
            &format!("{} salvage", building_type.salvage_cost()),
            rect.x + 30.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            if can_afford {
                style::TEXT_BODY
            } else {
                style::ALERT_RED
            },
        );
    }

    let helper = match mode {
        ToolbarMode::Rooms => "Room plans shape sleep, meals, and storage pressure.",
        ToolbarMode::Objects => "Work objects produce salvage and survey returns.",
        _ => "Plans reserve salvage and define colony space.",
    };
    draw_text(
        helper,
        context.x + 18.0,
        context.y + 111.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    if let Some(building_type) = hovered_building {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            building_type.name(),
            &format!(
                "{} Cost: {} salvage.",
                building_type.placement_impact(),
                building_type.salvage_cost()
            ),
        );
    }
}

fn draw_colony_context(context: Rect, active_priority: ColonyPriority) {
    let mut hovered_priority = None;
    for (index, priority) in ColonyPriority::all().iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let hovered = rect.contains(mouse_position().into());
        if hovered {
            hovered_priority = Some(*priority);
        }
        style::draw_button(rect, *priority == active_priority, hovered);
        draw_text(
            priority.short_label(),
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_text(
            &style::truncate_text(priority.description(), 26),
            rect.x + 10.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }

    if let Some(priority) = hovered_priority {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            priority.label(),
            priority.description(),
        );
    }
}

fn draw_research_context(
    context: Rect,
    mission_plans: &[MissionPlan],
    technology: &TechnologyState,
    active_mission_count: usize,
) {
    let mut hovered_plan = None;
    for (index, plan) in mission_plans.iter().enumerate() {
        let rect = toolbar_context_item_rect(context, index);
        let hovered = rect.contains(mouse_position().into());
        if hovered {
            hovered_plan = Some(plan);
        }
        style::draw_button(rect, plan.recommended, hovered);
        draw_text(
            plan.definition.short_name,
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );
        draw_text(
            &format!(
                "{}m | {}%",
                plan.definition.duration_minutes, plan.danger_percent
            ),
            rect.x + 10.0,
            rect.y + 34.0,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }

    let tech_label = technology
        .next_locked_tech()
        .map(|tech| tech.name())
        .unwrap_or("All field tech unlocked");
    draw_text(
        &format!(
            "Away {} | Tech {}/{} | Next: {}",
            active_mission_count,
            technology.unlocked_count(),
            TechId::all().len(),
            tech_label
        ),
        context.x + 18.0,
        context.y + 109.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );

    if let Some(plan) = hovered_plan {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            plan.definition.name,
            &format!(
                "{} {}",
                plan.definition.description, plan.definition.reward_profile
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{JobPreference, Trait};
    use crate::data::types::Position;

    fn test_colonist(id: u32) -> Colonist {
        Colonist::new(
            id,
            format!("Colonist {}", id),
            Position::new(id as i32, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        )
    }

    #[test]
    fn test_assign_visible_colonists_pin_selected_first() {
        let colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        let visible = assign_visible_colonists(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 0, 1, 2, 3]);
    }

    #[test]
    fn test_assign_visible_colonists_page_through_roster() {
        let colonists = (0..8).map(test_colonist).collect::<Vec<_>>();
        let page = assign_visible_colonists(
            &colonists,
            Some(5),
            1,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(
            assign_roster_page_count(&colonists, Some(5), AssignRosterFilter::All, None, None),
            2
        );
        assert_eq!(page, vec![5, 4, 6, 7]);
    }

    #[test]
    fn test_assign_visible_colonists_filter_pinned_and_sort_mood() {
        let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        colonists[1].assigned_habitat = Some(3);
        colonists[1].mood = 42.0;
        colonists[4].assigned_workplace = Some(8);
        colonists[4].mood = 21.0;

        let visible = assign_visible_colonists(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::Pinned,
            AssignRosterSort::Mood,
            None,
            None,
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 4, 1]);
    }

    #[test]
    fn test_assign_visible_colonists_filter_role() {
        let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        colonists[1].job_preference = JobPreference::Cook;
        colonists[3].job_preference = JobPreference::Cook;
        colonists[4].job_preference = JobPreference::Explorer;

        let visible = assign_visible_colonists(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            Some(JobPreference::Cook),
            None,
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 1, 3]);
    }

    #[test]
    fn test_assign_visible_colonists_filter_building_instance() {
        let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
        colonists[1].assigned_habitat = Some(7);
        colonists[3].assigned_workplace = Some(7);
        colonists[4].assigned_habitat = Some(8);

        let visible = assign_visible_colonists(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            Some(7),
        )
        .into_iter()
        .map(|colonist| colonist.id)
        .collect::<Vec<_>>();

        assert_eq!(visible, vec![5, 1, 3]);
    }

    #[test]
    fn test_assign_pair_action_reports_active_directive() {
        let mut colonists = vec![test_colonist(1), test_colonist(2)];
        colonists[0].relationships.insert(2, -24);
        colonists[1].relationships.insert(1, -20);
        colonists[0].avoided_partner_id = Some(2);
        colonists[1].avoided_partner_id = Some(1);

        let action = assign_pair_action(&colonists, 1, 2).unwrap();

        assert_eq!(action.directive, PairDirective::Separate);
        assert_eq!(action.label, "Apart set -22");
    }

    #[test]
    fn test_selected_assignment_label_reports_room_pins() {
        let mut colonist = test_colonist(1);
        assert_eq!(selected_assignment_label(&colonist), "H-- W--");

        colonist.assigned_habitat = Some(3);
        colonist.assigned_workplace = Some(8);

        assert_eq!(selected_assignment_label(&colonist), "H#3 W#8");
        assert!(selected_assignment_detail(
            &colonist,
            &[colonist.clone()],
            &TechnologyState::default()
        )
        .contains("H#3 W#8"));
    }

    #[test]
    fn test_assignment_pin_warning_flags_over_capacity_habitat() {
        let mut colonists = vec![test_colonist(1), test_colonist(2), test_colonist(3)];
        for colonist in &mut colonists {
            colonist.assigned_habitat = Some(7);
        }

        let warning =
            assignment_pin_warning(&colonists[0], &colonists, &TechnologyState::default()).unwrap();

        assert_eq!(warning.label, "CAP");
        assert!(warning.detail.contains("3/2"));
    }

    #[test]
    fn test_assignment_pin_warning_flags_tense_shared_workplace() {
        let mut colonists = vec![test_colonist(1), test_colonist(2)];
        colonists[0].assigned_workplace = Some(9);
        colonists[1].assigned_workplace = Some(9);
        colonists[0].relationships.insert(2, -24);
        colonists[1].relationships.insert(1, -20);

        let warning =
            assignment_pin_warning(&colonists[0], &colonists, &TechnologyState::default()).unwrap();

        assert_eq!(warning.label, "TENSE");
        assert!(warning.detail.contains("Colonist 2"));
        assert!(warning.detail.contains("W#9"));
    }

    #[test]
    fn test_social_brief_prioritizes_tense_pair() {
        let summary = ColonyPressureSummary {
            average_mood: 47.0,
            average_relationship: -2.0,
            close_pairs: 1,
            strained_pairs: 1,
            connected_pairs: vec![],
            tense_pairs: vec![],
            strongest_pair: None,
            weakest_pair: Some(RelationshipPairSummary {
                first_name: "Alice".to_string(),
                second_name: "Fiona".to_string(),
                value: -24,
                label: "Tense",
            }),
        };

        let brief = social_brief_lines(&summary);

        assert!(brief.header.contains("tense 1"));
        assert_eq!(brief.detail, "Watch Alice / Fiona: Tense -24");
        assert_eq!(brief.color, style::ALERT_RED);
    }

    #[test]
    fn test_social_brief_names_strongest_pair_when_stable() {
        let summary = ColonyPressureSummary {
            average_mood: 62.0,
            average_relationship: 4.0,
            close_pairs: 1,
            strained_pairs: 0,
            connected_pairs: vec![],
            tense_pairs: vec![],
            strongest_pair: Some(RelationshipPairSummary {
                first_name: "Charlie".to_string(),
                second_name: "Evan".to_string(),
                value: 28,
                label: "Friendly",
            }),
            weakest_pair: None,
        };

        let brief = social_brief_lines(&summary);

        assert_eq!(brief.detail, "Protect Charlie / Evan: Friendly +28");
        assert_eq!(brief.color, style::BAR_GREEN);
    }

    #[test]
    fn test_latest_social_history_is_available_to_log_context() {
        let history = SocialHistoryEntry::new(
            2,
            "Day 2 summary",
            "Relationships stabilized.",
            "Keep Charlie and Evan together.",
            64.0,
            5.0,
            1,
            0,
        );

        assert_eq!(history.day, 2);
        assert_eq!(history.recommendation, "Keep Charlie and Evan together.");
    }

    #[test]
    fn test_social_timeline_rows_show_latest_three_days_first() {
        let history = (0..5)
            .map(|day| {
                SocialHistoryEntry::new(
                    day,
                    format!("Day {} summary", day),
                    "Social detail.",
                    "Recommendation.",
                    50.0 + day as f32,
                    day as f32,
                    day,
                    0,
                )
            })
            .collect::<Vec<_>>();

        let rows = social_timeline_rows(&history, LogFilter::All, "", 0);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].day, 4);
        assert_eq!(rows[1].day, 3);
        assert_eq!(rows[2].day, 2);
        assert_eq!(rows[0].metrics, "M54 R+4 T0");
    }

    #[test]
    fn test_social_timeline_rows_page_through_archive() {
        let history = (0..7)
            .map(|day| SocialHistoryEntry::new(day, "", "", "", 50.0, day as f32, 0, 0))
            .collect::<Vec<_>>();

        let first_page = social_timeline_rows(&history, LogFilter::All, "", 0);
        let second_page = social_timeline_rows(&history, LogFilter::All, "", 1);
        let last_page = social_timeline_rows(&history, LogFilter::All, "", 2);
        let clamped_page = social_timeline_rows(&history, LogFilter::All, "", 99);

        assert_eq!(social_history_page_count(&history, LogFilter::All, ""), 3);
        assert_eq!(
            first_page.iter().map(|row| row.day).collect::<Vec<_>>(),
            vec![6, 5, 4]
        );
        assert_eq!(
            second_page.iter().map(|row| row.day).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(
            last_page.iter().map(|row| row.day).collect::<Vec<_>>(),
            vec![0]
        );
        assert_eq!(clamped_page[0].day, 0);
    }

    #[test]
    fn test_social_timeline_rows_filter_tense_and_support_reports() {
        let history = vec![
            SocialHistoryEntry::new(0, "Neutral", "", "", 52.0, 0.0, 0, 0),
            SocialHistoryEntry::new(1, "Tense", "", "", 43.0, -9.0, 0, 1),
            SocialHistoryEntry::new(2, "Support", "", "", 66.0, 12.0, 1, 0),
        ];

        let tense = social_timeline_rows(&history, LogFilter::Tense, "", 0);
        let support = social_timeline_rows(&history, LogFilter::Support, "", 0);

        assert_eq!(tense.len(), 1);
        assert_eq!(tense[0].day, 1);
        assert_eq!(support.len(), 1);
        assert_eq!(support[0].day, 2);
        assert_eq!(social_history_page_count(&history, LogFilter::Tense, ""), 1);
    }

    #[test]
    fn test_social_timeline_rows_search_reports() {
        let history = vec![
            SocialHistoryEntry::new(
                1,
                "Tension spike",
                "Alice isolated.",
                "Use Apart.",
                42.0,
                -8.0,
                0,
                1,
            ),
            SocialHistoryEntry::new(
                2,
                "Shared meal",
                "Bob encouraged Diana.",
                "Keep together.",
                66.0,
                14.0,
                1,
                0,
            ),
            SocialHistoryEntry::new(
                3,
                "Quiet shift",
                "Workshop stable.",
                "Watch mood.",
                52.0,
                2.0,
                0,
                0,
            ),
        ];

        let rows = social_timeline_rows(&history, LogFilter::All, "diana", 0);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].day, 2);
        assert_eq!(
            social_history_page_count(&history, LogFilter::All, "diana"),
            1
        );
    }

    #[test]
    fn test_social_timeline_day_at_matches_filtered_visible_rows() {
        let history = (0..5)
            .map(|day| {
                SocialHistoryEntry::new(
                    day,
                    format!("Day {}", day),
                    "",
                    "",
                    50.0,
                    if day % 2 == 0 { -8.0 } else { 10.0 },
                    u32::from(day % 2 == 1),
                    u32::from(day % 2 == 0),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            social_timeline_day_at(&history, LogFilter::Tense, "", 0, 0),
            Some(4)
        );
        assert_eq!(
            social_timeline_day_at(&history, LogFilter::Support, "", 0, 1),
            Some(1)
        );
        assert_eq!(
            social_timeline_day_at(&history, LogFilter::Support, "", 0, 2),
            None
        );
    }

    #[test]
    fn test_social_timeline_colors_pressure_and_support() {
        let tense = SocialHistoryEntry::new(2, "", "", "", 42.0, -2.0, 0, 1);
        let close = SocialHistoryEntry::new(3, "", "", "", 68.0, 9.0, 1, 0);
        let neutral = SocialHistoryEntry::new(4, "", "", "", 55.0, 0.0, 0, 0);

        assert_eq!(social_history_color(&tense), style::ALERT_RED);
        assert_eq!(social_history_color(&close), style::BAR_GREEN);
        assert_eq!(social_history_color(&neutral), style::HEADING_BLUE);
    }
}
