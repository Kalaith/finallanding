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
    let [top, right, bottom, left] = iso_diamond_points(center, tile_w, tile_h);
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
    let [top, right, bottom, left] = iso_diamond_points(center, tile_w, tile_h);
    draw_line(top.x, top.y, right.x, right.y, thickness, color);
    draw_line(right.x, right.y, bottom.x, bottom.y, thickness, color);
    draw_line(bottom.x, bottom.y, left.x, left.y, thickness, color);
    draw_line(left.x, left.y, top.x, top.y, thickness, color);
}

pub fn iso_diamond_points(center: Vec2, tile_w: f32, tile_h: f32) -> [Vec2; 4] {
    [
        vec2(center.x, center.y),
        vec2(center.x + tile_w * 0.5, center.y + tile_h * 0.5),
        vec2(center.x, center.y + tile_h),
        vec2(center.x - tile_w * 0.5, center.y + tile_h * 0.5),
    ]
}

pub fn draw_iso_prism(
    center: Vec2,
    width: f32,
    height: f32,
    wall_height: f32,
    roof_color: Color,
    front_color: Color,
    side_color: Color,
) {
    let [_top, right, bottom, left] = iso_diamond_points(center, width, height);
    let right_drop = right + vec2(0.0, wall_height);
    let bottom_drop = bottom + vec2(0.0, wall_height);
    let left_drop = left + vec2(0.0, wall_height);

    draw_quad(right, right_drop, bottom_drop, bottom, front_color);
    draw_quad(bottom, bottom_drop, left_drop, left, side_color);
    draw_iso_diamond(center, width, height, roof_color);
    draw_iso_diamond_lines(
        center,
        width,
        height,
        1.0,
        Color::new(0.84, 0.84, 0.76, 0.55),
    );
    draw_line(
        right.x,
        right.y,
        right_drop.x,
        right_drop.y,
        1.0,
        Color::new(0.03, 0.035, 0.035, 0.8),
    );
    draw_line(
        left.x,
        left.y,
        left_drop.x,
        left_drop.y,
        1.0,
        Color::new(0.03, 0.035, 0.035, 0.8),
    );
    draw_line(
        bottom_drop.x,
        bottom_drop.y,
        left_drop.x,
        left_drop.y,
        1.0,
        Color::new(0.03, 0.035, 0.035, 0.7),
    );
}

fn draw_quad(a: Vec2, b: Vec2, c: Vec2, d: Vec2, color: Color) {
    draw_triangle(a, b, c, color);
    draw_triangle(a, c, d, color);
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

    #[test]
    fn test_iso_diamond_points_keep_expected_order() {
        let points = iso_diamond_points(vec2(10.0, 20.0), 40.0, 20.0);

        assert_eq!(points[0], vec2(10.0, 20.0));
        assert_eq!(points[1], vec2(30.0, 30.0));
        assert_eq!(points[2], vec2(10.0, 40.0));
        assert_eq!(points[3], vec2(-10.0, 30.0));
    }
}
