use super::*;

impl GameplayState {
    pub(crate) fn draw_colonists_with_offset(&self, hovered_colonist_id: Option<u32>) {
        let iso = self.iso_view();

        self.draw_social_links(hovered_colonist_id);

        for colonist in &self.data.colonists {
            if colonist.is_on_mission() {
                continue;
            }

            let foot = iso.grid_to_screen(colonist.position);
            let x = foot.x - 16.0;
            let y = foot.y - 28.0;
            let size = 24.0;

            // Colonist color based on state
            let color = match colonist.state {
                ColonistState::Idle => SKYBLUE,
                ColonistState::Moving { .. } => GREEN,
                ColonistState::Working => ORANGE,
                ColonistState::Eating => YELLOW,
                ColonistState::Sleeping => Color::new(0.5, 0.5, 0.8, 1.0),
                ColonistState::OnMission { .. } => PURPLE,
            };

            let center_x = x + 16.0;
            let center_y = y + 16.0;
            draw_ellipse(
                center_x,
                center_y + 12.0,
                12.0,
                4.0,
                0.0,
                Color::new(0.0, 0.0, 0.0, 0.25),
            );
            let social_signal = self.social_body_language_for(colonist);
            if let Some(sprite) = self.art.colonist_sprite_for_pose(
                colonist.id,
                sprite_pose_for_colonist_frame(colonist, social_signal, self.data.tick),
            ) {
                draw_texture_ex(
                    sprite,
                    center_x - 18.0,
                    center_y - 37.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(36.0, 70.0)),
                        ..Default::default()
                    },
                );
            } else {
                draw_rectangle(center_x - 8.0, center_y + 2.0, 16.0, 15.0, color);
                draw_rectangle_lines(center_x - 8.0, center_y + 2.0, 16.0, 15.0, 1.0, WHITE);
                draw_circle(
                    center_x,
                    center_y - 5.0,
                    8.0,
                    Color::new(0.78, 0.68, 0.56, 1.0),
                );
                draw_circle_lines(center_x, center_y - 5.0, 8.0, 1.0, WHITE);
                draw_rectangle(center_x - 5.0, center_y - 10.0, 10.0, 3.0, color);
                draw_line(
                    center_x - 5.0,
                    center_y + 17.0,
                    center_x - 9.0,
                    center_y + 24.0,
                    2.0,
                    LIGHTGRAY,
                );
                draw_line(
                    center_x + 5.0,
                    center_y + 17.0,
                    center_x + 9.0,
                    center_y + 24.0,
                    2.0,
                    LIGHTGRAY,
                );
            }
            draw_circle(
                center_x + 8.0,
                center_y + 5.0,
                3.0,
                job_color(colonist.job_preference),
            );
            if let Some(value) = strongest_relationship_value(colonist) {
                if value.abs() >= 20 {
                    let color = social_color(value, 0.95);
                    draw_circle(center_x - 10.0, center_y - 22.0, 5.0, color);
                    draw_circle_lines(center_x - 10.0, center_y - 22.0, 5.0, 1.0, BLACK);
                    draw_text(
                        if value > 0 { "+" } else { "-" },
                        center_x - 13.0,
                        center_y - 18.0,
                        9.0,
                        style::TEXT_PRIMARY,
                    );
                }
            }
            if let Some(signal) = social_signal {
                let pulse = ((self.data.tick % 90) as f32 / 90.0 * std::f32::consts::TAU)
                    .sin()
                    .abs();
                let signal_color = signal.color(0.46 + pulse * 0.22);
                draw_circle_lines(
                    center_x,
                    center_y - 12.0,
                    15.0 + pulse * 3.0,
                    2.0,
                    signal_color,
                );
                draw_text(
                    signal.symbol(),
                    center_x + 8.0,
                    center_y - 25.0,
                    12.0,
                    signal.color(1.0),
                );
            }
            let selected = Some(colonist.id) == self.selected_colonist_id;
            let hovered = Some(colonist.id) == hovered_colonist_id;
            if selected || hovered {
                let outline_color = if selected {
                    style::ACCENT_GOLD
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.86)
                };
                draw_circle_lines(center_x, center_y, size / 2.0 + 6.0, 3.0, outline_color);
                draw_circle_lines(
                    center_x,
                    center_y,
                    size / 2.0 + 10.0,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.62),
                );

                let name_width = measure_text(&colonist.name, None, 12, 1.0).width;
                draw_rectangle(
                    center_x - name_width * 0.5 - 5.0,
                    y + 28.0,
                    name_width + 10.0,
                    16.0,
                    Color::new(0.03, 0.04, 0.04, 0.76),
                );
                draw_text(
                    &colonist.name,
                    center_x - name_width / 2.0,
                    y + 40.0,
                    12.0,
                    WHITE,
                );
            }
        }
    }

    pub(crate) fn draw_hover_colonist_card(&self, hovered_colonist_id: Option<u32>) {
        let Some(colonist) = hovered_colonist_id.and_then(|id| self.colonist_by_id(id)) else {
            return;
        };

        let mouse = mouse_position_vec2();
        draw_tooltip_at(
            mouse + vec2(18.0, 18.0),
            self.layout.game_area(),
            &colonist.name,
            &format!(
                "{} | Mood {:.0} | {}",
                colonist.job_preference.label(),
                colonist.mood,
                colonist_activity_summary(colonist)
            ),
        );
    }
}
