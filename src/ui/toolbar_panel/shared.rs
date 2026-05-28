use crate::systems::assignment_system::AssignmentPressure;
use crate::systems::relationship_directive_system::PairDirective;
use crate::ui::style;
use macroquad::prelude::{Color, Rect};

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
