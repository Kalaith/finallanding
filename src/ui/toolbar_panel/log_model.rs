use super::SOCIAL_TIMELINE_PAGE_SIZE;
use crate::data::colonist::RelationshipBand;
use crate::data::event_log::SocialHistoryEntry;
use crate::systems::summary_system::{ColonyPressureSummary, RelationshipPairSummary};
use crate::ui::hit_zones::LogFilter;
use crate::ui::style;
use macroquad::prelude::Color;

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
    count.div_ceil(SOCIAL_TIMELINE_PAGE_SIZE).max(1)
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
        LogFilter::Tense => social_history_signal(entry) == SocialHistorySignal::Tense,
        LogFilter::Support => social_history_signal(entry) == SocialHistorySignal::Support,
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
    match social_history_signal(entry) {
        SocialHistorySignal::Tense => style::ALERT_RED,
        SocialHistorySignal::Support => style::BAR_GREEN,
        SocialHistorySignal::Neutral => style::HEADING_BLUE,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SocialHistorySignal {
    Tense,
    Support,
    Neutral,
}

fn social_history_signal(entry: &SocialHistoryEntry) -> SocialHistorySignal {
    if entry.strained_pairs > 0 || entry.average_relationship < -5.0 {
        SocialHistorySignal::Tense
    } else if entry.close_pairs > 0 || entry.average_relationship > 8.0 {
        SocialHistorySignal::Support
    } else {
        SocialHistorySignal::Neutral
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
        .filter(|pair| RelationshipBand::from_value(pair.value).is_risk())
    {
        pair_line("Watch", pair)
    } else if let Some(pair) = summary
        .strongest_pair
        .as_ref()
        .filter(|pair| RelationshipBand::from_value(pair.value).is_support())
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

#[cfg(test)]
mod tests {
    use super::*;

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
