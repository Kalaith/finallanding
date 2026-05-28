use super::PageAction;
pub use crate::data::assign_roster::{AssignRosterFilter, AssignRosterSort};
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, rect_contains_point, HitTarget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignBatchAction {
    PageHome,
    PageWork,
    AllHome,
    AllWork,
}

impl AssignBatchAction {
    pub fn all() -> &'static [AssignBatchAction] {
        &[
            AssignBatchAction::PageHome,
            AssignBatchAction::PageWork,
            AssignBatchAction::AllHome,
            AssignBatchAction::AllWork,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            AssignBatchAction::PageHome => "P-H",
            AssignBatchAction::PageWork => "P-W",
            AssignBatchAction::AllHome => "ALL-H",
            AssignBatchAction::AllWork => "ALL-W",
        }
    }

    pub fn tooltip_title(self) -> &'static str {
        match self {
            AssignBatchAction::PageHome => "Copy page home",
            AssignBatchAction::PageWork => "Copy page work",
            AssignBatchAction::AllHome => "Copy colony home",
            AssignBatchAction::AllWork => "Copy colony work",
        }
    }

    pub fn tooltip_body(self) -> &'static str {
        match self {
            AssignBatchAction::PageHome => {
                "Copy this survivor's Habitat pin to compatible survivors on the visible roster page."
            }
            AssignBatchAction::PageWork => {
                "Copy this survivor's work pin to compatible survivors on the visible roster page."
            }
            AssignBatchAction::AllHome => {
                "Copy this survivor's Habitat pin to every compatible survivor in the colony."
            }
            AssignBatchAction::AllWork => {
                "Copy this survivor's work pin to every compatible survivor in the colony."
            }
        }
    }

    pub fn copies_home(self) -> bool {
        matches!(
            self,
            AssignBatchAction::PageHome | AssignBatchAction::AllHome
        )
    }

    pub fn targets_all(self) -> bool {
        matches!(
            self,
            AssignBatchAction::AllHome | AssignBatchAction::AllWork
        )
    }
}

pub fn assign_page_previous_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 96.0, context.y + 13.0, 28.0, 17.0)
}

pub fn assign_page_next_rect(context: Rect) -> Rect {
    Rect::new(context.x + context.w - 34.0, context.y + 13.0, 28.0, 17.0)
}

pub fn assign_page_action_at(context: Rect, x: f32, y: f32) -> Option<PageAction> {
    hit_test(
        [
            HitTarget::new(assign_page_previous_rect(context), PageAction::Previous),
            HitTarget::new(assign_page_next_rect(context), PageAction::Next),
        ],
        vec2(x, y),
    )
}

pub fn assign_filter_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + 72.0 + index as f32 * 44.0,
        context.y + 13.0,
        40.0,
        17.0,
    )
}

pub fn assign_filter_at(context: Rect, x: f32, y: f32) -> Option<AssignRosterFilter> {
    let point = vec2(x, y);
    hit_test(
        AssignRosterFilter::all()
            .iter()
            .enumerate()
            .map(|(index, filter)| HitTarget::new(assign_filter_rect(context, index), *filter)),
        point,
    )
}

pub fn assign_sort_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + 252.0 + index as f32 * 42.0,
        context.y + 13.0,
        40.0,
        17.0,
    )
}

pub fn assign_sort_at(context: Rect, x: f32, y: f32) -> Option<AssignRosterSort> {
    hit_test(
        AssignRosterSort::all()
            .iter()
            .enumerate()
            .map(|(index, sort)| HitTarget::new(assign_sort_rect(context, index), *sort)),
        vec2(x, y),
    )
}

pub fn assign_role_filter_rect(context: Rect) -> Rect {
    Rect::new(context.x + 380.0, context.y + 13.0, 40.0, 17.0)
}

pub fn assign_role_filter_at(context: Rect, x: f32, y: f32) -> bool {
    rect_contains_point(assign_role_filter_rect(context), vec2(x, y))
}

pub fn assign_batch_rect(context: Rect, index: usize) -> Rect {
    Rect::new(
        context.x + context.w - 222.0 + index as f32 * 53.0,
        context.y + 94.0,
        48.0,
        17.0,
    )
}

pub fn assign_batch_action_at(context: Rect, x: f32, y: f32) -> Option<AssignBatchAction> {
    hit_test(
        AssignBatchAction::all()
            .iter()
            .enumerate()
            .map(|(index, action)| HitTarget::new(assign_batch_rect(context, index), *action)),
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
    fn test_assign_page_hit_zones_match_roster_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (prev_x, prev_y) = center(assign_page_previous_rect(context));
        let (next_x, next_y) = center(assign_page_next_rect(context));

        assert_eq!(
            assign_page_action_at(context, prev_x, prev_y),
            Some(PageAction::Previous)
        );
        assert_eq!(
            assign_page_action_at(context, next_x, next_y),
            Some(PageAction::Next)
        );
        assert_eq!(
            assign_page_action_at(context, context.x + 18.0, context.y + 82.0),
            None
        );
    }

    #[test]
    fn test_assign_filter_and_sort_hit_zones_match_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (risk_x, risk_y) = center(assign_filter_rect(context, 1));
        let (bond_x, bond_y) = center(assign_sort_rect(context, 2));
        let (role_x, role_y) = center(assign_role_filter_rect(context));

        assert_eq!(
            assign_filter_at(context, risk_x, risk_y),
            Some(AssignRosterFilter::Risk)
        );
        assert_eq!(
            assign_sort_at(context, bond_x, bond_y),
            Some(AssignRosterSort::Bond)
        );
        assert!(assign_role_filter_at(context, role_x, role_y));
        assert_eq!(
            assign_filter_at(context, context.x + context.w - 20.0, context.y + 21.0),
            None
        );
    }

    #[test]
    fn test_assign_batch_hit_zones_match_copy_controls() {
        let context = Rect::new(380.0, 500.0, 520.0, 126.0);
        let (page_home_x, page_home_y) = center(assign_batch_rect(context, 0));
        let (all_work_x, all_work_y) = center(assign_batch_rect(context, 3));

        assert_eq!(
            assign_batch_action_at(context, page_home_x, page_home_y),
            Some(AssignBatchAction::PageHome)
        );
        assert_eq!(
            assign_batch_action_at(context, all_work_x, all_work_y),
            Some(AssignBatchAction::AllWork)
        );
        assert_eq!(
            assign_batch_action_at(context, context.x + 18.0, context.y + 111.0),
            None
        );
    }
}
