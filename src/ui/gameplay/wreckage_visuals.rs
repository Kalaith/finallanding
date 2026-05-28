use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CrashSmokeVent {
    pub(crate) cell: Position,
    pub(crate) strength: f32,
    pub(crate) seed: u64,
}

const SMOKE_VENTS: [CrashSmokeVent; 5] = [
    CrashSmokeVent {
        cell: Position { x: 8, y: 5 },
        strength: 1.35,
        seed: 13,
    },
    CrashSmokeVent {
        cell: Position { x: 13, y: 8 },
        strength: 1.0,
        seed: 41,
    },
    CrashSmokeVent {
        cell: Position { x: 18, y: 10 },
        strength: 1.35,
        seed: 79,
    },
    CrashSmokeVent {
        cell: Position { x: 10, y: 13 },
        strength: 0.9,
        seed: 107,
    },
    CrashSmokeVent {
        cell: Position { x: 20, y: 15 },
        strength: 1.1,
        seed: 151,
    },
];

pub(crate) fn draw_crash_site_context(iso: IsoView, tick: u64) {
    draw_crash_scar(iso);
    draw_hull_section(iso, Position::new(2, 2), 12.0, 6.0, HullMood::Main);
    draw_hull_section(iso, Position::new(12, 7), 11.0, 7.0, HullMood::Aft);
    draw_hull_section(iso, Position::new(15, 15), 8.0, 4.0, HullMood::Buried);
    draw_broken_spine(iso);
    draw_survival_camp(iso);
    draw_scattered_salvage(iso);
    draw_exposed_wiring(iso, tick);
    draw_flickering_lights(iso, tick);
    draw_smoke_vents(iso, tick);
}

pub(crate) fn crash_scorch_intensity(x: i32, y: i32) -> f32 {
    let anchors = [
        (8.0_f32, 5.0_f32, 7.0_f32),
        (15.0, 8.0, 7.5),
        (18.0, 14.0, 5.0),
    ];
    anchors
        .iter()
        .map(|(ax, ay, radius)| {
            let dx = x as f32 - ax;
            let dy = y as f32 - ay;
            let distance = (dx * dx + dy * dy).sqrt();
            (1.0 - distance / radius).clamp(0.0, 1.0)
        })
        .fold(0.0, f32::max)
}

pub(crate) fn flicker_alpha(tick: u64, seed: u64) -> f32 {
    let phase = ((tick.wrapping_add(seed) / 9) % 5) as f32;
    0.42 + phase * 0.12
}

fn draw_crash_scar(iso: IsoView) {
    for y in 2..20 {
        for x in 2..24 {
            let intensity = crash_scorch_intensity(x, y);
            if intensity <= 0.08 {
                continue;
            }
            let center = iso.grid_to_screen(Position::new(x, y));
            draw_iso_diamond(
                center + vec2(0.0, iso.tile_h * 0.08),
                iso.tile_w * (0.85 + intensity * 0.8),
                iso.tile_h * (0.54 + intensity * 0.68),
                Color::new(0.018, 0.016, 0.012, 0.34 + intensity * 0.38),
            );
            if intensity > 0.62 && (x + y) % 3 == 0 {
                draw_iso_diamond(
                    center + vec2(0.0, iso.tile_h * 0.14),
                    iso.tile_w * 0.72,
                    iso.tile_h * 0.28,
                    Color::new(0.58, 0.2, 0.07, 0.22),
                );
            }
        }
    }
}

#[derive(Clone, Copy)]
enum HullMood {
    Main,
    Aft,
    Buried,
}

