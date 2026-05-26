use crate::state::game_state::GameplayState;
use crate::state::{State, StateTransition};
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
        // Handle input only if strictly necessary here,
        // but UI buttons often handle this better in draw()
        // OR by using helper logic here if the toolkit allows input polling
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

        // Start Button
        // Using toolkit's button() function which serves as both draw + update
        // Since the current strict State trait has separate update/draw,
        // we might have a slight architecture clash if we strictly separate logic from rendering.
        // However, standard immediate mode (IM) often mixes them.
        // For strict separation, we'd capture the intent in draw and act in update?
        // No, standard Macroquad IM usually just acts.
        // But our `draw` signature is `&self`, preventing state mutation (transition).
        // Wait, `State::draw(&self)` is immutable.
        // If we want button clicks to trigger transitions, we need:
        // 1. `draw(&mut self)` (changes trait signature)
        // 2. OR `update()` handles UI input logic (Toolkit often needs to be in `next_frame`)
        // 3. OR we separate View data from Input data?

        // Let's look at `game_state.rs`. It does UI drawing in `draw` but input handling in `update`.
        // The standard `macroquad-toolkit` usually works best if called every frame.
        // If `draw` is const, we can't update internal button state (hover/click) if the toolkit relies on that.
        // Checking `CODE_STANDARDS.md`: "UI reads state, returns intents... UI never contains game logic".
        // It says `fn draw_button(...) -> bool`.

        // ISSUE: Our `State` trait defined in `state/mod.rs` has `fn draw(&self)`.
        // This prevents us from responding to UI events in `draw`.
        // We either need to change `State` trait to `fn draw(&mut self)` OR handle UI in `update`.
        // Given immediate mode, UI *is* logic+draw.
        // Let's assume for this MVP we can just display the menu in `draw` and handle "Start key" or "Mouse Click" in `update`.
        // OR we change the trait. Note: `game::mod.rs` has `draw(&self)`.

        // Simple Fix: Just check mouse/keyboard in `update` for this simple menu.
        // Draw just draws.
    }
}

// Re-implementing correctly to separate logic for the immutable draw trait constraint
impl MenuState {
    pub fn update_with_input(&mut self) -> StateTransition {
        let screen_center_x = screen_width() / 2.0;
        let screen_center_y = screen_height() / 2.0;

        // Simple "Start" button logic without using the toolkit's drawing widget which might require mutable state
        // Detailed UI libraries usually maintain internal state.

        // Check for Space or Enter to start
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
            return StateTransition::ToGameplay(GameplayState::new());
        }

        // Mouse click check on a "virtual" button area
        let btn_rect = Rect::new(screen_center_x - 100.0, screen_center_y, 200.0, 50.0);
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
            screen_center_y - 100.0,
            50.0,
            WHITE,
        );

        // Start Button Visuals
        let btn_rect = Rect::new(screen_center_x - 100.0, screen_center_y, 200.0, 50.0);
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
        let instr = "Press SPACE or CLICK to Start";
        let instr_dim = measure_text(instr, None, 20, 1.0);
        draw_text(
            instr,
            screen_center_x - instr_dim.width / 2.0,
            screen_center_y + 100.0,
            20.0,
            GRAY,
        );
    }
}
