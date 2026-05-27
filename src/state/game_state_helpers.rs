use super::*;

pub(super) fn building_wall_height(building_type: BuildingType, tile_h: f32) -> f32 {
    let multiplier = match building_type {
        BuildingType::Habitat => 0.95,
        BuildingType::MessHall => 0.78,
        BuildingType::Workshop => 1.12,
        BuildingType::Storage => 0.64,
        BuildingType::ExplorationGate => 1.25,
    };
    tile_h * multiplier
}

pub(super) fn building_outline_style(
    hovered: bool,
    assignment_color: Option<Color>,
) -> Option<(Color, f32)> {
    if hovered {
        Some((Color::new(0.92, 0.8, 0.45, 0.96), 3.0))
    } else {
        assignment_color.map(|color| (Color::new(color.r, color.g, color.b, 0.9), 2.0))
    }
}

pub(super) fn assignment_marker_with_filter(
    assignment_marker: Option<(&'static str, Color)>,
    filter_match: bool,
) -> Option<(&'static str, Color)> {
    assignment_marker.or_else(|| filter_match.then_some(("FILTER", style::ACCENT_GOLD)))
}

pub(super) fn building_outline_style_for_assign_filter(
    hovered: bool,
    assignment_color: Option<Color>,
    filter_match: bool,
) -> Option<(Color, f32)> {
    if filter_match {
        Some((style::ACCENT_GOLD, 3.0))
    } else {
        building_outline_style(hovered, assignment_color)
    }
}

pub(super) fn building_shell_colors(building_type: BuildingType) -> (Color, Color, Color) {
    match building_type {
        BuildingType::Habitat => (
            Color::new(0.24, 0.34, 0.42, 1.0),
            Color::new(0.13, 0.2, 0.25, 1.0),
            Color::new(0.09, 0.15, 0.19, 1.0),
        ),
        BuildingType::MessHall => (
            Color::new(0.48, 0.36, 0.18, 1.0),
            Color::new(0.29, 0.21, 0.11, 1.0),
            Color::new(0.2, 0.15, 0.09, 1.0),
        ),
        BuildingType::Workshop => (
            Color::new(0.39, 0.31, 0.25, 1.0),
            Color::new(0.24, 0.18, 0.15, 1.0),
            Color::new(0.16, 0.13, 0.12, 1.0),
        ),
        BuildingType::Storage => (
            Color::new(0.35, 0.36, 0.34, 1.0),
            Color::new(0.2, 0.22, 0.21, 1.0),
            Color::new(0.15, 0.16, 0.16, 1.0),
        ),
        BuildingType::ExplorationGate => (
            Color::new(0.32, 0.29, 0.45, 1.0),
            Color::new(0.2, 0.18, 0.3, 1.0),
            Color::new(0.13, 0.12, 0.2, 1.0),
        ),
    }
}

pub(super) fn draw_building_shell_detail(
    building_type: BuildingType,
    center: Vec2,
    width: f32,
    height: f32,
) {
    draw_iso_diamond(
        center + vec2(0.0, height * 0.08),
        width * 0.66,
        height * 0.5,
        Color::new(0.05, 0.06, 0.055, 0.52),
    );

    match building_type {
        BuildingType::Habitat => {
            for index in 0..2 {
                let bed_x = center.x - width * 0.2 + index as f32 * width * 0.2;
                draw_rectangle(
                    bed_x,
                    center.y + height * 0.28,
                    12.0,
                    7.0,
                    Color::new(0.56, 0.64, 0.68, 1.0),
                );
                draw_rectangle(
                    bed_x + 1.5,
                    center.y + height * 0.29,
                    4.0,
                    5.0,
                    Color::new(0.18, 0.24, 0.27, 1.0),
                );
            }
            draw_line(
                center.x - width * 0.28,
                center.y + height * 0.23,
                center.x + width * 0.28,
                center.y + height * 0.23,
                1.0,
                Color::new(0.66, 0.74, 0.75, 0.7),
            );
            draw_circle(
                center.x,
                center.y + height * 0.23,
                2.4,
                Color::new(
                    style::HEADING_BLUE.r,
                    style::HEADING_BLUE.g,
                    style::HEADING_BLUE.b,
                    0.85,
                ),
            );
            draw_rectangle(
                center.x - width * 0.14,
                center.y + height * 0.56,
                6.0,
                4.0,
                style::HEADING_BLUE,
            );
            draw_rectangle(
                center.x + width * 0.05,
                center.y + height * 0.56,
                6.0,
                4.0,
                style::HEADING_BLUE,
            );
        }
        BuildingType::MessHall => {
            draw_rectangle(
                center.x - width * 0.17,
                center.y + height * 0.31,
                width * 0.34,
                7.0,
                Color::new(0.17, 0.12, 0.07, 1.0),
            );
            for index in 0..4 {
                let place_x = center.x - width * 0.14 + index as f32 * width * 0.09;
                draw_circle(
                    place_x,
                    center.y + height * 0.34,
                    2.2,
                    Color::new(0.76, 0.68, 0.48, 1.0),
                );
            }
            for index in 0..3 {
                let stool_x = center.x - width * 0.16 + index as f32 * width * 0.16;
                draw_circle(
                    stool_x,
                    center.y + height * 0.44,
                    2.4,
                    Color::new(0.31, 0.22, 0.12, 0.95),
                );
            }
            draw_line(
                center.x - width * 0.22,
                center.y + height * 0.48,
                center.x + width * 0.22,
                center.y + height * 0.48,
                2.0,
                style::ACCENT_GOLD,
            );
            draw_circle(center.x, center.y + height * 0.48, 3.0, style::BAR_GOLD);
        }
        BuildingType::Workshop => {
            draw_rectangle(
                center.x - width * 0.18,
                center.y + height * 0.3,
                width * 0.36,
                8.0,
                Color::new(0.13, 0.1, 0.08, 1.0),
            );
            draw_line(
                center.x - width * 0.13,
                center.y + height * 0.32,
                center.x + width * 0.1,
                center.y + height * 0.32,
                1.0,
                style::ACCENT_GOLD,
            );
            draw_circle(
                center.x + width * 0.14,
                center.y + height * 0.27,
                2.2,
                Color::new(0.9, 0.58, 0.23, 0.9),
            );
            draw_rectangle(
                center.x - 6.0,
                center.y + height * 0.38,
                12.0,
                8.0,
                Color::new(0.07, 0.08, 0.08, 1.0),
            );
            for index in 0..3 {
                let spark_x = center.x + width * 0.03 + index as f32 * 4.0;
                draw_line(
                    spark_x,
                    center.y + height * 0.23,
                    spark_x + 2.0,
                    center.y + height * 0.18,
                    1.0,
                    style::ACCENT_GOLD,
                );
            }
            draw_rectangle(
                center.x - width * 0.25,
                center.y + height * 0.2,
                9.0,
                12.0,
                Color::new(0.09, 0.11, 0.11, 0.95),
            );
            draw_line(
                center.x - width * 0.245,
                center.y + height * 0.25,
                center.x - width * 0.12,
                center.y + height * 0.25,
                1.0,
                style::TEXT_MUTED,
            );
            draw_line(
                center.x + width * 0.18,
                center.y + height * 0.2,
                center.x + width * 0.25,
                center.y - 10.0,
                2.0,
                style::TEXT_MUTED,
            );
        }
        BuildingType::Storage => {
            draw_iso_diamond(
                center + vec2(0.0, height * 0.32),
                width * 0.44,
                height * 0.2,
                Color::new(0.19, 0.2, 0.18, 1.0),
            );
            for index in 0..3 {
                draw_rectangle(
                    center.x - 18.0 + index as f32 * 12.0,
                    center.y + height * 0.5,
                    9.0,
                    7.0,
                    Color::new(0.48, 0.42, 0.32, 1.0),
                );
                draw_rectangle(
                    center.x - 16.0 + index as f32 * 12.0,
                    center.y + height * 0.5,
                    5.0,
                    2.0,
                    Color::new(0.68, 0.58, 0.35, 0.9),
                );
            }
            draw_iso_diamond(
                center + vec2(width * 0.2, height * 0.42),
                width * 0.16,
                height * 0.08,
                Color::new(0.38, 0.45, 0.44, 0.9),
            );
        }
        BuildingType::ExplorationGate => {
            draw_iso_diamond(
                center + vec2(0.0, height * 0.5),
                width * 0.48,
                height * 0.18,
                Color::new(0.1, 0.12, 0.16, 0.86),
            );
            draw_line(
                center.x,
                center.y + height * 0.18,
                center.x,
                center.y - height * 0.4,
                2.0,
                style::TEXT_BODY,
            );
            draw_circle_lines(
                center.x,
                center.y - height * 0.45,
                8.0,
                1.0,
                style::HEADING_BLUE,
            );
            draw_circle(center.x, center.y - height * 0.45, 2.4, style::ACCENT_GOLD);
            let arch_y = center.y + height * 0.36;
            draw_line(
                center.x - width * 0.18,
                arch_y + 18.0,
                center.x - width * 0.18,
                arch_y - 10.0,
                3.0,
                style::HEADING_BLUE,
            );
            draw_line(
                center.x + width * 0.18,
                arch_y + 18.0,
                center.x + width * 0.18,
                arch_y - 10.0,
                3.0,
                style::HEADING_BLUE,
            );
            draw_line(
                center.x - width * 0.18,
                arch_y - 10.0,
                center.x + width * 0.18,
                arch_y - 10.0,
                3.0,
                style::HEADING_BLUE,
            );
            draw_line(
                center.x - width * 0.22,
                arch_y + 14.0,
                center.x - width * 0.34,
                arch_y + 24.0,
                1.2,
                Color::new(0.06, 0.065, 0.065, 0.9),
            );
            draw_line(
                center.x + width * 0.22,
                arch_y + 14.0,
                center.x + width * 0.34,
                arch_y + 24.0,
                1.2,
                Color::new(0.06, 0.065, 0.065, 0.9),
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TerrainDetail {
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

pub(super) fn terrain_color(cell_type: Option<CellType>, x: i32, y: i32) -> Color {
    let seed = terrain_seed(x, y);
    let tint = ((seed % 9) as f32 - 4.0) * 0.006;

    match cell_type {
        Some(CellType::Empty) => Color::new(0.18 + tint, 0.16 + tint, 0.105 + tint, 1.0),
        Some(CellType::Floor) => Color::new(0.235 + tint, 0.215 + tint, 0.15 + tint, 1.0),
        Some(CellType::Wall) => Color::new(0.145 + tint, 0.165 + tint, 0.145 + tint, 1.0),
        None => BLACK,
    }
}

pub(super) fn terrain_detail(cell_type: Option<CellType>, x: i32, y: i32) -> TerrainDetail {
    if cell_type.is_none() {
        return TerrainDetail::None;
    }

    if let Some(detail) = crash_site_detail(x, y) {
        return detail;
    }

    let seed = terrain_seed(x, y);
    if seed % 31 == 0 {
        TerrainDetail::HullPanel
    } else if seed % 29 == 0 {
        TerrainDetail::SupplyCrate
    } else if seed % 23 == 0 {
        TerrainDetail::Scrap
    } else if seed % 19 == 0 {
        TerrainDetail::Scorch
    } else if seed % 13 == 0 {
        TerrainDetail::Brush
    } else {
        TerrainDetail::None
    }
}

pub(super) fn crash_site_detail(x: i32, y: i32) -> Option<TerrainDetail> {
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

pub(super) fn draw_terrain_detail(center: Vec2, tile_w: f32, tile_h: f32, detail: TerrainDetail) {
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

pub(super) fn terrain_seed(x: i32, y: i32) -> u32 {
    let x = x as u32;
    let y = y as u32;
    x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ 0x9E37_79B9
}

pub(super) fn placement_result_reason(result: &PlacementResult) -> &'static str {
    match result {
        PlacementResult::Success(_) => "Placement succeeded.",
        PlacementResult::OutOfBounds => "Footprint leaves the map.",
        PlacementResult::AreaOccupied => "Footprint overlaps blocked or occupied space.",
        PlacementResult::InvalidBuilding => "Building configuration is invalid.",
    }
}

pub(super) fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let mut truncated = text
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

pub(super) fn job_color(job_preference: crate::data::colonist::JobPreference) -> Color {
    match job_preference {
        crate::data::colonist::JobPreference::Explorer => PURPLE,
        crate::data::colonist::JobPreference::Builder => YELLOW,
        crate::data::colonist::JobPreference::Cook => GREEN,
        crate::data::colonist::JobPreference::Hauler => GRAY,
        crate::data::colonist::JobPreference::None => WHITE,
    }
}

pub(super) fn colonist_activity_summary(colonist: &Colonist) -> &'static str {
    match colonist.state {
        ColonistState::Idle => "Idle",
        ColonistState::Moving { .. } => "Moving",
        ColonistState::Working => "Working",
        ColonistState::Eating => "Eating",
        ColonistState::Sleeping => "Resting",
        ColonistState::OnMission { .. } => "On mission",
    }
}

pub(super) fn sprite_pose_for_state(state: ColonistState) -> SpritePose {
    match state {
        ColonistState::Idle => SpritePose::Idle,
        ColonistState::Moving { .. } => SpritePose::Moving,
        ColonistState::Working => SpritePose::Working,
        ColonistState::Eating => SpritePose::Eating,
        ColonistState::Sleeping => SpritePose::Sleeping,
        ColonistState::OnMission { .. } => SpritePose::Moving,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SocialBodyLanguage {
    Supported(i32),
    Tense(i32),
}

impl SocialBodyLanguage {
    pub(super) fn intensity(self) -> i32 {
        match self {
            SocialBodyLanguage::Supported(value) | SocialBodyLanguage::Tense(value) => value.abs(),
        }
    }

    pub(super) fn color(self, alpha: f32) -> Color {
        match self {
            SocialBodyLanguage::Supported(_) => Color::new(
                style::BAR_GREEN.r,
                style::BAR_GREEN.g,
                style::BAR_GREEN.b,
                alpha,
            ),
            SocialBodyLanguage::Tense(_) => Color::new(
                style::ALERT_RED.r,
                style::ALERT_RED.g,
                style::ALERT_RED.b,
                alpha,
            ),
        }
    }

    pub(super) fn symbol(self) -> &'static str {
        match self {
            SocialBodyLanguage::Supported(_) => "+",
            SocialBodyLanguage::Tense(_) => "!",
        }
    }
}

pub(super) fn sprite_pose_for_colonist(
    colonist: &Colonist,
    social_signal: Option<SocialBodyLanguage>,
) -> SpritePose {
    sprite_pose_for_colonist_frame(colonist, social_signal, 0)
}

pub(super) fn sprite_pose_for_colonist_frame(
    colonist: &Colonist,
    social_signal: Option<SocialBodyLanguage>,
    tick: u64,
) -> SpritePose {
    if let Some(signal) = social_signal {
        return match signal {
            SocialBodyLanguage::Supported(_) => {
                if social_pose_uses_alternate_frame(tick) {
                    SpritePose::SupportedReach
                } else {
                    SpritePose::Supported
                }
            }
            SocialBodyLanguage::Tense(_) => {
                if social_pose_uses_alternate_frame(tick) {
                    SpritePose::TenseGuarded
                } else {
                    SpritePose::Tense
                }
            }
        };
    }

    sprite_pose_for_state(colonist.state)
}

pub(super) fn social_pose_uses_alternate_frame(tick: u64) -> bool {
    (tick / 45) % 2 == 1
}

pub(super) fn shared_assignment_pin(first: &Colonist, second: &Colonist) -> bool {
    first
        .assigned_habitat
        .is_some_and(|id| second.assigned_habitat == Some(id))
        || first
            .assigned_workplace
            .is_some_and(|id| second.assigned_workplace == Some(id))
}

pub(super) fn adjacent_positions(first: Position, second: Position) -> bool {
    (first.x - second.x).abs() + (first.y - second.y).abs() <= 1
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpaceAssignmentKind {
    Recovery,
    Work,
}

pub(super) fn space_assignment_kind(
    job_preference: crate::data::colonist::JobPreference,
    building_type: BuildingType,
) -> Option<SpaceAssignmentKind> {
    if building_type == BuildingType::Habitat {
        return Some(SpaceAssignmentKind::Recovery);
    }

    (building_type == job_preference.work_building_type()).then_some(SpaceAssignmentKind::Work)
}

pub(super) fn directive_log_detail(
    directive: PairDirective,
    first_name: &str,
    second_name: &str,
) -> String {
    match directive {
        PairDirective::Pair => format!(
            "{} and {} will prefer the same work and recovery spaces when the settlement has a choice.",
            first_name, second_name
        ),
        PairDirective::Separate => format!(
            "{} and {} will avoid sharing work and recovery spaces when another option exists.",
            first_name, second_name
        ),
    }
}

pub(super) fn initial_toolbar_mode() -> ToolbarMode {
    std::env::var("TFL_START_TOOLBAR_MODE")
        .ok()
        .and_then(|value| toolbar_mode_from_name(&value))
        .unwrap_or(ToolbarMode::Build)
}

pub(super) fn initial_selected_building(toolbar_mode: ToolbarMode) -> Option<BuildingType> {
    std::env::var("TFL_START_SELECTED_BUILDING")
        .ok()
        .and_then(|value| building_type_from_name(&value))
        .filter(|building_type| {
            toolbar_mode.uses_building_choices()
                && toolbar_buildings_for_mode(toolbar_mode).contains(building_type)
        })
}

pub(super) fn initial_capture_preview_position() -> Option<Position> {
    let x = std::env::var("TFL_PREVIEW_GRID_X")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())?;
    let y = std::env::var("TFL_PREVIEW_GRID_Y")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())?;
    Some(Position::new(x, y))
}

pub(super) fn seed_assign_spaces_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_ASSIGN_SPACES").is_ok_and(|value| value != "0") {
        return;
    }

    let placements = [
        (BuildingType::Habitat, Position::new(3, 4)),
        (BuildingType::Habitat, Position::new(8, 4)),
        (BuildingType::Workshop, Position::new(6, 8)),
        (BuildingType::Storage, Position::new(12, 8)),
    ];

    let mut habitat_id = None;
    let mut workshop_id = None;
    for (building_type, position) in placements {
        if let PlacementResult::Success(building_id) =
            data.building_system
                .try_place_building(&mut data.grid, building_type, position)
        {
            if building_type == BuildingType::Habitat && habitat_id.is_none() {
                habitat_id = Some(building_id);
            } else if building_type == BuildingType::Workshop {
                workshop_id = Some(building_id);
            }
        }
    }

    if let Some(colonist) = data.colonists.iter_mut().find(|colonist| colonist.id == 5) {
        colonist.assigned_habitat = habitat_id;
        colonist.assigned_workplace = workshop_id;
    }
    if let Some(colonist) = data.colonists.iter_mut().find(|colonist| colonist.id == 0) {
        colonist.assigned_habitat = habitat_id;
    }
}

pub(super) fn seed_social_history_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_SOCIAL_HISTORY").is_ok_and(|value| value != "0") {
        return;
    }

    for entry in [
        SocialHistoryEntry::new(
            0,
            "Crash night summary",
            "The first shelter line held, but Alice and Fiona carried visible tension while Charlie and Evan kept field work steady.",
            "Use Assign to keep Alice and Fiona apart until recovery space improves.",
            50.0,
            1.0,
            2,
            1,
        ),
        SocialHistoryEntry::new(
            1,
            "Mess routine settled",
            "Shared meals improved morale around Bob and Diana, but the workshop queue still created late shifts.",
            "Keep cooks near supportive partners and reduce workshop crowding.",
            58.0,
            6.0,
            2,
            0,
        ),
        SocialHistoryEntry::new(
            2,
            "Workshop strain returned",
            "Alice and Fiona overlapped at the stockpile again, cutting into the recovery gains from yesterday.",
            "Separate tense workers before assigning the next salvage push.",
            47.0,
            -4.0,
            1,
            1,
        ),
        SocialHistoryEntry::new(
            3,
            "Habitat pairs adjusted",
            "Room pins gave Charlie and Evan a reliable recovery loop while Alice took quieter repair shifts.",
            "Protect the supportive pair and avoid crowding the west habitat.",
            61.0,
            8.0,
            2,
            0,
        ),
        SocialHistoryEntry::new(
            4,
            "Late repair friction",
            "The workshop recovered output, but Diana and Fiona clashed during the evening tool handoff.",
            "Move one of them to field prep before the next high-pressure day.",
            53.0,
            -7.0,
            1,
            1,
        ),
    ] {
        data.push_social_history(entry);
    }
}

pub(super) fn seed_activity_poses_for_capture(data: &mut GameState) {
    if !std::env::var("TFL_SEED_ACTIVITY_POSES").is_ok_and(|value| value != "0") {
        return;
    }

    data.time.speed = TimeSpeed::Paused;
    let pose_layout = [
        (0, Position::new(3, 7), ColonistState::Idle),
        (
            1,
            Position::new(6, 7),
            ColonistState::Moving {
                target: Position::new(7, 7),
            },
        ),
        (2, Position::new(9, 7), ColonistState::Working),
        (3, Position::new(12, 7), ColonistState::Eating),
        (4, Position::new(15, 7), ColonistState::Sleeping),
    ];

    for (index, position, state) in pose_layout {
        if let Some(colonist) = data.colonists.get_mut(index) {
            colonist.position = position;
            colonist.visual_x = position.x as f32 * 32.0;
            colonist.visual_y = position.y as f32 * 32.0;
            colonist.state = state;
        }
    }
}

pub(super) fn initial_selected_colonist_id(
    data: &GameState,
    toolbar_mode: ToolbarMode,
) -> Option<u32> {
    std::env::var("TFL_START_SELECTED_COLONIST")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|id| data.colonists.iter().any(|colonist| colonist.id == *id))
        .or_else(|| {
            (toolbar_mode == ToolbarMode::Assign)
                .then(|| data.colonists.first().map(|colonist| colonist.id))
                .flatten()
        })
}

pub(super) fn initial_selected_social_history_day(data: &GameState) -> Option<u32> {
    std::env::var("TFL_START_SOCIAL_HISTORY_DAY")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|day| data.social_history.iter().any(|entry| entry.day == *day))
}

