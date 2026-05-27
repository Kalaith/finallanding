use crate::data::types::Position;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct IsoView {
    pub origin: Vec2,
    pub tile_w: f32,
    pub tile_h: f32,
}

impl IsoView {
    pub fn for_area(area: Rect, grid_width: u32, grid_height: u32) -> Self {
        let tile_w = (area.w / ((grid_width + grid_height) as f32 * 0.52)).clamp(28.0, 52.0);
        let tile_h = tile_w * 0.5;
        let map_h = (grid_width + grid_height) as f32 * tile_h * 0.5;
        Self {
            origin: vec2(
                area.x + area.w * 0.5,
                area.y + (area.h - map_h) * 0.28 + 18.0,
            ),
            tile_w,
            tile_h,
        }
    }

    pub fn grid_to_screen(self, position: Position) -> Vec2 {
        let x = position.x as f32;
        let y = position.y as f32;
        vec2(
            self.origin.x + (x - y) * self.tile_w * 0.5,
            self.origin.y + (x + y) * self.tile_h * 0.5,
        )
    }

    pub fn screen_to_grid(self, point: Vec2) -> Position {
        let dx = (point.x - self.origin.x) / (self.tile_w * 0.5);
        let dy = (point.y - self.origin.y) / (self.tile_h * 0.5);
        Position::new(
            ((dy + dx) * 0.5).floor() as i32,
            ((dy - dx) * 0.5).floor() as i32,
        )
    }
}

pub fn draw_iso_diamond(center: Vec2, tile_w: f32, tile_h: f32, color: Color) {
    let top = vec2(center.x, center.y);
    let right = vec2(center.x + tile_w * 0.5, center.y + tile_h * 0.5);
    let bottom = vec2(center.x, center.y + tile_h);
    let left = vec2(center.x - tile_w * 0.5, center.y + tile_h * 0.5);
    draw_triangle(top, right, bottom, color);
    draw_triangle(top, bottom, left, color);
}

pub fn draw_iso_diamond_lines(
    center: Vec2,
    tile_w: f32,
    tile_h: f32,
    thickness: f32,
    color: Color,
) {
    let top = vec2(center.x, center.y);
    let right = vec2(center.x + tile_w * 0.5, center.y + tile_h * 0.5);
    let bottom = vec2(center.x, center.y + tile_h);
    let left = vec2(center.x - tile_w * 0.5, center.y + tile_h * 0.5);
    draw_line(top.x, top.y, right.x, right.y, thickness, color);
    draw_line(right.x, right.y, bottom.x, bottom.y, thickness, color);
    draw_line(bottom.x, bottom.y, left.x, left.y, thickness, color);
    draw_line(left.x, left.y, top.x, top.y, thickness, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_projection_round_trips_grid_cell() {
        let view = IsoView::for_area(Rect::new(300.0, 66.0, 680.0, 568.0), 20, 20);
        let position = Position::new(6, 9);
        let screen = view.grid_to_screen(position);

        assert_eq!(
            view.screen_to_grid(screen + vec2(0.0, view.tile_h * 0.25)),
            position
        );
    }
}
