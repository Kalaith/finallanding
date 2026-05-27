use crate::state::game_state::GameplayState;
use crate::state::{State, StateTransition};
use crate::ui::menu_start_rect;
use macroquad::prelude::*;

pub struct MenuState {
    // Menu specific data could go here (e.g. animation timers)
}

impl MenuState {
    pub fn new() -> Self {
        Self {}
    }
}

impl State for MenuState {
    fn update(&mut self) -> StateTransition {
        StateTransition::None
    }

    fn draw(&self) {
        clear_background(macroquad_toolkit::colors::dark::BACKGROUND);

        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;

        // Title
        let title = "The Final Landing";
        let title_dim = measure_text(title, None, 50, 1.0);
        draw_text(
            title,
            screen_center_x - title_dim.width / 2.0,
            screen_center_y - 100.0,
            50.0,
            WHITE,
        );
    }
}

// Re-implementing correctly to separate logic for the immutable draw trait constraint
impl MenuState {
    pub fn update_with_input(&mut self) -> StateTransition {
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
            return StateTransition::ToGameplay(GameplayState::new());
        }

        let btn_rect = menu_start_rect(screen_width(), screen_height());
        let mouse_pos = mouse_position().into();

        if is_mouse_button_pressed(MouseButton::Left) && btn_rect.contains(mouse_pos) {
            return StateTransition::ToGameplay(GameplayState::new());
        }

        StateTransition::None
    }

    pub fn draw_ui(&self) {
        clear_background(macroquad_toolkit::colors::dark::BACKGROUND);

        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;

        // Title
        let title = "The Final Landing";
        let title_dim = measure_text(title, None, 50, 1.0);
        draw_text(
            title,
            screen_center_x - title_dim.width / 2.0,
            screen_center_y - 170.0,
            50.0,
            WHITE,
        );

        let premise = [
            "Guide six crash survivors through the first week.",
            "Build shelter, food, storage, repairs, and scouting routes.",
            "Priorities shape work pressure, recovery, missions, and relationships.",
        ];
        for (index, line) in premise.iter().enumerate() {
            let dim = measure_text(line, None, 20, 1.0);
            draw_text(
                line,
                screen_center_x - dim.width / 2.0,
                screen_center_y - 112.0 + index as f32 * 28.0,
                20.0,
                LIGHTGRAY,
            );
        }

        // Start Button Visuals
        let btn_rect = menu_start_rect(screen_width(), screen_height());
        let mouse_pos: Vec2 = mouse_position().into();
        let is_hovered = btn_rect.contains(mouse_pos);

        let color = if is_hovered {
            macroquad_toolkit::colors::dark::ACCENT
        } else {
            macroquad_toolkit::colors::dark::PANEL
        };

        draw_rectangle(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, color);

        let btn_text = "Start Game";
        let btn_dim = measure_text(btn_text, None, 30, 1.0);
        draw_text(
            btn_text,
            btn_rect.x + (btn_rect.w - btn_dim.width) / 2.0,
            btn_rect.y + 35.0,
            30.0,
            WHITE,
        );

        // Instructions
        let controls = [
            "Mouse places buildings and uses panels",
            "Q W E R T choose building tools | Z undo | Esc cancel",
            "1 2 3 set priority | Space pause | M launch recommended mission",
        ];
        for (index, line) in controls.iter().enumerate() {
            let dim = measure_text(line, None, 16, 1.0);
            draw_text(
                line,
                screen_center_x - dim.width / 2.0,
                screen_center_y + 104.0 + index as f32 * 22.0,
                16.0,
                GRAY,
            );
        }
    }
}
