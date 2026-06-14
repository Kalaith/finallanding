use super::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl GameplayState {
    pub(crate) fn draw_scenario_overlay(&self) {
        if !self.data.scenario.is_finished() {
            return;
        }

        let w = 520.0;
        let h = 190.0;
        let x = (screen_width() - w) * 0.5;
        let y = (screen_height() - h) * 0.5;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.55),
        );
        draw_rectangle(x, y, w, h, Color::new(0.08, 0.08, 0.1, 0.95));
        draw_rectangle_lines(x, y, w, h, 2.0, WHITE);

        let title = self.data.scenario.outcome.label();
        let title_width = measure_ui_text(title, None, 28, 1.0).width;
        draw_ui_text(title, x + (w - title_width) * 0.5, y + 42.0, 28.0, WHITE);

        let line = ScenarioSystem::objective_line(&self.data);
        let line_width = measure_ui_text(&line, None, 16, 1.0).width;
        draw_ui_text(&line, x + (w - line_width) * 0.5, y + 82.0, 16.0, LIGHTGRAY);

        let prompt = "Scenario complete. Review the log, then restart for another plan.";
        let prompt_width = measure_ui_text(prompt, None, 14, 1.0).width;
        draw_ui_text(prompt, x + (w - prompt_width) * 0.5, y + 116.0, 14.0, GRAY);

        let button = restart_button_rect(screen_width(), screen_height());
        let button_color = if style::button_hovered(button) {
            Color::new(0.25, 0.38, 0.48, 1.0)
        } else {
            Color::new(0.16, 0.22, 0.28, 1.0)
        };
        draw_rectangle(button.x, button.y, button.w, button.h, button_color);
        draw_rectangle_lines(button.x, button.y, button.w, button.h, 1.0, WHITE);
        let button_text = "Restart Run";
        let button_width = measure_ui_text(button_text, None, 18, 1.0).width;
        draw_ui_text(
            button_text,
            button.x + (button.w - button_width) * 0.5,
            button.y + 25.0,
            18.0,
            WHITE,
        );
        let restart_hint = "R or Enter";
        let hint_width = measure_ui_text(restart_hint, None, 12, 1.0).width;
        draw_ui_text(
            restart_hint,
            x + (w - hint_width) * 0.5,
            y + 170.0,
            12.0,
            LIGHTGRAY,
        );
    }
}
