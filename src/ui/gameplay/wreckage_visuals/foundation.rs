use super::*;

pub(super) fn draw_ship_underlay(iso: IsoView) {
    draw_impact_shadow(iso);
    draw_fuselage_keel(iso);
}

pub(super) fn draw_ship_landmarks(iso: IsoView, tick: u64) {
    draw_open_deck(iso);
    draw_bow_section(iso, tick);
    draw_engine_section(iso, tick);
    draw_survivor_deck_markings(iso);
}

fn draw_impact_shadow(iso: IsoView) {
    for (cell, width, height, alpha) in [
        (Position::new(8, 7), 13.0, 5.2, 0.42),
        (Position::new(13, 11), 14.0, 5.6, 0.48),
        (Position::new(17, 16), 10.0, 4.0, 0.38),
    ] {
        let center = iso.grid_to_screen(cell) + vec2(0.0, iso.tile_h * 1.1);
        draw_iso_diamond(
            center,
            iso.tile_w * width,
            iso.tile_h * height,
            Color::new(0.0, 0.0, 0.0, alpha),
        );
    }
}

fn draw_fuselage_keel(iso: IsoView) {
    let plates = [
        (
            Position::new(7, 7),
            11.6,
            3.2,
            Color::new(0.17, 0.24, 0.25, 0.98),
        ),
        (
            Position::new(11, 10),
            12.8,
            3.6,
            Color::new(0.2, 0.28, 0.3, 0.98),
        ),
        (
            Position::new(15, 13),
            11.2,
            3.2,
            Color::new(0.17, 0.23, 0.24, 0.98),
        ),
    ];

    for (cell, width, height, color) in plates {
        let center = iso.grid_to_screen(cell) + vec2(0.0, iso.tile_h * 0.28);
        draw_iso_diamond(center, iso.tile_w * width, iso.tile_h * height, color);
        draw_iso_diamond_lines(
            center,
            iso.tile_w * width,
            iso.tile_h * height,
            2.0,
            Color::new(0.86, 0.9, 0.82, 0.46),
        );
    }

    for cell in [
        Position::new(8, 8),
        Position::new(11, 10),
        Position::new(14, 12),
        Position::new(17, 14),
    ] {
        let center = iso.grid_to_screen(cell) + vec2(0.0, iso.tile_h * 0.42);
        draw_line(
            center.x - iso.tile_w * 2.7,
            center.y,
            center.x + iso.tile_w * 2.7,
            center.y,
            3.0,
            Color::new(0.03, 0.04, 0.04, 0.82),
        );
    }
}

fn draw_open_deck(iso: IsoView) {
    let deck_center = iso.grid_to_screen(Position::new(9, 8)) + vec2(0.0, iso.tile_h * 0.1);
    draw_iso_diamond(
        deck_center + vec2(0.0, iso.tile_h * 0.9),
        iso.tile_w * 7.1,
        iso.tile_h * 3.3,
        Color::new(0.0, 0.0, 0.0, 0.34),
    );
    draw_iso_prism(
        deck_center,
        iso.tile_w * 6.5,
        iso.tile_h * 3.0,
        iso.tile_h * 0.58,
        Color::new(0.24, 0.34, 0.36, 0.98),
        Color::new(0.06, 0.085, 0.09, 0.98),
        Color::new(0.04, 0.06, 0.06, 0.98),
    );

    for offset in [-2.4_f32, -1.2, 0.0, 1.2, 2.4] {
        draw_line(
            deck_center.x + iso.tile_w * offset,
            deck_center.y + iso.tile_h * 0.72,
            deck_center.x + iso.tile_w * (offset + 0.52),
            deck_center.y + iso.tile_h * 1.2,
            2.0,
            Color::new(0.74, 0.82, 0.78, 0.44),
        );
    }

    for cell in [
        Position::new(7, 9),
        Position::new(9, 10),
        Position::new(12, 10),
    ] {
        let center = iso.grid_to_screen(cell) + vec2(0.0, iso.tile_h * 0.45);
        draw_circle(center.x, center.y, 5.0, Color::new(0.02, 0.024, 0.022, 0.9));
        draw_circle_lines(
            center.x,
            center.y,
            8.0,
            2.0,
            Color::new(0.68, 0.72, 0.66, 0.52),
        );
    }
}

