use super::*;

pub(super) fn draw_assign_context(
    context: Rect,
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    assign_roster_page: usize,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
    technology: &TechnologyState,
) {
    let mut hovered_forecast = None;
    let mut hovered_name = None;
    let mut hovered_directive = None;
    let mut hovered_filter = None;
    let mut hovered_sort = None;
    let mut hovered_role_filter = false;
    draw_assign_roster_controls(
        context,
        active_filter,
        active_sort,
        active_role_filter,
        &mut hovered_filter,
        &mut hovered_sort,
        &mut hovered_role_filter,
    );
    let page_count = assign_roster_page_count(
        colonists,
        selected_colonist_id,
        active_filter,
        active_role_filter,
        active_building_filter,
    );
    let current_page = assign_roster_page.min(page_count.saturating_sub(1));
    if page_count > 1 {
        draw_assign_page_controls(context, current_page, page_count);
    }

    for (slot, colonist) in assign_visible_colonists(
        colonists,
        selected_colonist_id,
        current_page,
        active_filter,
        active_sort,
        active_role_filter,
        active_building_filter,
    )
    .into_iter()
    .enumerate()
    {
        let rect = toolbar_list_item_rect(context, slot);
        let selected = selected_colonist_id == Some(colonist.id);
        let hovered = rect.contains(mouse_position().into());
        let pair_action = selected_colonist_id
            .filter(|selected_id| *selected_id != colonist.id)
            .and_then(|selected_id| assign_pair_action(colonists, selected_id, colonist.id));
        let pin_warning = assignment_pin_warning(colonist, colonists, technology);

        style::draw_button(rect, selected, hovered);
        draw_rectangle(
            rect.x,
            rect.y,
            3.0,
            rect.h,
            pin_warning
                .as_ref()
                .filter(|_| selected)
                .map(|_| style::ALERT_RED)
                .or_else(|| {
                    pair_action
                        .as_ref()
                        .map(|action| directive_color(action.directive))
                })
                .unwrap_or_else(|| {
                    let next_role = colonist.job_preference.next_assignable();
                    let forecast =
                        AssignmentSystem::forecast_role_change(colonists, colonist.id, next_role);
                    assignment_pressure_color(forecast.pressure)
                }),
        );
        draw_text(
            &style::truncate_text(&colonist.name, 11),
            rect.x + 10.0,
            rect.y + 18.0,
            style::SMALL_SIZE,
            style::TEXT_PRIMARY,
        );

        if selected {
            if hovered {
                hovered_directive =
                    Some(selected_assignment_detail(colonist, colonists, technology));
                hovered_name = Some(colonist.name.clone());
            }
            let label = pin_warning
                .as_ref()
                .map(|warning| format!("{} {}", warning.label, selected_assignment_label(colonist)))
                .unwrap_or_else(|| selected_assignment_label(colonist));
            draw_text(
                &style::truncate_text(&label, 17),
                rect.x + 10.0,
                rect.y + 34.0,
                style::TINY_SIZE,
                if pin_warning.is_some() {
                    style::ALERT_RED
                } else {
                    style::TEXT_BODY
                },
            );
        } else if let Some(action) = pair_action {
            if hovered {
                hovered_directive = Some(action.detail);
                hovered_name = Some(colonist.name.clone());
            }
            draw_text(
                &style::truncate_text(&action.label, 16),
                rect.x + 10.0,
                rect.y + 34.0,
                style::TINY_SIZE,
                directive_color(action.directive),
            );
        } else {
            let next_role = colonist.job_preference.next_assignable();
            let forecast =
                AssignmentSystem::forecast_role_change(colonists, colonist.id, next_role);
            if hovered {
                hovered_forecast = Some(forecast.clone());
                hovered_name = Some(colonist.name.clone());
            }
            draw_text(
                &style::truncate_text(
                    &format!(
                        "{} -> {}",
                        colonist.job_preference.label(),
                        next_role.label()
                    ),
                    15,
                ),
                rect.x + 10.0,
                rect.y + 34.0,
                style::TINY_SIZE,
                style::HEADING_BLUE,
            );
        }
    }

    let selected_colonist =
        selected_colonist_id.and_then(|id| colonists.iter().find(|colonist| colonist.id == id));
    let selected_warning = selected_colonist
        .and_then(|colonist| assignment_pin_warning(colonist, colonists, technology));
    if let Some(colonist) = selected_colonist {
        draw_assign_batch_controls(context, colonist);
    }
    let footer = selected_colonist
        .map(|colonist| {
            let filter_note = active_building_filter
                .map(|id| format!(" | room filter #{}", id))
                .unwrap_or_default();
            format!(
                "Selected {} | click rooms to pin | right-click room to filter{}",
                colonist.name, filter_note
            )
        })
        .unwrap_or_else(|| {
            "Roles, pair directives, and space directives shape work blocks.".to_string()
        });
    let footer = selected_warning
        .as_ref()
        .map(|warning| warning.detail.clone())
        .unwrap_or(footer);
    draw_text(
        &style::truncate_text(&footer, 76),
        context.x + 18.0,
        context.y + 111.0,
        style::TINY_SIZE,
        if selected_warning.is_some() {
            style::ALERT_RED
        } else {
            style::TEXT_MUTED
        },
    );

    if let Some(filter) = hovered_filter {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            filter.tooltip_title(),
            filter.tooltip_body(),
        );
    } else if let Some(sort) = hovered_sort {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            sort.tooltip_title(),
            sort.tooltip_body(),
        );
    } else if hovered_role_filter {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            "Role filter",
            "Cycle the visible roster between all roles and one work-role group.",
        );
    } else if let (Some(name), Some(detail)) = (hovered_name.clone(), hovered_directive) {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &name, &detail);
    } else if let (Some(name), Some(forecast)) = (hovered_name, hovered_forecast) {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &name, &forecast.detail);
    }
}

