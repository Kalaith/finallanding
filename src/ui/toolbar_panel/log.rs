use super::*;

pub(super) fn draw_log_context(
    context: Rect,
    logs: &[ColonyLogEntry],
    social_history: &[SocialHistoryEntry],
    social_history_page: usize,
    social_history_filter: LogFilter,
    social_history_query: &str,
    social_history_search_active: bool,
    selected_social_history_day: Option<u32>,
    summary: &ColonyPressureSummary,
) {
    let mut hovered_history = None;
    draw_log_search_control(context, social_history_query, social_history_search_active);
    let social_brief = social_brief_lines(summary);
    draw_text(
        &social_brief.header,
        context.x + 18.0,
        context.y + 51.0,
        style::TINY_SIZE,
        social_brief.color,
    );
    draw_text(
        &style::truncate_text(&social_brief.detail, 72),
        context.x + 18.0,
        context.y + 68.0,
        style::TINY_SIZE,
        style::TEXT_BODY,
    );

    let page_count =
        social_history_page_count(social_history, social_history_filter, social_history_query);
    let current_page = social_history_page.min(page_count.saturating_sub(1));
    let timeline = social_timeline_rows(
        social_history,
        social_history_filter,
        social_history_query,
        current_page,
    );
    if !social_history.is_empty() {
        draw_text(
            "SOCIAL TIMELINE",
            context.x + 18.0,
            context.y + 82.0,
            style::TINY_SIZE,
            style::HEADING_BLUE,
        );
        draw_log_filter_controls(context, social_history_filter);
        if page_count > 1 {
            draw_log_page_controls(context, current_page, page_count);
        }

        if timeline.is_empty() {
            draw_text(
                "No matching daily reports in this archive.",
                context.x + 18.0,
                context.y + 102.0,
                style::TINY_SIZE,
                style::TEXT_MUTED,
            );
            return;
        }

        for (index, row) in timeline.iter().enumerate() {
            let y = context.y + 94.0 + index as f32 * 13.0;
            let rect = log_timeline_row_rect(context, index);
            if rect.contains(mouse_position().into()) {
                hovered_history = Some(row);
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.1, 0.14, 0.15, 0.7),
                );
            }
            if selected_social_history_day == Some(row.day) {
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.18, 0.22, 0.2, 0.82),
                );
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, style::ACCENT_GOLD);
            }
            draw_rectangle(rect.x, rect.y, 3.0, rect.h, row.color);
            draw_text(
                &format!("D{}", row.day),
                rect.x + 9.0,
                y,
                style::TINY_SIZE,
                row.color,
            );
            draw_text(
                &style::truncate_text(&row.title, 34),
                rect.x + 39.0,
                y,
                style::TINY_SIZE,
                style::TEXT_BODY,
            );
            draw_text(
                &row.metrics,
                rect.x + rect.w - 104.0,
                y,
                style::TINY_SIZE,
                style::TEXT_MUTED,
            );
        }

        if let Some(row) = hovered_history {
            draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &row.title, &row.detail);
        }
        if let Some(entry) =
            selected_social_history_entry(social_history, selected_social_history_day)
        {
            draw_social_report_drilldown(context, entry);
        }
        return;
    }

    let mut hovered_log = None;
    for (index, log) in logs.iter().rev().take(2).enumerate() {
        let y = context.y + 91.0 + index as f32 * 20.0;
        let row = Rect::new(context.x + 12.0, y - 14.0, context.w - 24.0, 18.0);
        if row.contains(mouse_position().into()) {
            hovered_log = Some(log);
            draw_rectangle(
                row.x,
                row.y,
                row.w,
                row.h,
                Color::new(0.1, 0.14, 0.15, 0.65),
            );
        }
        draw_text(
            category_prefix(log.category),
            context.x + 18.0,
            y,
            style::TINY_SIZE,
            style::HEADING_BLUE,
        );
        draw_text(
            &style::truncate_text(
                &format!("{:02}:{:02} {}", log.hour, log.minute, log.title),
                64,
            ),
            context.x + 52.0,
            y,
            style::TINY_SIZE,
            style::TEXT_BODY,
        );
    }

    if let Some(log) = hovered_log {
        draw_tooltip_near_mouse(toolbar_tooltip_bounds(context), &log.title, &log.detail);
    }
}