fn draw_bow_section(iso: IsoView, tick: u64) {
    let nose = iso.grid_to_screen(Position::new(4, 5)) + vec2(0.0, iso.tile_h * 0.3);
    draw_triangle(
        vec2(nose.x - iso.tile_w * 4.8, nose.y + iso.tile_h * 1.72),
        vec2(nose.x + iso.tile_w * 4.2, nose.y - iso.tile_h * 0.52),
        vec2(nose.x + iso.tile_w * 2.8, nose.y + iso.tile_h * 3.44),
        Color::new(0.0, 0.0, 0.0, 0.32),
    );
    draw_triangle(
        vec2(nose.x - iso.tile_w * 4.2, nose.y + iso.tile_h * 1.45),
        vec2(nose.x + iso.tile_w * 3.7, nose.y - iso.tile_h * 0.35),
        vec2(nose.x + iso.tile_w * 2.2, nose.y + iso.tile_h * 3.1),
        Color::new(0.21, 0.31, 0.33, 0.98),
    );
    draw_triangle(
        vec2(nose.x - iso.tile_w * 3.1, nose.y + iso.tile_h * 1.45),
        vec2(nose.x + iso.tile_w * 2.1, nose.y + iso.tile_h * 0.16),
        vec2(nose.x + iso.tile_w * 1.2, nose.y + iso.tile_h * 2.45),
        Color::new(0.04, 0.055, 0.06, 0.88),
    );

    let light = if tick % 80 < 44 { 0.88 } else { 0.28 };
    for offset in [-0.7_f32, 0.0, 0.7] {
        draw_circle(
            nose.x + iso.tile_w * offset,
            nose.y + iso.tile_h * 1.22,
            3.5,
            Color::new(0.72, 0.94, 0.94, light),
        );
    }
}

fn draw_engine_section(iso: IsoView, tick: u64) {
    let base = iso.grid_to_screen(Position::new(20, 17)) + vec2(0.0, iso.tile_h * 0.52);
    for (offset, scale) in [(-1.28_f32, 1.05_f32), (0.0, 1.2), (1.28, 1.05)] {
        let center = base + vec2(iso.tile_w * offset, iso.tile_h * offset.abs() * 0.18);
        draw_circle(
            center.x,
            center.y,
            iso.tile_w * 0.76 * scale,
            Color::new(0.01, 0.014, 0.015, 0.98),
        );
        draw_circle_lines(
            center.x,
            center.y,
            iso.tile_w * 0.76 * scale,
            3.0,
            Color::new(0.7, 0.78, 0.74, 0.78),
        );
        draw_circle(
            center.x,
            center.y,
            iso.tile_w * 0.34 * scale,
            Color::new(0.92, 0.42, 0.12, if tick % 90 < 42 { 0.68 } else { 0.36 }),
        );
    }
}

fn draw_survivor_deck_markings(iso: IsoView) {
    let camp_center = iso.grid_to_screen(Position::new(10, 9)) + vec2(0.0, iso.tile_h * 0.74);
    draw_iso_diamond_lines(
        camp_center,
        iso.tile_w * 4.4,
        iso.tile_h * 1.8,
        2.0,
        Color::new(0.95, 0.62, 0.22, 0.72),
    );

    for offset in [-1.6_f32, -0.8, 0.0, 0.8, 1.6] {
        draw_line(
            camp_center.x + iso.tile_w * offset,
            camp_center.y + iso.tile_h * 0.82,
            camp_center.x + iso.tile_w * (offset + 0.38),
            camp_center.y + iso.tile_h * 1.06,
            2.0,
            Color::new(0.95, 0.62, 0.22, 0.7),
        );
    }
}
