use super::*;

impl GameplayState {
    pub(super) fn draw_scenario_overlay(&self) {
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
        let title_width = measure_text(title, None, 28, 1.0).width;
        draw_text(title, x + (w - title_width) * 0.5, y + 42.0, 28.0, WHITE);

        let line = ScenarioSystem::objective_line(&self.data);
        let line_width = measure_text(&line, None, 16, 1.0).width;
        draw_text(&line, x + (w - line_width) * 0.5, y + 82.0, 16.0, LIGHTGRAY);

        let prompt = "Scenario complete. Review the log, then restart for another plan.";
        let prompt_width = measure_text(prompt, None, 14, 1.0).width;
        draw_text(prompt, x + (w - prompt_width) * 0.5, y + 116.0, 14.0, GRAY);

        let button = restart_button_rect(screen_width(), screen_height());
        let button_color = if style::button_hovered(button) {
            Color::new(0.25, 0.38, 0.48, 1.0)
        } else {
            Color::new(0.16, 0.22, 0.28, 1.0)
        };
        draw_rectangle(button.x, button.y, button.w, button.h, button_color);
        draw_rectangle_lines(button.x, button.y, button.w, button.h, 1.0, WHITE);
        let button_text = "Restart Run";
        let button_width = measure_text(button_text, None, 18, 1.0).width;
        draw_text(
            button_text,
            button.x + (button.w - button_width) * 0.5,
            button.y + 25.0,
            18.0,
            WHITE,
        );
        let restart_hint = "R or Enter";
        let hint_width = measure_text(restart_hint, None, 12, 1.0).width;
        draw_text(
            restart_hint,
            x + (w - hint_width) * 0.5,
            y + 170.0,
            12.0,
            LIGHTGRAY,
        );
    }

    /// Draw buildings on the grid
    pub(super) fn draw_buildings(&self) {
        let iso = self.iso_view();
        let hovered_building_id = self.building_at_mouse().map(|building| building.id);
        for building in self.data.building_system.buildings() {
            let (width, height) = building.size();
            let (r, g, b) = building.building_type.color();
            let color = Color::new(
                r as f32 / 255.0 * 0.72,
                g as f32 / 255.0 * 0.72,
                b as f32 / 255.0 * 0.72,
                1.0,
            );

            for cell in building.occupied_cells() {
                let center = iso.grid_to_screen(cell);
                draw_iso_diamond(center, iso.tile_w, iso.tile_h, color);
                draw_iso_diamond_lines(
                    center,
                    iso.tile_w,
                    iso.tile_h,
                    1.0,
                    Color::new(0.82, 0.82, 0.76, 0.55),
                );
            }

            let filter_match = self.toolbar_mode == ToolbarMode::Assign
                && self.assign_building_filter == Some(building.id);
            let assignment_marker = assignment_marker_with_filter(
                self.assignment_marker_for_building(building.id),
                filter_match,
            );
            let outline_style = building_outline_style_for_assign_filter(
                hovered_building_id == Some(building.id),
                assignment_marker.map(|(_, color)| color),
                filter_match,
            );
            self.draw_building_shell(
                building.building_type,
                building.position,
                width,
                height,
                &iso,
                outline_style,
            );
            if let Some((outline_color, thickness)) = outline_style {
                self.draw_building_footprint_outline(building, &iso, outline_color, thickness);
            }

            let name = building.building_type.name();
            let label_pos = iso.grid_to_screen(Position::new(
                building.position.x + width as i32 / 2,
                building.position.y + height as i32 / 2,
            ));
            if let Some((assignment_label, assignment_color)) = assignment_marker {
                let marker_width = measure_text(assignment_label, None, 10, 1.0).width + 10.0;
                draw_rectangle(
                    label_pos.x - marker_width * 0.5,
                    label_pos.y - 29.0,
                    marker_width,
                    14.0,
                    Color::new(0.03, 0.04, 0.04, 0.82),
                );
                draw_rectangle_lines(
                    label_pos.x - marker_width * 0.5,
                    label_pos.y - 29.0,
                    marker_width,
                    14.0,
                    1.0,
                    assignment_color,
                );
                draw_text(
                    assignment_label,
                    label_pos.x - marker_width * 0.5 + 5.0,
                    label_pos.y - 18.0,
                    10.0,
                    assignment_color,
                );
            }
            let label_width = measure_text(name, None, 12, 1.0).width;
            draw_text(
                name,
                label_pos.x - label_width * 0.5,
                label_pos.y - 8.0,
                12.0,
                WHITE,
            );
        }
    }

    pub(super) fn draw_building_footprint_outline(
        &self,
        building: &Building,
        iso: &IsoView,
        color: Color,
        thickness: f32,
    ) {
        for cell in building.occupied_cells() {
            let center = iso.grid_to_screen(cell);
            draw_iso_diamond_lines(center, iso.tile_w, iso.tile_h, thickness, color);
        }
    }

    pub(super) fn assignment_marker_for_building(
        &self,
        building_id: u32,
    ) -> Option<(&'static str, Color)> {
        if self.toolbar_mode != ToolbarMode::Assign {
            return None;
        }

        let colonist = self
            .selected_colonist_id
            .and_then(|id| self.colonist_by_id(id))?;

        if colonist.assigned_habitat == Some(building_id) {
            Some(("HOME", style::BAR_GREEN))
        } else if colonist.assigned_workplace == Some(building_id) {
            Some(("WORK", style::HEADING_BLUE))
        } else {
            None
        }
    }