pub(super) fn draw_log_search_control(context: Rect, query: &str, active: bool) {
    let search = log_search_rect(context);
    let clear = log_search_clear_rect(context);
    let export = log_search_export_rect(context);
    let mouse = mouse_position().into();
    style::draw_button(search, active, search.contains(mouse));
    style::draw_button(clear, false, !query.is_empty() && clear.contains(mouse));
    style::draw_button(export, false, export.contains(mouse));

    let mut label = if query.is_empty() {
        "SEARCH REPORTS".to_string()
    } else {
        style::truncate_text(query, 25)
    };
    if active {
        label.push('|');
    }

    draw_text(
        &label,
        search.x + 7.0,
        search.y + 12.0,
        style::TINY_SIZE,
        if query.is_empty() {
            style::TEXT_MUTED
        } else {
            style::TEXT_PRIMARY
        },
    );
    draw_text(
        "CLR",
        clear.x + 8.0,
        clear.y + 12.0,
        style::TINY_SIZE,
        if query.is_empty() {
            style::TEXT_MUTED
        } else {
            style::TEXT_PRIMARY
        },
    );
    draw_text(
        "EXP",
        export.x + 9.0,
        export.y + 12.0,
        style::TINY_SIZE,
        style::TEXT_PRIMARY,
    );
}

pub(super) fn draw_social_report_drilldown(context: Rect, entry: &SocialHistoryEntry) {
    let rect = Rect::new(
        context.x + context.w - 330.0,
        (context.y - 78.0).max(70.0),
        320.0,
        68.0,
    );
    style::draw_deep_panel(rect);
    draw_rectangle(rect.x, rect.y, 4.0, rect.h, social_history_color(entry));
    draw_text(
        &format!(
            "DAY {}: {}",
            entry.day,
            style::truncate_text(&entry.title, 34)
        ),
        rect.x + 12.0,
        rect.y + 17.0,
        style::TINY_SIZE,
        style::TEXT_PRIMARY,
    );
    draw_text(
        &style::truncate_text(&entry.detail, 58),
        rect.x + 12.0,
        rect.y + 37.0,
        style::TINY_SIZE,
        style::TEXT_BODY,
    );
    draw_text(
        &style::truncate_text(&entry.recommendation, 58),
        rect.x + 12.0,
        rect.y + 55.0,
        style::TINY_SIZE,
        style::HEADING_BLUE,
    );
}

pub(super) fn draw_log_filter_controls(context: Rect, active_filter: LogFilter) {
    for (index, filter) in LogFilter::all().iter().enumerate() {
        let rect = log_filter_rect(context, index);
        let active = *filter == active_filter;
        style::draw_button(rect, active, rect.contains(mouse_position().into()));
        draw_text(
            filter.label(),
            rect.x + 6.0,
            rect.y + 12.0,
            style::TINY_SIZE,
            if active {
                style::TEXT_PRIMARY
            } else {
                style::TEXT_MUTED
            },
        );
    }
}

pub(super) fn draw_log_page_controls(context: Rect, current_page: usize, page_count: usize) {
    let previous = log_page_previous_rect(context);
    let next = log_page_next_rect(context);
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
        context.y + 84.0,
        style::TINY_SIZE,
        style::TEXT_MUTED,
    );
}

pub(super) struct SocialTimelineRow {
    pub(super) day: u32,
    pub(super) title: String,
    pub(super) detail: String,
    pub(super) metrics: String,
    pub(super) color: Color,
}

pub fn social_history_page_count(
    history: &[SocialHistoryEntry],
    filter: LogFilter,
    query: &str,
) -> usize {
    let count = history
        .iter()
        .filter(|entry| social_history_matches_filter(entry, filter))
        .filter(|entry| social_history_matches_query(entry, query))
        .count();
    ((count + SOCIAL_TIMELINE_PAGE_SIZE - 1) / SOCIAL_TIMELINE_PAGE_SIZE).max(1)
}

pub(super) fn social_timeline_rows(
    history: &[SocialHistoryEntry],
    filter: LogFilter,
    query: &str,
    page: usize,
) -> Vec<SocialTimelineRow> {
    let page = page.min(social_history_page_count(history, filter, query).saturating_sub(1));
    history
        .iter()
        .rev()
        .filter(|entry| social_history_matches_filter(entry, filter))
        .filter(|entry| social_history_matches_query(entry, query))
        .skip(page * SOCIAL_TIMELINE_PAGE_SIZE)
        .take(SOCIAL_TIMELINE_PAGE_SIZE)
        .map(|entry| SocialTimelineRow {
            day: entry.day,
            title: entry.title.clone(),
            detail: format!("{} {}", entry.detail, entry.recommendation),
            metrics: format!(
                "M{:.0} R{:+.0} T{}",
                entry.average_mood, entry.average_relationship, entry.strained_pairs
            ),
            color: social_history_color(entry),
        })
        .collect()
}

