use crate::state::game_state::GameplayState;
use crate::state::{State, StateTransition};
use crate::ui::font::{draw_text, measure_text};
use crate::ui::{menu_start_rect, style};
use macroquad::prelude::*;
use macroquad_toolkit::input::InputState;

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
        let input = InputState::capture();
        if input.space_pressed || input.enter_pressed {
            return StateTransition::ToGameplay(GameplayState::new());
        }

        let btn_rect = menu_start_rect(screen_width(), screen_height());
        if input.left_pressed_rect(btn_rect) {
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
        style::draw_button(btn_rect, false, style::button_hovered(btn_rect));

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