fn draw_hull_section(iso: IsoView, origin: Position, width: f32, height: f32, mood: HullMood) {
    let center = iso.grid_to_screen(Position::new(
        origin.x + (width * 0.5) as i32,
        origin.y + (height * 0.5) as i32,
    ));
    let plate_w = iso.tile_w * width * 1.02;
    let plate_h = iso.tile_h * height * 1.24;
    let wall_h = match mood {
        HullMood::Main => iso.tile_h * 2.35,
        HullMood::Aft => iso.tile_h * 1.95,
        HullMood::Buried => iso.tile_h * 1.05,
    };
    let roof = match mood {
        HullMood::Main => Color::new(0.25, 0.34, 0.36, 0.98),
        HullMood::Aft => Color::new(0.2, 0.28, 0.31, 0.97),
        HullMood::Buried => Color::new(0.16, 0.21, 0.2, 0.94),
    };
    let hull_center = center - vec2(0.0, wall_h * 0.45);

    draw_iso_diamond(
        center + vec2(0.0, wall_h * 0.55),
        plate_w * 1.06,
        plate_h * 1.05,
        Color::new(0.0, 0.0, 0.0, 0.36),
    );

    draw_iso_prism(
        hull_center,
        plate_w,
        plate_h,
        wall_h,
        roof,
        Color::new(0.055, 0.075, 0.08, 0.99),
        Color::new(0.035, 0.045, 0.048, 0.99),
    );
    draw_hull_ribs(hull_center, plate_w, plate_h, width as usize);
    draw_hull_breach(hull_center, plate_w, plate_h, mood);
    draw_torn_edges(center, plate_w, plate_h, wall_h, mood);
}

fn draw_hull_ribs(center: Vec2, width: f32, height: f32, count: usize) {
    let rib_count = count.clamp(4, 8);
    for index in 0..rib_count {
        let t = -0.38 + index as f32 * (0.76 / (rib_count - 1) as f32);
        let x = center.x + width * t;
        draw_line(
            x,
            center.y + height * 0.16,
            x + width * 0.12,
            center.y + height * 0.42,
            2.2,
            Color::new(0.74, 0.82, 0.78, 0.68),
        );
    }
    draw_line(
        center.x - width * 0.42,
        center.y + height * 0.36,
        center.x + width * 0.42,
        center.y + height * 0.36,
        2.4,
        Color::new(0.02, 0.024, 0.024, 0.86),
    );
}

fn draw_hull_breach(center: Vec2, width: f32, height: f32, mood: HullMood) {
    let breach_count = if matches!(mood, HullMood::Buried) {
        2
    } else {
        4
    };
    for index in 0..breach_count {
        let x = center.x - width * 0.32 + index as f32 * width * 0.2;
        draw_triangle(
            vec2(x, center.y + height * 0.18),
            vec2(x + width * 0.08, center.y + height * 0.42),
            vec2(x - width * 0.06, center.y + height * 0.4),
            Color::new(0.006, 0.008, 0.008, 0.92),
        );
    }
}

fn draw_torn_edges(center: Vec2, width: f32, height: f32, wall_h: f32, mood: HullMood) {
    let tear_color = Color::new(0.035, 0.04, 0.038, 0.92);
    let spark_color = Color::new(0.9, 0.55, 0.22, 0.78);
    let tear_y = center.y + wall_h * 0.25 + height * 0.2;

    for index in 0..8 {
        let x = center.x - width * 0.42 + index as f32 * width * 0.12;
        draw_triangle(
            vec2(x, tear_y),
            vec2(x + width * 0.06, tear_y + 12.0),
            vec2(x + width * 0.13, tear_y - 3.0),
            tear_color,
        );
    }

    if matches!(mood, HullMood::Main | HullMood::Aft) {
        draw_circle(center.x + width * 0.24, tear_y - 4.0, 4.0, spark_color);
        draw_line(
            center.x + width * 0.22,
            tear_y - 2.0,
            center.x + width * 0.3,
            tear_y + 4.0,
            1.0,
            spark_color,
        );
    }
}

fn draw_broken_spine(iso: IsoView) {
    let sections = [
        (Position::new(6, 8), 5.0, 1.7),
        (Position::new(10, 10), 4.4, 1.5),
        (Position::new(14, 12), 4.8, 1.6),
        (Position::new(18, 14), 4.0, 1.4),
    ];
    for (cell, width, height) in sections {
        let center = iso.grid_to_screen(cell);
        draw_iso_diamond(
            center + vec2(0.0, iso.tile_h * 0.35),
            iso.tile_w * width,
            iso.tile_h * height,
            Color::new(0.11, 0.15, 0.16, 0.96),
        );
        draw_iso_diamond_lines(
            center + vec2(0.0, iso.tile_h * 0.35),
            iso.tile_w * width,
            iso.tile_h * height,
            1.8,
            Color::new(0.75, 0.82, 0.78, 0.48),
        );
    }
}