pub fn social_timeline_day_at(
    history: &[SocialHistoryEntry],
    filter: LogFilter,
    query: &str,
    page: usize,
    row_index: usize,
) -> Option<u32> {
    social_timeline_rows(history, filter, query, page)
        .get(row_index)
        .map(|row| row.day)
}

pub(super) fn selected_social_history_entry(
    history: &[SocialHistoryEntry],
    selected_day: Option<u32>,
) -> Option<&SocialHistoryEntry> {
    let day = selected_day?;
    history.iter().find(|entry| entry.day == day)
}

pub(super) fn social_history_matches_filter(entry: &SocialHistoryEntry, filter: LogFilter) -> bool {
    match filter {
        LogFilter::All => true,
        LogFilter::Tense => entry.strained_pairs > 0 || entry.average_relationship < -5.0,
        LogFilter::Support => entry.close_pairs > 0 || entry.average_relationship > 8.0,
    }
}

pub(super) fn social_history_matches_query(entry: &SocialHistoryEntry, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return true;
    }

    let needle = query.to_ascii_lowercase();
    entry.title.to_ascii_lowercase().contains(&needle)
        || entry.detail.to_ascii_lowercase().contains(&needle)
        || entry.recommendation.to_ascii_lowercase().contains(&needle)
        || format!("day {}", entry.day).contains(&needle)
        || entry.day.to_string().contains(&needle)
}

pub(super) fn social_history_color(entry: &SocialHistoryEntry) -> Color {
    if entry.strained_pairs > 0 || entry.average_relationship < -5.0 {
        style::ALERT_RED
    } else if entry.close_pairs > 0 || entry.average_relationship > 8.0 {
        style::BAR_GREEN
    } else {
        style::HEADING_BLUE
    }
}

pub(super) struct SocialBriefLines {
    pub(super) header: String,
    pub(super) detail: String,
    pub(super) color: Color,
}

pub(super) fn social_brief_lines(summary: &ColonyPressureSummary) -> SocialBriefLines {
    let color = if summary.strained_pairs > 0 {
        style::ALERT_RED
    } else if summary.close_pairs > 0 {
        style::BAR_GREEN
    } else {
        style::HEADING_BLUE
    };

    let header = format!(
        "Social pressure: mood {:.0} | close {} | tense {}",
        summary.average_mood, summary.close_pairs, summary.strained_pairs
    );
    let detail = if let Some(pair) = summary
        .weakest_pair
        .as_ref()
        .filter(|pair| pair.value <= -10)
    {
        pair_line("Watch", pair)
    } else if let Some(pair) = summary
        .strongest_pair
        .as_ref()
        .filter(|pair| pair.value >= 10)
    {
        pair_line("Protect", pair)
    } else {
        "No strong social signal yet; routine will shape the first bonds.".to_string()
    };

    SocialBriefLines {
        header,
        detail,
        color,
    }
}

pub(super) fn pair_line(prefix: &str, pair: &RelationshipPairSummary) -> String {
    format!(
        "{} {} / {}: {} {:+}",
        prefix, pair.first_name, pair.second_name, pair.label, pair.value
    )
}

pub(super) fn toolbar_tooltip_bounds(context: Rect) -> Rect {
    Rect::new(
        context.x,
        (context.y - 58.0).max(0.0),
        context.w,
        context.h + 58.0,
    )
}

pub(super) fn assignment_pressure_color(pressure: AssignmentPressure) -> Color {
    match pressure {
        AssignmentPressure::Supported => style::BAR_GREEN,
        AssignmentPressure::Neutral => style::HEADING_BLUE,
        AssignmentPressure::Tense => style::ALERT_RED,
    }
}

pub(super) fn directive_color(directive: PairDirective) -> Color {
    match directive {
        PairDirective::Pair => style::BAR_GREEN,
        PairDirective::Separate => style::ALERT_RED,
    }
}

pub(super) fn category_prefix(category: LogCategory) -> &'static str {
    match category {
        LogCategory::Time => "TIME",
        LogCategory::Social => "SOC",
        LogCategory::Work => "WORK",
        LogCategory::Mood => "MOOD",
        LogCategory::Resource => "RES",
        LogCategory::Mission => "MIS",
        LogCategory::Technology => "TECH",
        LogCategory::Colony => "COL",
        LogCategory::System => "SYS",
    }
}
