use super::*;

impl GameplayState {
    pub(crate) fn draw_social_links(&self, hovered_colonist_id: Option<u32>) {
        let focus_id = hovered_colonist_id.or(self.selected_colonist_id);
        let iso = self.iso_view();

        for first_index in 0..self.data.colonists.len() {
            let first = &self.data.colonists[first_index];
            if first.is_on_mission() {
                continue;
            }

            for second in self.data.colonists.iter().skip(first_index + 1) {
                if second.is_on_mission() {
                    continue;
                }

                let value = average_relationship_between(first, second);
                let focused_pair = focus_id.is_some_and(|id| id == first.id || id == second.id);
                let shared_location = shared_social_location(first, second);
                let strong_pair = value.abs() >= 25;

                if !(strong_pair || shared_location || focused_pair) || value.abs() < 10 {
                    continue;
                }

                let first_anchor = iso.grid_to_screen(first.position) + vec2(0.0, -28.0);
                let second_anchor = iso.grid_to_screen(second.position) + vec2(0.0, -28.0);
                let color = social_color(
                    value,
                    if focused_pair || shared_location {
                        0.72
                    } else {
                        0.34
                    },
                );

                draw_line(
                    first_anchor.x,
                    first_anchor.y,
                    second_anchor.x,
                    second_anchor.y,
                    if focused_pair || shared_location {
                        2.0
                    } else {
                        1.0
                    },
                    color,
                );

                if focused_pair || (shared_location && value.abs() >= 20) {
                    let mid = (first_anchor + second_anchor) * 0.5;
                    let label = format!("{:+}", value);
                    let width = measure_text(&label, None, 10, 1.0).width;
                    draw_rectangle(
                        mid.x - width * 0.5 - 4.0,
                        mid.y - 11.0,
                        width + 8.0,
                        14.0,
                        Color::new(0.03, 0.04, 0.04, 0.78),
                    );
                    draw_text(
                        &label,
                        mid.x - width * 0.5,
                        mid.y,
                        10.0,
                        social_color(value, 1.0),
                    );
                }
            }
        }
    }

    pub(crate) fn social_body_language_for(
        &self,
        colonist: &Colonist,
    ) -> Option<SocialBodyLanguage> {
        if matches!(
            colonist.state,
            ColonistState::Moving { .. }
                | ColonistState::Sleeping
                | ColonistState::OnMission { .. }
        ) {
            return None;
        }

        let mut best_signal = None;
        for other in &self.data.colonists {
            if other.id == colonist.id || other.is_on_mission() {
                continue;
            }

            let value = average_relationship_between(colonist, other);
            if value.abs() < 20 {
                continue;
            }

            let active_contact = shared_social_location(colonist, other)
                || shared_assignment_pin(colonist, other)
                || adjacent_positions(colonist.position, other.position);
            if !active_contact {
                continue;
            }

            let signal = if value < 0 {
                SocialBodyLanguage::Tense(value)
            } else {
                SocialBodyLanguage::Supported(value)
            };
            if best_signal
                .map(|best: SocialBodyLanguage| signal.intensity() > best.intensity())
                .unwrap_or(true)
            {
                best_signal = Some(signal);
            }
        }

        best_signal
    }
}
