use super::*;

impl GameplayState {
    pub(crate) fn draw_grid_with_offset(&self) {
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

        draw_crash_site_context(iso, self.data.tick);

        // Highlight hovered cell
        if let Some(pos) = self.hovered_cell {
            let center = iso.grid_to_screen(pos);
            draw_iso_diamond_lines(center, iso.tile_w, iso.tile_h, 2.0, YELLOW);
        }
    }
}
