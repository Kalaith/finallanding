use super::PageAction;
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, HitTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSearchAction {
    Focus,
    Clear,
    Export,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogFilter {
    All,
    Tense,
    Support,
}

impl LogFilter {
    pub fn all() -> &'static [LogFilter] {
        &[LogFilter::All, LogFilter::Tense, LogFilter::Support]
    }

    pub fn label(self) -> &'static str {
        match self {
            LogFilter::All => "ALL",
            LogFilter::Tense => "TENSE",
            LogFilter::Support => "PLUS",
        }
    }
}

pub fn log_page_previous_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 96.0, context.y + 72.0, 28.0, 17.0)
}

pub fn log_page_next_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 34.0, context.y + 72.0, 28.0, 17.0)
}

pub fn log_page_action_at(context: Rect, x: f32, y: f32) -> Option<PageAction> {
    hit_test(
        [
            HitTarget::new(log_page_previous_rect(context), PageAction::Previous),
            HitTarget::new(log_page_next_rect(context), PageAction::Next),
        ],
        vec2(x, y),
    )
}

pub fn log_search_rect(context: Rect) -> Rect {
    Rect::new(context.x + 72.0, context.y + 13.0, 200.0, 17.0)
}

pub fn log_search_clear_rect(context: Rect) -> Rect {
    Rect::new(context.x + 278.0, context.y + 13.0, 42.0, 17.0)
}

pub fn log_search_export_rect(context: Rect) -> Rect {
    Rect::new(context.x + 326.0, context.y + 13.0, 46.0, 17.0)
}

pub fn log_search_action_at(context: Rect, x: f32, y: f32) -> Option<LogSearchAction> {
    hit_test(
        [
            HitTarget::new(log_search_rect(context), LogSearchAction::Focus),
            HitTarget::new(log_search_clear_rect(context), LogSearchAction::Clear),
            HitTarget::new(log_search_export_rect(context), LogSearchAction::Export),
        ],
        vec2(x, y),
    )
}

pub fn log_filter_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + 120.0 + index as f32 * 50.0,
        context.y + 72.0,
        46.0,
        17.0,
    )
}

pub fn log_filter_at(context: Rect, x: f32, y: f32) -> Option<LogFilter> {
    hit_test(
        LogFilter::all()
            .iter()
            .enumerate()
            .map(|(index, filter)| HitTarget::new(log_filter_rect(context, index), *filter)),
        vec2(x, y),
    )
}

pub fn log_timeline_row_rect(context: Rect, index: usize) -> Rect {
    let y = context.y + 94.0 + index as f32 * 13.0;
    Rect::new(context.x + 12.0, y - 11.0, context.w - 24.0, 13.0)
}

pub fn log_timeline_row_at(context: Rect, row_count: usize, x: f32, y: f32) -> Option<usize> {
    hit_test(
        (0..row_count.min(3))
            .map(|index| HitTarget::new(log_timeline_row_rect(context, index), index)),
        vec2(x, y),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn center(rect: Rect) -> (f32, f32) {
        (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
    }

    #[test]
    fn test_log_page_hit_zones_match_archive_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (prev_x, prev_y) = center(log_page_previous_rect(context));
        let (next_x, next_y) = center(log_page_next_rect(context));

        assert_eq!(
            log_page_action_at(context, prev_x, prev_y),
            Some(PageAction::Previous)
        );
        assert_eq!(
            log_page_action_at(context, next_x, next_y),
            Some(PageAction::Next)
        );
        assert_eq!(
            log_page_action_at(context, context.x + 18.0, context.y + 82.0),
            None
        );
    }

    #[test]
    fn test_log_filter_hit_zones_match_filter_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (all_x, all_y) = center(log_filter_rect(context, 0));
        let (support_x, support_y) = center(log_filter_rect(context, 2));

        assert_eq!(log_filter_at(context, all_x, all_y), Some(LogFilter::All));
        assert_eq!(
            log_filter_at(context, support_x, support_y),
            Some(LogFilter::Support)
        );
        assert_eq!(
            log_filter_at(context, context.x + 18.0, context.y + 82.0),
            None
        );
    }

    #[test]
    fn test_log_search_hit_zones_match_search_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (search_x, search_y) = center(log_search_rect(context));
        let (clear_x, clear_y) = center(log_search_clear_rect(context));
        let (export_x, export_y) = center(log_search_export_rect(context));

        assert_eq!(
            log_search_action_at(context, search_x, search_y),
            Some(LogSearchAction::Focus)
        );
        assert_eq!(
            log_search_action_at(context, clear_x, clear_y),
            Some(LogSearchAction::Clear)
        );
        assert_eq!(
            log_search_action_at(context, export_x, export_y),
            Some(LogSearchAction::Export)
        );
        assert_eq!(
            log_search_action_at(context, context.x + 18.0, context.y + 82.0),
            None
        );
    }

    #[test]
    fn test_log_timeline_hit_zones_match_visible_rows() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (first_x, first_y) = center(log_timeline_row_rect(context, 0));
        let (third_x, third_y) = center(log_timeline_row_rect(context, 2));

        assert_eq!(log_timeline_row_at(context, 3, first_x, first_y), Some(0));
        assert_eq!(log_timeline_row_at(context, 3, third_x, third_y), Some(2));
        assert_eq!(log_timeline_row_at(context, 2, third_x, third_y), None);
    }
}
