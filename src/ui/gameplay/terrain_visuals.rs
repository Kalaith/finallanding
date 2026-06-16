use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TerrainDetail {
    None,
    Scrap,
    Brush,
    Scorch,
    Wreckage,
    Cable,
    Track,
    SupplyCrate,
    HullPanel,
    SignalBeacon,
    FuelDrum,
}

pub(crate) fn terrain_color(cell_type: Option<CellType>, x: i32, y: i32) -> Color {
    let seed = terrain_seed(x, y);
    let tint = ((seed % 9) as f32 - 4.0) * 0.006;

    match cell_type {
        Some(CellType::Empty) => Color::new(0.18 + tint, 0.16 + tint, 0.105 + tint, 1.0),
        Some(CellType::Floor) => Color::new(0.235 + tint, 0.215 + tint, 0.15 + tint, 1.0),
        Some(CellType::Wall) => Color::new(0.145 + tint, 0.165 + tint, 0.145 + tint, 1.0),
        None => BLACK,
    }
}

pub(crate) fn terrain_detail(cell_type: Option<CellType>, x: i32, y: i32) -> TerrainDetail {
    if cell_type.is_none() {
        return TerrainDetail::None;
    }

    if let Some(detail) = crash_site_detail(x, y) {
        return detail;
    }

    let seed = terrain_seed(x, y);
    if seed.is_multiple_of(31) {
        TerrainDetail::HullPanel
    } else if seed.is_multiple_of(29) {
        TerrainDetail::SupplyCrate
    } else if seed.is_multiple_of(23) {
        TerrainDetail::Scrap
    } else if seed.is_multiple_of(19) {
        TerrainDetail::Scorch
    } else if seed.is_multiple_of(13) {
        TerrainDetail::Brush
    } else {
        TerrainDetail::None
    }
}

pub(crate) fn crash_site_detail(x: i32, y: i32) -> Option<TerrainDetail> {
    if (10..=12).contains(&x) && y == 10 {
        return Some(TerrainDetail::SupplyCrate);
    }

    if (14..=15).contains(&x) && y == 5 {
        return Some(TerrainDetail::SignalBeacon);
    }

    if (5..=7).contains(&x) && (10..=11).contains(&y) && (x + y) % 2 == 0 {
        return Some(TerrainDetail::HullPanel);
    }

    if (12..=14).contains(&x) && y == 7 && x % 2 == 1 {
        return Some(TerrainDetail::FuelDrum);
    }

    if (6..=13).contains(&x) && (7..=9).contains(&y) && (x + y) % 3 == 0 {
        return Some(TerrainDetail::Wreckage);
    }

    if (4..=15).contains(&x) && (x - y).abs() <= 1 && (x + y) % 4 == 0 {
        return Some(TerrainDetail::Track);
    }

    if (7..=15).contains(&x) && (5..=11).contains(&y) && (x * 2 + y) % 11 == 0 {
        return Some(TerrainDetail::Cable);
    }

    None
}

