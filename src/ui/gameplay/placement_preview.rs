use super::*;
use macroquad_toolkit::ui::draw_ui_text;

impl GameplayState {
    pub(crate) fn draw_ghost_preview(&self) {
        if let Some(building_type) = self.selected_building {
            let mouse = mouse_position_vec2();
            let mouse_x = mouse.x;
            let mouse_y = mouse.y;
            let game_area = self.layout.game_area();
            let iso = self.iso_view();
            let pos = if let Some(position) = self.capture_preview_position {
                position
            } else {
                if mouse_x < game_area.x
                    || mouse_x > game_area.x + game_area.w
                    || mouse_y < game_area.y
                    || mouse_y > game_area.y + game_area.h
                {
                    return;
                }

                iso.screen_to_grid(vec2(mouse_x, mouse_y))
            };
            let (width, height) = building_type.size();
            let feedback = PlanningSystem::building_feedback(&self.data, building_type, pos);
            let can_place = feedback.can_place();

            // Green if valid, red if invalid
            let color = if can_place {
                Color::new(0.0, 1.0, 0.0, 0.4)
            } else {
                Color::new(1.0, 0.0, 0.0, 0.4)
            };

            for dx in 0..width as i32 {
                for dy in 0..height as i32 {
                    let center = iso.grid_to_screen(Position::new(pos.x + dx, pos.y + dy));
                    draw_iso_diamond(center, iso.tile_w, iso.tile_h, color);
                }
            }

            let outline_color = if can_place { GREEN } else { RED };
            let label_pos = iso.grid_to_screen(pos);

            draw_ui_text(
                &format!(
                    "{} {}x{} | {} salvage",
                    building_type.name(),
                    width,
                    height,
                    building_type.salvage_cost()
                ),
                label_pos.x - 18.0,
                label_pos.y - 8.0,
                14.0,
                outline_color,
            );

            let panel_anchor = self
                .capture_preview_position
                .map(|_| label_pos)
                .unwrap_or_else(|| vec2(mouse_x, mouse_y));
            self.draw_placement_feedback_panel(&feedback, panel_anchor);
        }
    }

    pub(crate) fn draw_placement_feedback_panel(
        &self,
        feedback: &BuildingPlacementFeedback,
        anchor: Vec2,
    ) {
        let game_area = self.layout.game_area();
        let width = (game_area.w - 24.0).clamp(260.0, 340.0);
        let height = 124.0;
        let x = (anchor.x + 18.0)
            .min(game_area.x + game_area.w - width - 8.0)
            .max(game_area.x + 8.0);
        let y = (anchor.y + 18.0)
            .min(game_area.y + game_area.h - height - 8.0)
            .max(game_area.y + 8.0);
        let status_color = if feedback.can_place() { GREEN } else { ORANGE };

        draw_rectangle(x, y, width, height, Color::new(0.035, 0.04, 0.045, 0.94));
        draw_rectangle(x, y, 4.0, height, status_color);
        draw_rectangle_lines(x, y, width, height, 1.0, Color::new(0.45, 0.5, 0.55, 0.85));

        draw_ui_text(
            &format!(
                "{} | {}x{} | {} salvage",
                feedback.building_type.name(),
                feedback.footprint.0,
                feedback.footprint.1,
                feedback.cost
            ),
            x + 12.0,
            y + 22.0,
            14.0,
            WHITE,
        );
        draw_ui_text(
            &format!("Helps: {}", feedback.helps),
            x + 12.0,
            y + 43.0,
            12.0,
            LIGHTGRAY,
        );
        draw_ui_text(
            &truncate_text(feedback.purpose, 48),
            x + 12.0,
            y + 63.0,
            11.0,
            Color::new(0.75, 0.78, 0.8, 1.0),
        );

        if let Some(reason) = feedback.invalid_reason.as_ref() {
            draw_ui_text(
                &format!("Blocked: {}", truncate_text(reason, 39)),
                x + 12.0,
                y + 88.0,
                12.0,
                ORANGE,
            );
            draw_ui_text(
                "Move the footprint or pick another building.",
                x + 12.0,
                y + 108.0,
                11.0,
                GRAY,
            );
        } else {
            draw_ui_text(
                &format!("Impact: {}", truncate_text(feedback.impact, 42)),
                x + 12.0,
                y + 88.0,
                12.0,
                LIGHTGRAY,
            );
            draw_ui_text("Click to place this plan.", x + 12.0, y + 108.0, 11.0, GRAY);
        }
    }
}