pub(super) fn draw_assign_roster_controls(
    context: Rect,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    hovered_filter: &mut Option<AssignRosterFilter>,
    hovered_sort: &mut Option<AssignRosterSort>,
    hovered_role_filter: &mut bool,
) {
    let mouse = mouse_position().into();

    for (index, filter) in AssignRosterFilter::all().iter().enumerate() {
        let rect = assign_filter_rect(context, index);
        let hovered = rect.contains(mouse);
        if hovered {
            *hovered_filter = Some(*filter);
        }
        style::draw_button(rect, *filter == active_filter, hovered);
        draw_text(
            filter.label(),
            rect.x + 5.0,
            rect.y + 12.0,
            style::TINY_SIZE,
            if *filter == active_filter {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_MUTED
            },
        );
    }

    for (index, sort) in AssignRosterSort::all().iter().enumerate() {
        let rect = assign_sort_rect(context, index);
        let hovered = rect.contains(mouse);
        if hovered {
            *hovered_sort = Some(*sort);
        }
        style::draw_button(rect, *sort == active_sort, hovered);
        draw_text(
            sort.label(),
            rect.x + 5.0,
            rect.y + 12.0,
            style::TINY_SIZE,
            if *sort == active_sort {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_MUTED
            },
        );
    }

    let role = assign_role_filter_rect(context);
    let role_hovered = role.contains(mouse);
    if role_hovered {
        *hovered_role_filter = true;
    }
    style::draw_button(role, active_role_filter.is_some(), role_hovered);
    draw_text(
        &format!("R:{}", assign_role_filter_label(active_role_filter)),
        role.x + 4.0,
        role.y + 12.0,
        style::TINY_SIZE,
        if active_role_filter.is_some() {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
}

pub(super) fn draw_assign_batch_controls(context: Rect, selected_colonist: &Colonist) {
    let mouse = mouse_position().into();
    let home_enabled = selected_colonist.assigned_habitat.is_some();
    let work_enabled = selected_colonist.assigned_workplace.is_some();
    let mut hovered_action = None;

    for (index, action) in AssignBatchAction::all().iter().enumerate() {
        let rect = assign_batch_rect(context, index);
        let enabled = if action.copies_home() {
            home_enabled
        } else {
            work_enabled
        };
        let hovered = rect.contains(mouse);
        if hovered {
            hovered_action = Some(*action);
        }
        style::draw_button(rect, false, enabled && hovered);
        draw_text(
            action.label(),
            rect.x + 5.0,
            rect.y + 12.0,
            style::TINY_SIZE,
            if enabled {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_MUTED
            },
        );
    }

    if let Some(action) = hovered_action {
        draw_tooltip_near_mouse(
            toolbar_tooltip_bounds(context),
            action.tooltip_title(),
            action.tooltip_body(),
        );
    }
}

pub(super) fn draw_assign_page_controls(context: Rect, current_page: usize, page_count: usize) {
    let previous = assign_page_previous_rect(context);
    let next = assign_page_next_rect(context);
    let mouse = mouse_position().into();
    let can_go_previous = current_page > 0;
    let can_go_next = current_page + 1 < page_count;

    style::draw_button(previous, false, can_go_previous && previous.contains(mouse));
    style::draw_button(next, false, can_go_next && next.contains(mouse));
    draw_text(
        "<",
        previous.x + 10.0,
        previous.y + 12.0,
        style::TINY_SIZE,
        if can_go_previous {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_text(
        ">",
        next.x + 10.0,
        next.y + 12.0,
        style::TINY_SIZE,
        if can_go_next {
            style::TEXT_PRIMARY
        } else {
            style::TEXT_MUTED
        },
    );
    draw_text(
        &format!("{}/{}", current_page + 1, page_count),
        context.x + context.w - 63.0,
        context.y + 25.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
}

pub(super) struct AssignPairAction {
    pub(super) label: String,
    pub(super) detail: String,
    pub(super) directive: PairDirective,
}

pub(super) fn assign_roster_page_count(
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> usize {
    let selected_exists = selected_colonist_id
        .and_then(|id| colonists.iter().position(|colonist| colonist.id == id))
        .is_some();
    let other_count = colonists
        .iter()
        .filter(|colonist| Some(colonist.id) != selected_colonist_id)
        .filter(|colonist| {
            assign_roster_filter_matches(colonist, active_filter, active_role_filter)
                && assign_building_filter_matches(colonist, active_building_filter)
        })
        .count();
    let page_size = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(usize::from(selected_exists));

    ((other_count + page_size - 1) / page_size).max(1)
}

pub(super) fn assign_visible_colonists<'a>(
    colonists: &'a [Colonist],
    selected_colonist_id: Option<u32>,
    page: usize,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<&'a Colonist> {
    let mut visible = Vec::new();

    let selected_id =
        selected_colonist_id.filter(|id| colonists.iter().any(|colonist| colonist.id == *id));
    if let Some(selected_id) = selected_id {
        if let Some(colonist) = colonists.iter().find(|colonist| colonist.id == selected_id) {
            visible.push(colonist);
        }
    }

    let open_slots = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(visible.len());
    let page = page.min(
        assign_roster_page_count(
            colonists,
            selected_colonist_id,
            active_filter,
            active_role_filter,
            active_building_filter,
        ) - 1,
    );

    let roster = assign_sorted_roster(
        colonists,
        selected_id,
        active_filter,
        active_sort,
        active_role_filter,
        active_building_filter,
    );
    visible.extend(roster.into_iter().skip(page * open_slots).take(open_slots));

    visible
}

pub(super) fn assign_sorted_roster<'a>(
    colonists: &'a [Colonist],
    selected_id: Option<u32>,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<&'a Colonist> {
    let mut roster = colonists
        .iter()
        .filter(|colonist| Some(colonist.id) != selected_id)
        .filter(|colonist| {
            assign_roster_filter_matches(colonist, active_filter, active_role_filter)
                && assign_building_filter_matches(colonist, active_building_filter)
        })
        .collect::<Vec<_>>();

    match active_sort {
        AssignRosterSort::Roster => {}
        AssignRosterSort::Mood => roster.sort_by(|left, right| {
            left.mood
                .partial_cmp(&right.mood)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.id.cmp(&right.id))
        }),
        AssignRosterSort::Bond => roster.sort_by(|left, right| {
            relationship_pressure_score(right)
                .cmp(&relationship_pressure_score(left))
                .then_with(|| left.id.cmp(&right.id))
        }),
    }

    roster
}

pub(super) fn assign_roster_filter_matches(
    colonist: &Colonist,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
) -> bool {
    let relationship_match = match active_filter {
        AssignRosterFilter::All => true,
        AssignRosterFilter::Risk => colonist.relationships.values().any(|value| *value <= -10),
        AssignRosterFilter::Support => colonist.relationships.values().any(|value| *value >= 10),
        AssignRosterFilter::Pinned => {
            colonist.assigned_habitat.is_some() || colonist.assigned_workplace.is_some()
        }
    };
    relationship_match && active_role_filter.is_none_or(|role| colonist.job_preference == role)
}

pub(super) fn relationship_pressure_score(colonist: &Colonist) -> i32 {
    colonist
        .relationships
        .values()
        .map(|value| value.abs())
        .max()
        .unwrap_or(0)
}

pub(super) fn assign_building_filter_matches(
    colonist: &Colonist,
    building_id: Option<u32>,
) -> bool {
    building_id.is_none_or(|id| {
        colonist.assigned_habitat == Some(id) || colonist.assigned_workplace == Some(id)
    })
}

pub(super) fn assign_role_filter_label(role: Option<JobPreference>) -> &'static str {
    match role {
        None => "ALL",
        Some(JobPreference::Explorer) => "EXP",
        Some(JobPreference::Builder) => "BLD",
        Some(JobPreference::Cook) => "CK",
        Some(JobPreference::Hauler) => "HL",
        Some(JobPreference::None) => "GEN",
    }
}

pub(super) fn assign_pair_action(
    colonists: &[Colonist],
    selected_id: u32,
    target_id: u32,
) -> Option<AssignPairAction> {
    let current =
        RelationshipDirectiveSystem::directive_for_pair(colonists, selected_id, target_id);
    let directive = current.or_else(|| {
        RelationshipDirectiveSystem::recommended_directive(colonists, selected_id, target_id)
    })?;
    let value =
        RelationshipDirectiveSystem::average_relationship(colonists, selected_id, target_id)
            .unwrap_or(0);
    let label = match current {
        Some(active) => format!("{} set {:+}", active.label(), value),
        None => format!("{} {:+}", directive.label(), value),
    };
    let detail = RelationshipDirectiveSystem::directive_detail(colonists, selected_id, target_id);

    Some(AssignPairAction {
        label,
        detail,
        directive,
    })
}

pub(super) fn selected_assignment_label(colonist: &Colonist) -> String {
    let home = colonist
        .assigned_habitat
        .map(|id| format!("H#{}", id))
        .unwrap_or_else(|| "H--".to_string());
    let work = colonist
        .assigned_workplace
        .map(|id| format!("W#{}", id))
        .unwrap_or_else(|| "W--".to_string());
    format!("{} {}", home, work)
}

pub(super) fn selected_assignment_detail(
    colonist: &Colonist,
    colonists: &[Colonist],
    technology: &TechnologyState,
) -> String {
    let base = format!(
        "Click this card to cycle role. Click a compatible map building to pin or clear recovery/work space. Current pins: {}.",
        selected_assignment_label(colonist)
    );
    assignment_pin_warning(colonist, colonists, technology)
        .map(|warning| format!("{} {}", base, warning.detail))
        .unwrap_or(base)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AssignmentPinWarning {
    pub(super) label: String,
    pub(super) detail: String,
}

pub(super) fn assignment_pin_warning(
    colonist: &Colonist,
    colonists: &[Colonist],
    technology: &TechnologyState,
) -> Option<AssignmentPinWarning> {
    if let Some(habitat_id) = colonist.assigned_habitat {
        let count = colonists
            .iter()
            .filter(|candidate| candidate.assigned_habitat == Some(habitat_id))
            .count() as u32;
        let capacity = 2 + technology.habitat_capacity_bonus();
        if count > capacity {
            return Some(AssignmentPinWarning {
                label: "CAP".to_string(),
                detail: format!(
                    "Habitat #{} over capacity: {}/{} pinned survivors.",
                    habitat_id, count, capacity
                ),
            });
        }

        if let Some((name, value)) = first_assignment_conflict(
            colonist,
            colonists,
            AssignmentPinLocation::Habitat(habitat_id),
        ) {
            return Some(AssignmentPinWarning {
                label: "TENSE".to_string(),
                detail: format!(
                    "{}: {} {:+} in H#{}. Pin another room or use Apart.",
                    name,
                    relationship_label(value),
                    value,
                    habitat_id
                ),
            });
        }
    }

    if let Some(workplace_id) = colonist.assigned_workplace {
        if let Some((name, value)) = first_assignment_conflict(
            colonist,
            colonists,
            AssignmentPinLocation::Work(workplace_id),
        ) {
            return Some(AssignmentPinWarning {
                label: "TENSE".to_string(),
                detail: format!(
                    "{}: {} {:+} at W#{}. Pin another space or use Apart.",
                    name,
                    relationship_label(value),
                    value,
                    workplace_id
                ),
            });
        }
    }

    None
}

#[derive(Clone, Copy)]
pub(super) enum AssignmentPinLocation {
    Habitat(u32),
    Work(u32),
}

pub(super) fn first_assignment_conflict(
    colonist: &Colonist,
    colonists: &[Colonist],
    location: AssignmentPinLocation,
) -> Option<(String, i32)> {
    colonists
        .iter()
        .filter(|candidate| candidate.id != colonist.id)
        .filter(|candidate| match location {
            AssignmentPinLocation::Habitat(id) => candidate.assigned_habitat == Some(id),
            AssignmentPinLocation::Work(id) => candidate.assigned_workplace == Some(id),
        })
        .filter_map(|candidate| {
            let value = RelationshipDirectiveSystem::average_relationship(
                colonists,
                colonist.id,
                candidate.id,
            )
            .unwrap_or(0);
            (value <= -10).then(|| (candidate.name.clone(), value))
        })
        .min_by_key(|(_, value)| *value)
}