    pub(super) fn draw_building_shell(
        &self,
        building_type: BuildingType,
        position: Position,
        width: u32,
        height: u32,
        iso: &IsoView,
        outline_style: Option<(Color, f32)>,
    ) {
        let center = iso.grid_to_screen(Position::new(
            position.x + width as i32 / 2,
            position.y + height as i32 / 2,
        ));
        let shell_width = iso.tile_w * width as f32 * 0.86;
        let shell_height = iso.tile_h * height as f32 * 0.86;
        let wall_height = building_wall_height(building_type, iso.tile_h);
        let roof_center = center - vec2(0.0, wall_height + iso.tile_h * 0.12);
        let (roof, front, side) = building_shell_colors(building_type);

        draw_iso_prism(
            roof_center,
            shell_width,
            shell_height,
            wall_height,
            roof,
            front,
            side,
        );
        draw_building_shell_detail(building_type, roof_center, shell_width, shell_height);
        if let Some((outline_color, thickness)) = outline_style {
            draw_iso_diamond_lines(
                roof_center,
                shell_width + 4.0,
                shell_height + 4.0,
                thickness,
                outline_color,
            );
        }
    }

    /// Draw ghost preview of building at cursor
    pub(super) fn draw_ghost_preview(&self) {
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

            draw_text(
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

    pub(super) fn draw_placement_feedback_panel(
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

        draw_text(
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
        draw_text(
            &format!("Helps: {}", feedback.helps),
            x + 12.0,
            y + 43.0,
            12.0,
            LIGHTGRAY,
        );
        draw_text(
            &truncate_text(feedback.purpose, 48),
            x + 12.0,
            y + 63.0,
            11.0,
            Color::new(0.75, 0.78, 0.8, 1.0),
        );

        if let Some(reason) = feedback.invalid_reason.as_ref() {
            draw_text(
                &format!("Blocked: {}", truncate_text(reason, 39)),
                x + 12.0,
                y + 88.0,
                12.0,
                ORANGE,
            );
            draw_text(
                "Move the footprint or pick another building.",
                x + 12.0,
                y + 108.0,
                11.0,
                GRAY,
            );
        } else {
            draw_text(
                &format!("Impact: {}", truncate_text(feedback.impact, 42)),
                x + 12.0,
                y + 88.0,
                12.0,
                LIGHTGRAY,
            );
            draw_text("Click to place this plan.", x + 12.0, y + 108.0, 11.0, GRAY);
        }
    }

    /// Draw the grid with offset for top bar
    pub(super) fn draw_grid_with_offset(&self) {
        let iso = self.iso_view();

        for y in 0..self.data.grid.height {
            for x in 0..self.data.grid.width {
                let cell_type = self
                    .data
                    .grid
                    .get_cell(x as i32, y as i32)
                    .map(|cell| cell.cell_type);
                let color = terrain_color(cell_type, x as i32, y as i32);

                let center = iso.grid_to_screen(Position::new(x as i32, y as i32));
                draw_iso_diamond(center, iso.tile_w, iso.tile_h, color);
                draw_terrain_detail(
                    center,
                    iso.tile_w,
                    iso.tile_h,
                    terrain_detail(cell_type, x as i32, y as i32),
                );
                draw_iso_diamond_lines(
                    center,
                    iso.tile_w,
                    iso.tile_h,
                    1.0,
                    Color::new(0.12, 0.13, 0.11, 0.45),
                );
            }
        }

        // Highlight hovered cell
        if let Some(pos) = self.hovered_cell {
            let center = iso.grid_to_screen(pos);
            draw_iso_diamond_lines(center, iso.tile_w, iso.tile_h, 2.0, YELLOW);
        }
    }

    /// Draw colonists with offset for top bar
    pub(super) fn draw_colonists_with_offset(&self, hovered_colonist_id: Option<u32>) {
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

    pub(super) fn draw_social_links(&self, hovered_colonist_id: Option<u32>) {
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

    pub(super) fn social_body_language_for(
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

    pub(super) fn draw_hover_colonist_card(&self, hovered_colonist_id: Option<u32>) {
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

    pub(super) fn colonist_id_at_mouse(&self) -> Option<u32> {
        let game_area = self.layout.game_area();
        let mouse = mouse_position_vec2();
        let mouse_x = mouse.x;
        let mouse_y = mouse.y;
        if mouse_x < game_area.x
            || mouse_x > game_area.x + game_area.w
            || mouse_y < game_area.y
            || mouse_y > game_area.y + game_area.h
        {
            return None;
        }

        self.data
            .colonists
            .iter()
            .filter(|colonist| !colonist.is_on_mission())
            .filter_map(|colonist| {
                let foot = self.iso_view().grid_to_screen(colonist.position);
                let center_x = foot.x;
                let center_y = foot.y - 8.0;
                let dx = mouse_x - center_x;
                let dy = mouse_y - center_y;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= 18.0 * 18.0 {
                    Some((colonist.id, distance_sq))
                } else {
                    None
                }
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(id, _)| id)
    }

    pub(super) fn building_at_mouse(&self) -> Option<&Building> {
        let game_area = self.layout.game_area();
        let mouse = mouse_position_vec2();
        if !game_area.contains(mouse) {
            return None;
        }

        let grid_pos = self.iso_view().screen_to_grid(mouse);
        self.data.building_system.get_building_at(grid_pos)
    }

    pub(super) fn colonist_by_id(&self, id: u32) -> Option<&Colonist> {
        self.data
            .colonists
            .iter()
            .find(|colonist| colonist.id == id)
    }

    pub(super) fn inspected_colonist(&self, hovered_colonist_id: Option<u32>) -> Option<&Colonist> {
        hovered_colonist_id
            .and_then(|id| self.colonist_by_id(id))
            .or_else(|| {
                self.selected_colonist_id
                    .and_then(|id| self.colonist_by_id(id))
            })
    }
}
