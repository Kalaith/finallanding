use super::*;

pub(crate) fn building_wall_height(building_type: BuildingType, tile_h: f32) -> f32 {
    let multiplier = match building_type {
        BuildingType::Habitat => 0.95,
        BuildingType::MessHall => 0.78,
        BuildingType::Workshop => 1.12,
        BuildingType::Storage => 0.64,
        BuildingType::ExplorationGate => 1.25,
    };
    tile_h * multiplier
}

pub(crate) fn building_outline_style(
    hovered: bool,
    assignment_color: Option<Color>,
) -> Option<(Color, f32)> {
    if hovered {
        Some((Color::new(0.92, 0.8, 0.45, 0.96), 3.0))
    } else {
        assignment_color.map(|color| (Color::new(color.r, color.g, color.b, 0.9), 2.0))
    }
}

pub(crate) fn assignment_marker_with_filter(
    assignment_marker: Option<(&'static str, Color)>,
    filter_match: bool,
) -> Option<(&'static str, Color)> {
    assignment_marker.or_else(|| filter_match.then_some(("FILTER", style::ACCENT_GOLD)))
}

pub(crate) fn building_outline_style_for_assign_filter(
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

pub(crate) fn building_shell_colors(building_type: BuildingType) -> (Color, Color, Color) {
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

pub(crate) fn draw_building_shell_detail(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_outline_style_prioritizes_hover_over_assignment() {
        let hovered = building_outline_style(true, Some(style::BAR_GREEN)).unwrap();
        let assigned = building_outline_style(false, Some(style::BAR_GREEN)).unwrap();

        assert_eq!(hovered.1, 3.0);
        assert_eq!(assigned.1, 2.0);
        assert!(hovered.0.r > assigned.0.r);
        assert!(building_outline_style(false, None).is_none());
    }

    #[test]
    fn test_assignment_marker_with_filter_adds_filter_without_replacing_assignment() {
        let filtered = assignment_marker_with_filter(None, true).unwrap();
        assert_eq!(filtered.0, "FILTER");
        assert_eq!(filtered.1.r, style::ACCENT_GOLD.r);
        assert_eq!(filtered.1.g, style::ACCENT_GOLD.g);
        assert_eq!(filtered.1.b, style::ACCENT_GOLD.b);

        let assigned = assignment_marker_with_filter(Some(("HOME", style::BAR_GREEN)), true)
            .expect("assignment marker should remain visible");
        assert_eq!(assigned.0, "HOME");
        assert_eq!(assigned.1.r, style::BAR_GREEN.r);
        assert!(assignment_marker_with_filter(None, false).is_none());
    }

    #[test]
    fn test_assign_filter_outline_uses_gold_room_highlight() {
        let filtered =
            building_outline_style_for_assign_filter(true, Some(style::BAR_GREEN), true).unwrap();
        assert_eq!(filtered.0.r, style::ACCENT_GOLD.r);
        assert_eq!(filtered.0.g, style::ACCENT_GOLD.g);
        assert_eq!(filtered.0.b, style::ACCENT_GOLD.b);
        assert_eq!(filtered.1, 3.0);

        let assigned =
            building_outline_style_for_assign_filter(false, Some(style::BAR_GREEN), false).unwrap();
        assert_eq!(assigned.1, 2.0);
        assert!(building_outline_style_for_assign_filter(false, None, false).is_none());
    }
}