pub(crate) fn draw_terrain_detail(center: Vec2, tile_w: f32, tile_h: f32, detail: TerrainDetail) {
    match detail {
        TerrainDetail::None => {}
        TerrainDetail::Scrap => {
            draw_line(
                center.x - tile_w * 0.11,
                center.y + tile_h * 0.46,
                center.x + tile_w * 0.06,
                center.y + tile_h * 0.34,
                1.0,
                Color::new(0.48, 0.48, 0.42, 0.65),
            );
            draw_circle(
                center.x + tile_w * 0.09,
                center.y + tile_h * 0.58,
                1.4,
                Color::new(0.62, 0.52, 0.34, 0.75),
            );
        }
        TerrainDetail::Brush => {
            draw_line(
                center.x - tile_w * 0.08,
                center.y + tile_h * 0.55,
                center.x - tile_w * 0.02,
                center.y + tile_h * 0.38,
                1.2,
                Color::new(0.22, 0.32, 0.16, 0.7),
            );
            draw_line(
                center.x + tile_w * 0.02,
                center.y + tile_h * 0.58,
                center.x + tile_w * 0.08,
                center.y + tile_h * 0.42,
                1.2,
                Color::new(0.18, 0.28, 0.13, 0.7),
            );
        }
        TerrainDetail::Scorch => {
            draw_iso_diamond(
                center + vec2(0.0, tile_h * 0.12),
                tile_w * 0.48,
                tile_h * 0.48,
                Color::new(0.05, 0.045, 0.035, 0.35),
            );
        }
        TerrainDetail::Wreckage => {
            draw_iso_diamond(
                center + vec2(tile_w * 0.04, tile_h * 0.35),
                tile_w * 0.32,
                tile_h * 0.18,
                Color::new(0.36, 0.35, 0.31, 0.8),
            );
            draw_line(
                center.x - tile_w * 0.12,
                center.y + tile_h * 0.42,
                center.x + tile_w * 0.18,
                center.y + tile_h * 0.34,
                1.2,
                Color::new(0.62, 0.52, 0.34, 0.78),
            );
            draw_circle(
                center.x + tile_w * 0.16,
                center.y + tile_h * 0.34,
                1.8,
                style::ACCENT_GOLD,
            );
        }
        TerrainDetail::Cable => {
            draw_line(
                center.x - tile_w * 0.2,
                center.y + tile_h * 0.5,
                center.x - tile_w * 0.04,
                center.y + tile_h * 0.43,
                1.3,
                Color::new(0.05, 0.055, 0.055, 0.82),
            );
            draw_line(
                center.x - tile_w * 0.04,
                center.y + tile_h * 0.43,
                center.x + tile_w * 0.18,
                center.y + tile_h * 0.54,
                1.3,
                Color::new(0.05, 0.055, 0.055, 0.82),
            );
        }
        TerrainDetail::Track => {
            draw_iso_diamond(
                center + vec2(0.0, tile_h * 0.2),
                tile_w * 0.72,
                tile_h * 0.34,
                Color::new(0.08, 0.07, 0.045, 0.28),
            );
        }
        TerrainDetail::SupplyCrate => {
            draw_iso_diamond(
                center + vec2(0.0, tile_h * 0.36),
                tile_w * 0.36,
                tile_h * 0.22,
                Color::new(0.42, 0.33, 0.21, 0.92),
            );
            draw_rectangle(
                center.x - tile_w * 0.11,
                center.y + tile_h * 0.32,
                tile_w * 0.22,
                tile_h * 0.22,
                Color::new(0.24, 0.18, 0.12, 0.92),
            );
            draw_line(
                center.x - tile_w * 0.1,
                center.y + tile_h * 0.39,
                center.x + tile_w * 0.1,
                center.y + tile_h * 0.39,
                1.0,
                style::ACCENT_GOLD,
            );
        }
        TerrainDetail::HullPanel => {
            draw_iso_diamond(
                center + vec2(tile_w * 0.02, tile_h * 0.32),
                tile_w * 0.46,
                tile_h * 0.2,
                Color::new(0.24, 0.3, 0.31, 0.82),
            );
            draw_line(
                center.x - tile_w * 0.16,
                center.y + tile_h * 0.34,
                center.x + tile_w * 0.16,
                center.y + tile_h * 0.42,
                1.0,
                Color::new(0.66, 0.7, 0.67, 0.7),
            );
        }
        TerrainDetail::SignalBeacon => {
            draw_line(
                center.x,
                center.y + tile_h * 0.5,
                center.x,
                center.y + tile_h * 0.05,
                1.6,
                Color::new(0.55, 0.58, 0.55, 0.92),
            );
            draw_circle(
                center.x,
                center.y + tile_h * 0.02,
                3.0,
                Color::new(
                    style::HEADING_BLUE.r,
                    style::HEADING_BLUE.g,
                    style::HEADING_BLUE.b,
                    0.9,
                ),
            );
            draw_circle_lines(
                center.x,
                center.y + tile_h * 0.02,
                6.0,
                1.0,
                Color::new(
                    style::HEADING_BLUE.r,
                    style::HEADING_BLUE.g,
                    style::HEADING_BLUE.b,
                    0.45,
                ),
            );
        }
        TerrainDetail::FuelDrum => {
            draw_rectangle(
                center.x - tile_w * 0.08,
                center.y + tile_h * 0.31,
                tile_w * 0.16,
                tile_h * 0.28,
                Color::new(0.34, 0.24, 0.18, 0.95),
            );
            draw_ellipse(
                center.x,
                center.y + tile_h * 0.31,
                tile_w * 0.08,
                tile_h * 0.04,
                0.0,
                Color::new(0.52, 0.38, 0.24, 0.95),
            );
            draw_line(
                center.x - tile_w * 0.07,
                center.y + tile_h * 0.46,
                center.x + tile_w * 0.07,
                center.y + tile_h * 0.46,
                1.0,
                style::ACCENT_GOLD,
            );
        }
    }
}

pub(crate) fn terrain_seed(x: i32, y: i32) -> u32 {
    let x = x as u32;
    let y = y as u32;
    x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ 0x9E37_79B9
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::grid::CellType;

    #[test]
    fn test_terrain_detail_is_deterministic_and_skips_missing_cells() {
        assert_eq!(
            terrain_detail(Some(CellType::Empty), 7, 11),
            terrain_detail(Some(CellType::Empty), 7, 11)
        );
        assert_eq!(terrain_detail(None, 7, 11), TerrainDetail::None);
    }

    #[test]
    fn test_crash_site_detail_adds_deterministic_map_dressing() {
        assert_eq!(crash_site_detail(10, 10), Some(TerrainDetail::SupplyCrate));
        assert_eq!(crash_site_detail(15, 5), Some(TerrainDetail::SignalBeacon));
        assert_eq!(crash_site_detail(5, 11), Some(TerrainDetail::HullPanel));
        assert_eq!(crash_site_detail(13, 7), Some(TerrainDetail::FuelDrum));
        assert_eq!(crash_site_detail(8, 7), Some(TerrainDetail::Wreckage));
        assert_eq!(crash_site_detail(4, 4), Some(TerrainDetail::Track));
        assert_eq!(crash_site_detail(8, 6), Some(TerrainDetail::Cable));
        assert_eq!(crash_site_detail(0, 0), None);
    }

    #[test]
    fn test_terrain_color_varies_without_leaving_palette() {
        let first = terrain_color(Some(CellType::Empty), 1, 1);
        let second = terrain_color(Some(CellType::Empty), 2, 1);

        assert_ne!(first, second);
        assert!((0.14..=0.22).contains(&first.r));
        assert!((0.08..=0.14).contains(&first.b));
    }
}
