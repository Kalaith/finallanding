use macroquad::prelude::Rect;

pub fn menu_start_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(screen_width * 0.5 - 100.0, screen_height * 0.5, 200.0, 50.0)
}

pub fn restart_button_rect(screen_width: f32, screen_height: f32) -> Rect {
    Rect::new(
        screen_width * 0.5 - 90.0,
        screen_height * 0.5 + 48.0,
        180.0,
        38.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::vec2;
    use macroquad_toolkit::input::rect_contains_point;

    #[test]
    fn test_menu_start_rect_contains_button_center() {
        let rect = menu_start_rect(1280.0, 720.0);

        assert!(rect_contains_point(rect, vec2(640.0, 385.0)));
    }

    #[test]
    fn test_restart_rect_contains_button_center() {
        let rect = restart_button_rect(1280.0, 720.0);

        assert!(rect_contains_point(rect, vec2(640.0, 427.0)));
    }
}