pub(super) fn write_social_archive_markdown(
    history: &[SocialHistoryEntry],
) -> Result<PathBuf, String> {
    let output_dir = PathBuf::from("docs").join("exports");
    std::fs::create_dir_all(&output_dir)
        .map_err(|error| format!("Could not create {}: {}", output_dir.display(), error))?;
    let output_path = output_dir.join("social_archive.md");
    std::fs::write(&output_path, social_archive_markdown(history))
        .map_err(|error| format!("Could not write {}: {}", output_path.display(), error))?;
    Ok(output_path)
}

pub(super) fn social_archive_markdown(history: &[SocialHistoryEntry]) -> String {
    let mut output = String::from("# The Final Landing Social Archive\n\n");
    output.push_str(&format!("Reports: {}\n\n", history.len()));

    for entry in history.iter().rev() {
        output.push_str(&format!("## Day {}: {}\n\n", entry.day, entry.title));
        output.push_str(&format!(
            "- Mood: {:.0}\n- Relationship: {:+.0}\n- Close pairs: {}\n- Strained pairs: {}\n\n",
            entry.average_mood, entry.average_relationship, entry.close_pairs, entry.strained_pairs
        ));
        output.push_str(&format!("{}\n\n", entry.detail));
        output.push_str(&format!("Recommendation: {}\n\n", entry.recommendation));
    }

    output
}

