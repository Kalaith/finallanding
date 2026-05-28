use super::*;

impl GameplayState {
    pub(crate) fn draw_buildings(&self) {
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

    pub(crate) fn draw_building_footprint_outline(
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

    pub(crate) fn assignment_marker_for_building(
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

    pub(crate) fn draw_building_shell(
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
}