fn draw_survival_camp(iso: IsoView) {
    for (cell, color) in [
        (Position::new(4, 14), Color::new(0.72, 0.56, 0.27, 0.99)),
        (Position::new(8, 16), Color::new(0.5, 0.68, 0.67, 0.99)),
        (Position::new(12, 15), Color::new(0.72, 0.38, 0.25, 0.99)),
    ] {
        draw_emergency_tent(iso.grid_to_screen(cell), iso.tile_w, iso.tile_h, color);
    }

    for cell in [
        Position::new(6, 13),
        Position::new(10, 15),
        Position::new(13, 14),
    ] {
        draw_crate_stack(iso.grid_to_screen(cell), iso.tile_w, iso.tile_h);
    }

    for (cell, color) in [
        (Position::new(4, 12), Color::new(0.58, 0.27, 0.12, 0.98)),
        (Position::new(11, 14), Color::new(0.32, 0.41, 0.43, 0.98)),
        (Position::new(14, 15), Color::new(0.52, 0.43, 0.22, 0.98)),
    ] {
        draw_supply_container(iso.grid_to_screen(cell), iso.tile_w, iso.tile_h, color);
    }
}

fn draw_emergency_tent(center: Vec2, tile_w: f32, tile_h: f32, color: Color) {
    let base_y = center.y + tile_h * 0.56;
    draw_iso_diamond(
        center + vec2(0.0, tile_h * 0.58),
        tile_w * 1.34,
        tile_h * 0.58,
        Color::new(0.04, 0.04, 0.035, 0.35),
    );
    draw_triangle(
        vec2(center.x - tile_w * 0.48, base_y),
        vec2(center.x, base_y - tile_h * 1.0),
        vec2(center.x + tile_w * 0.48, base_y),
        color,
    );
    draw_triangle(
        vec2(center.x, base_y - tile_h * 1.0),
        vec2(center.x + tile_w * 0.48, base_y),
        vec2(center.x + tile_w * 0.62, base_y + tile_h * 0.24),
        Color::new(color.r * 0.62, color.g * 0.62, color.b * 0.62, color.a),
    );
    draw_line(
        center.x,
        base_y - tile_h * 0.68,
        center.x,
        base_y - tile_h * 0.06,
        1.0,
        Color::new(0.85, 0.78, 0.58, 0.72),
    );
}

fn draw_crate_stack(center: Vec2, tile_w: f32, tile_h: f32) {
    for index in 0..3 {
        let x = center.x - tile_w * 0.2 + index as f32 * tile_w * 0.14;
        draw_rectangle(
            x,
            center.y + tile_h * (0.35 + index as f32 * 0.03),
            tile_w * 0.13,
            tile_h * 0.2,
            Color::new(0.39, 0.31, 0.2, 0.96),
        );
        draw_rectangle_lines(
            x,
            center.y + tile_h * (0.35 + index as f32 * 0.03),
            tile_w * 0.13,
            tile_h * 0.2,
            1.0,
            Color::new(0.73, 0.58, 0.32, 0.55),
        );
    }
}

fn draw_supply_container(center: Vec2, tile_w: f32, tile_h: f32, color: Color) {
    let top = Color::new(color.r * 1.08, color.g * 1.08, color.b * 1.08, color.a);
    draw_iso_prism(
        center + vec2(0.0, tile_h * 0.3),
        tile_w * 1.62,
        tile_h * 0.58,
        tile_h * 0.52,
        top,
        Color::new(color.r * 0.62, color.g * 0.62, color.b * 0.62, color.a),
        Color::new(color.r * 0.45, color.g * 0.45, color.b * 0.45, color.a),
    );

    for offset in [-0.28_f32, 0.0, 0.28] {
        draw_line(
            center.x + tile_w * offset,
            center.y + tile_h * 0.32,
            center.x + tile_w * (offset + 0.16),
            center.y + tile_h * 0.44,
            1.0,
            Color::new(0.94, 0.82, 0.55, 0.45),
        );
    }
}

fn draw_scattered_salvage(iso: IsoView) {
    for cell in [
        Position::new(3, 9),
        Position::new(6, 10),
        Position::new(15, 5),
        Position::new(19, 7),
        Position::new(21, 12),
        Position::new(17, 17),
    ] {
        let center = iso.grid_to_screen(cell);
        draw_half_buried_machine(center, iso.tile_w, iso.tile_h);
    }
}