pub(super) fn toolbar_mode_from_name(value: &str) -> Option<ToolbarMode> {
    match value.trim().to_ascii_lowercase().as_str() {
        "build" => Some(ToolbarMode::Build),
        "rooms" => Some(ToolbarMode::Rooms),
        "objects" => Some(ToolbarMode::Objects),
        "colony" => Some(ToolbarMode::Colony),
        "research" => Some(ToolbarMode::Research),
        "assign" => Some(ToolbarMode::Assign),
        "log" => Some(ToolbarMode::Log),
        _ => None,
    }
}

pub(super) fn building_type_from_name(value: &str) -> Option<BuildingType> {
    match value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
        .as_str()
    {
        "habitat" => Some(BuildingType::Habitat),
        "mess_hall" | "messhall" => Some(BuildingType::MessHall),
        "workshop" => Some(BuildingType::Workshop),
        "storage" => Some(BuildingType::Storage),
        "exploration_gate" | "explorationgate" | "gate" => Some(BuildingType::ExplorationGate),
        _ => None,
    }
}

pub(super) const ASSIGN_ROSTER_SLOT_COUNT: usize = 5;

pub(super) fn assign_roster_page_count(
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> usize {
    let selected_exists = selected_colonist_id
        .and_then(|id| colonists.iter().position(|colonist| colonist.id == id))
        .is_some();
    let other_count = (0..colonists.len())
        .filter(|index| Some(colonists[*index].id) != selected_colonist_id)
        .filter(|index| {
            assign_roster_filter_matches(&colonists[*index], active_filter, active_role_filter)
                && assign_building_filter_matches(&colonists[*index], active_building_filter)
        })
        .count();
    let page_size = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(usize::from(selected_exists));

    ((other_count + page_size - 1) / page_size).max(1)
}

pub(super) fn assign_visible_colonist_indices(
    colonists: &[Colonist],
    selected_colonist_id: Option<u32>,
    page: usize,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<usize> {
    let mut indices = Vec::new();

    let selected_index =
        selected_colonist_id.and_then(|id| colonists.iter().position(|colonist| colonist.id == id));

    if let Some(index) = selected_index {
        indices.push(index);
    }

    let open_slots = ASSIGN_ROSTER_SLOT_COUNT.saturating_sub(indices.len());
    let page = page.min(
        assign_roster_page_count(
            colonists,
            selected_colonist_id,
            active_filter,
            active_role_filter,
            active_building_filter,
        ) - 1,
    );

    let roster = assign_sorted_roster_indices(
        colonists,
        selected_index,
        active_filter,
        active_sort,
        active_role_filter,
        active_building_filter,
    );
    indices.extend(roster.into_iter().skip(page * open_slots).take(open_slots));

    indices
}

pub(super) fn assign_sorted_roster_indices(
    colonists: &[Colonist],
    selected_index: Option<usize>,
    active_filter: AssignRosterFilter,
    active_sort: AssignRosterSort,
    active_role_filter: Option<JobPreference>,
    active_building_filter: Option<u32>,
) -> Vec<usize> {
    let mut indices = (0..colonists.len())
        .filter(|index| Some(*index) != selected_index)
        .filter(|index| {
            assign_roster_filter_matches(&colonists[*index], active_filter, active_role_filter)
                && assign_building_filter_matches(&colonists[*index], active_building_filter)
        })
        .collect::<Vec<_>>();

    match active_sort {
        AssignRosterSort::Roster => {}
        AssignRosterSort::Mood => indices.sort_by(|left, right| {
            colonists[*left]
                .mood
                .partial_cmp(&colonists[*right].mood)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| colonists[*left].id.cmp(&colonists[*right].id))
        }),
        AssignRosterSort::Bond => indices.sort_by(|left, right| {
            relationship_pressure_score(&colonists[*right])
                .cmp(&relationship_pressure_score(&colonists[*left]))
                .then_with(|| colonists[*left].id.cmp(&colonists[*right].id))
        }),
    }

    indices
}

pub(super) fn assign_roster_filter_matches(
    colonist: &Colonist,
    active_filter: AssignRosterFilter,
    active_role_filter: Option<JobPreference>,
) -> bool {
    let relationship_match = match active_filter {
        AssignRosterFilter::All => true,
        AssignRosterFilter::Risk => colonist.relationships.values().any(|value| *value <= -10),
        AssignRosterFilter::Support => colonist.relationships.values().any(|value| *value >= 10),
        AssignRosterFilter::Pinned => {
            colonist.assigned_habitat.is_some() || colonist.assigned_workplace.is_some()
        }
    };
    relationship_match && active_role_filter.is_none_or(|role| colonist.job_preference == role)
}

pub(super) fn relationship_pressure_score(colonist: &Colonist) -> i32 {
    colonist
        .relationships
        .values()
        .map(|value| value.abs())
        .max()
        .unwrap_or(0)
}

pub(super) fn assign_building_filter_matches(
    colonist: &Colonist,
    building_id: Option<u32>,
) -> bool {
    building_id.is_none_or(|id| {
        colonist.assigned_habitat == Some(id) || colonist.assigned_workplace == Some(id)
    })
}

pub(super) fn next_assign_role_filter(current: Option<JobPreference>) -> Option<JobPreference> {
    match current {
        None => Some(JobPreference::Explorer),
        Some(JobPreference::Explorer) => Some(JobPreference::Builder),
        Some(JobPreference::Builder) => Some(JobPreference::Cook),
        Some(JobPreference::Cook) => Some(JobPreference::Hauler),
        Some(JobPreference::Hauler) | Some(JobPreference::None) => None,
    }
}

pub(super) fn apply_batch_home_pin(
    colonists: &mut [Colonist],
    selected_id: u32,
    habitat_id: u32,
    visible_indices: &[usize],
    capacity: u32,
) -> Vec<String> {
    let mut assigned_count = colonists
        .iter()
        .filter(|colonist| colonist.assigned_habitat == Some(habitat_id))
        .count() as u32;
    let mut assigned = Vec::new();

    for index in visible_indices {
        if assigned_count >= capacity {
            break;
        }

        let Some(colonist) = colonists.get_mut(*index) else {
            continue;
        };
        if colonist.id == selected_id || colonist.assigned_habitat == Some(habitat_id) {
            continue;
        }

        colonist.assigned_habitat = Some(habitat_id);
        assigned_count += 1;
        assigned.push(colonist.name.clone());
    }

    assigned
}

pub(super) fn apply_batch_work_pin(
    colonists: &mut [Colonist],
    selected_id: u32,
    workplace_id: u32,
    building_type: BuildingType,
    target_indices: &[usize],
) -> Vec<String> {
    let mut assigned = Vec::new();

    for index in target_indices {
        let Some(colonist) = colonists.get_mut(*index) else {
            continue;
        };
        if colonist.id == selected_id
            || colonist.assigned_workplace == Some(workplace_id)
            || colonist.job_preference.work_building_type() != building_type
        {
            continue;
        }

        colonist.assigned_workplace = Some(workplace_id);
        if matches!(
            colonist.state,
            ColonistState::Working | ColonistState::Moving { .. }
        ) {
            colonist.state = ColonistState::Idle;
            colonist.activity_location = ActivityLocation::None;
        }
        assigned.push(colonist.name.clone());
    }

    assigned
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BatchAssignmentScope {
    Page,
    All,
}

impl BatchAssignmentScope {
    fn label(self) -> &'static str {
        match self {
            BatchAssignmentScope::Page => "visible roster",
            BatchAssignmentScope::All => "all compatible survivors",
        }
    }
}

pub(super) fn batch_assignment_log(
    title: &'static str,
    source_name: &str,
    pin_prefix: &str,
    building_id: u32,
    scope: BatchAssignmentScope,
    assigned: Vec<String>,
) -> (String, String) {
    let detail = if assigned.is_empty() {
        format!(
            "{} had no compatible survivors in {} to copy {}#{} to.",
            source_name,
            scope.label(),
            pin_prefix,
            building_id
        )
    } else {
        format!(
            "Copied {}#{} from {} to {} in {}.",
            pin_prefix,
            building_id,
            source_name,
            truncate_text(&assigned.join(", "), 45),
            scope.label()
        )
    };

    (title.to_string(), detail)
}

pub(super) fn strongest_relationship_value(colonist: &Colonist) -> Option<i32> {
    colonist
        .relationships
        .values()
        .max_by_key(|value| value.abs())
        .copied()
}

pub(super) fn average_relationship_between(first: &Colonist, second: &Colonist) -> i32 {
    let first_value = first.relationships.get(&second.id).copied().unwrap_or(0);
    let second_value = second.relationships.get(&first.id).copied().unwrap_or(0);

    if first_value == 0 {
        second_value
    } else if second_value == 0 {
        first_value
    } else {
        (first_value + second_value) / 2
    }
}

pub(super) fn shared_social_location(first: &Colonist, second: &Colonist) -> bool {
    match (&first.activity_location, &second.activity_location) {
        (
            ActivityLocation::Building {
                building_id: first_id,
                ..
            },
            ActivityLocation::Building {
                building_id: second_id,
                ..
            },
        ) => first_id == second_id,
        (ActivityLocation::Ground(first_pos), ActivityLocation::Ground(second_pos)) => {
            first_pos == second_pos
        }
        _ => false,
    }
}

pub(super) fn social_color(value: i32, alpha: f32) -> Color {
    if value >= 10 {
        Color::new(
            style::BAR_GREEN.r,
            style::BAR_GREEN.g,
            style::BAR_GREEN.b,
            alpha,
        )
    } else if value <= -10 {
        Color::new(
            style::ALERT_RED.r,
            style::ALERT_RED.g,
            style::ALERT_RED.b,
            alpha,
        )
    } else {
        Color::new(
            style::TEXT_MUTED.r,
            style::TEXT_MUTED.g,
            style::TEXT_MUTED.b,
            alpha,
        )
    }
}
