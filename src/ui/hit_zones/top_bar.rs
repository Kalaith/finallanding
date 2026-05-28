use crate::data::game_state::TimeSpeed;
use crate::data::priority::ColonyPriority;
use macroquad::prelude::{vec2, Rect};
use macroquad_toolkit::input::{hit_test, HitTarget};

pub const TOP_BAR_BUTTON_Y: f32 = 10.0;
pub const TOP_BAR_BUTTON_H: f32 = 30.0;
pub const SPEED_BUTTON_W: f32 = 50.0;
pub const SPEED_BUTTON_START_X: f32 = 300.0;
pub const PRIORITY_LABEL_X: f32 = 850.0;
pub const PRIORITY_BUTTON_W: f32 = 68.0;
pub const PRIORITY_BUTTON_START_X: f32 = 915.0;
pub const BUTTON_GAP: f32 = 5.0;

pub fn speed_button_rect(index: usize) -> Rect {
    Rect::new(
        SPEED_BUTTON_START_X + index as f32 * (SPEED_BUTTON_W + BUTTON_GAP),
        TOP_BAR_BUTTON_Y,
        SPEED_BUTTON_W,
        TOP_BAR_BUTTON_H,
    )
}

pub fn priority_button_rect(index: usize) -> Rect {
    Rect::new(
        PRIORITY_BUTTON_START_X + index as f32 * (PRIORITY_BUTTON_W + BUTTON_GAP),
        TOP_BAR_BUTTON_Y,
        PRIORITY_BUTTON_W,
        TOP_BAR_BUTTON_H,
    )
}

pub fn top_bar_speed_at(x: f32, y: f32) -> Option<TimeSpeed> {
    let speeds = [
        TimeSpeed::Paused,
        TimeSpeed::Normal,
        TimeSpeed::Fast,
        TimeSpeed::SuperFast,
    ];
    hit_test(
        speeds
            .into_iter()
            .enumerate()
            .map(|(index, speed)| HitTarget::new(speed_button_rect(index), speed)),
        vec2(x, y),
    )
}

pub fn top_bar_priority_at(x: f32, y: f32) -> Option<ColonyPriority> {
    hit_test(
        ColonyPriority::all()
            .iter()
            .enumerate()
            .map(|(index, priority)| HitTarget::new(priority_button_rect(index), *priority)),
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
    fn test_top_bar_speed_hit_zones_match_visible_buttons() {
        let (x, y) = center(speed_button_rect(1));

        assert_eq!(top_bar_speed_at(x, y), Some(TimeSpeed::Normal));
        assert_eq!(top_bar_speed_at(10.0, y), None);
    }

    #[test]
    fn test_top_bar_priority_hit_zones_match_visible_buttons() {
        let (x, y) = center(priority_button_rect(2));

        assert_eq!(top_bar_priority_at(x, y), Some(ColonyPriority::Survey));
    }
}