fn draw_half_buried_machine(center: Vec2, tile_w: f32, tile_h: f32) {
    draw_iso_diamond(
        center + vec2(0.0, tile_h * 0.48),
        tile_w * 0.94,
        tile_h * 0.34,
        Color::new(0.06, 0.065, 0.06, 0.38),
    );
    draw_rectangle(
        center.x - tile_w * 0.28,
        center.y + tile_h * 0.28,
        tile_w * 0.56,
        tile_h * 0.3,
        Color::new(0.16, 0.18, 0.17, 0.96),
    );
    draw_circle_lines(
        center.x - tile_w * 0.08,
        center.y + tile_h * 0.38,
        4.0,
        1.4,
        Color::new(0.62, 0.6, 0.52, 0.62),
    );
    draw_line(
        center.x + tile_w * 0.04,
        center.y + tile_h * 0.28,
        center.x + tile_w * 0.24,
        center.y + tile_h * 0.1,
        1.5,
        Color::new(0.52, 0.44, 0.32, 0.75),
    );
}

fn draw_exposed_wiring(iso: IsoView, tick: u64) {
    let cable_runs = [
        [
            Position::new(7, 7),
            Position::new(9, 10),
            Position::new(8, 14),
        ],
        [
            Position::new(15, 10),
            Position::new(13, 13),
            Position::new(11, 16),
        ],
        [
            Position::new(18, 11),
            Position::new(20, 13),
            Position::new(22, 16),
        ],
    ];

    for (index, run) in cable_runs.iter().enumerate() {
        for segment in run.windows(2) {
            let a = iso.grid_to_screen(segment[0]) + vec2(0.0, iso.tile_h * 0.52);
            let b = iso.grid_to_screen(segment[1]) + vec2(0.0, iso.tile_h * 0.52);
            draw_line(a.x, a.y, b.x, b.y, 4.4, Color::new(0.0, 0.0, 0.0, 0.88));
            draw_line(a.x, a.y, b.x, b.y, 2.0, Color::new(0.86, 0.45, 0.16, 0.74));
            if (tick / 18 + index as u64) % 3 == 0 {
                let mid = (a + b) * 0.5;
                draw_circle(mid.x, mid.y, 4.0, Color::new(0.94, 0.62, 0.18, 0.88));
            }
        }
    }
}

fn draw_flickering_lights(iso: IsoView, tick: u64) {
    for (cell, seed, color) in [
        (Position::new(6, 5), 7, style::HEADING_BLUE),
        (Position::new(11, 7), 23, style::ACCENT_GOLD),
        (Position::new(16, 8), 47, style::HEADING_BLUE),
        (Position::new(19, 14), 89, style::ACCENT_GOLD),
    ] {
        let center = iso.grid_to_screen(cell);
        let alpha = flicker_alpha(tick, seed);
        draw_circle(
            center.x,
            center.y + iso.tile_h * 0.18,
            7.0 + alpha * 5.0,
            Color::new(color.r, color.g, color.b, alpha * 0.5),
        );
        draw_circle(
            center.x,
            center.y + iso.tile_h * 0.18,
            3.0,
            Color::new(color.r, color.g, color.b, alpha),
        );
    }
}

fn draw_smoke_vents(iso: IsoView, tick: u64) {
    for vent in SMOKE_VENTS {
        let base = iso.grid_to_screen(vent.cell) + vec2(0.0, iso.tile_h * 0.08);
        draw_circle(
            base.x,
            base.y + iso.tile_h * 0.2,
            3.4 * vent.strength,
            Color::new(0.025, 0.024, 0.02, 0.82),
        );
        for puff in 0..4 {
            let drift = ((tick + vent.seed + puff * 17) % 90) as f32 / 90.0;
            let x = base.x + (puff as f32 - 1.4) * 8.0 + drift * 14.0;
            let y = base.y - drift * 66.0 - puff as f32 * 7.0;
            let radius = (8.0 + drift * 16.0) * vent.strength;
            let alpha = (0.48 * (1.0 - drift)).max(0.1) * vent.strength;
            draw_circle(x, y, radius, Color::new(0.48, 0.5, 0.46, alpha));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_scorch_intensity_marks_wreck_core() {
        assert!(crash_scorch_intensity(8, 5) > 0.9);
        assert!(crash_scorch_intensity(0, 0) < 0.1);
    }

    #[test]
    fn test_flicker_alpha_stays_visible_and_bounded() {
        for tick in [0, 9, 18, 27, 36, 90, 144] {
            let alpha = flicker_alpha(tick, 13);
            assert!((0.42..=0.9).contains(&alpha));
        }
    }
}
