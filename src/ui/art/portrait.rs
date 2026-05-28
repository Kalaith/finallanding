use super::pixels::*;
use super::profiles::SurvivorArtProfile;
use macroquad::prelude::{Color, Image};

const PORTRAIT_SIZE: u16 = 128;
const PORTRAIT_SCALE: i32 = PORTRAIT_SIZE as i32 / 64;

pub(super) fn generate_portrait(profile: SurvivorArtProfile, index: usize) -> Image {
    let mut image = Image::gen_image_color(PORTRAIT_SIZE, PORTRAIT_SIZE, transparent());
    let bg = Color::new(0.05, 0.065, 0.065, 1.0);
    fill_rect(
        &mut image,
        0,
        0,
        PORTRAIT_SIZE as i32,
        PORTRAIT_SIZE as i32,
        bg,
    );
    fill_rect_scaled(&mut image, 0, 0, 64, 16, Color::new(0.08, 0.11, 0.12, 1.0));
    for offset in [-28, 2, 32] {
        draw_line_pixels(
            &mut image,
            offset * PORTRAIT_SCALE,
            63 * PORTRAIT_SCALE,
            (offset + 36) * PORTRAIT_SCALE,
            0,
            Color::new(0.12, 0.16, 0.17, 1.0),
        );
    }
    fill_circle_scaled(&mut image, 32, 34, 28, mix_color(bg, profile.accent, 0.18));
    fill_rect_scaled(&mut image, 14, 48, 36, 12, darken_color(profile.suit, 0.24));
    fill_rect_scaled(&mut image, 18, 44, 28, 14, profile.suit);
    fill_rect_scaled(&mut image, 22, 44, 6, 15, darken_color(profile.suit, 0.16));
    fill_rect_scaled(&mut image, 37, 44, 6, 15, darken_color(profile.suit, 0.16));
    fill_rect_scaled(&mut image, 25, 40, 14, 9, profile.skin);
    fill_circle_scaled(&mut image, 22, 30, 4, darken_color(profile.skin, 0.12));
    fill_circle_scaled(&mut image, 42, 30, 4, darken_color(profile.skin, 0.12));
    fill_circle_scaled(&mut image, 32, 28, 15, profile.skin);
    fill_rect_scaled(&mut image, 22, 33, 20, 7, darken_color(profile.skin, 0.08));
    fill_ellipse_scaled(&mut image, 32, 20, 17, 10, profile.hair);
    fill_rect_scaled(&mut image, 19, 23, 6, 10, profile.hair);
    fill_rect_scaled(&mut image, 39, 23, 6, 10, profile.hair);
    fill_rect_scaled(
        &mut image,
        22,
        47,
        20,
        3,
        Color::new(profile.accent.r, profile.accent.g, profile.accent.b, 0.95),
    );
    fill_rect_scaled(&mut image, 20, 52, 5, 2, profile.accent);
    fill_rect_scaled(&mut image, 39, 52, 5, 2, profile.accent);

    let eye_color = Color::new(0.04, 0.05, 0.05, 1.0);
    fill_rect_scaled(&mut image, 26, 28, 3, 2, eye_color);
    fill_rect_scaled(&mut image, 36, 28, 3, 2, eye_color);
    fill_rect_scaled(&mut image, 29, 31, 2, 4, darken_color(profile.skin, 0.18));
    fill_rect_scaled(&mut image, 30, 36, 7, 1, Color::new(0.22, 0.12, 0.10, 0.85));
    fill_rect_scaled(&mut image, 23, 26, 7, 1, darken_color(profile.hair, 0.1));
    fill_rect_scaled(&mut image, 35, 26, 7, 1, darken_color(profile.hair, 0.1));

    if index % 2 == 0 {
        fill_rect_scaled(&mut image, 22, 17, 20, 3, profile.hair);
        fill_rect_scaled(&mut image, 25, 14, 14, 2, lighten_color(profile.hair, 0.12));
    } else {
        fill_rect_scaled(&mut image, 21, 15, 23, 4, profile.hair);
        fill_rect_scaled(&mut image, 39, 18, 5, 10, profile.hair);
        fill_rect_scaled(&mut image, 24, 13, 13, 2, lighten_color(profile.hair, 0.12));
    }
    fill_rect_scaled(
        &mut image,
        14,
        60,
        36,
        1,
        mix_color(bg, profile.accent, 0.44),
    );

    add_noise(&mut image, index as u32, 0.035);
    image
}

fn fill_rect_scaled(image: &mut Image, x: i32, y: i32, width: i32, height: i32, color: Color) {
    fill_rect(
        image,
        x * PORTRAIT_SCALE,
        y * PORTRAIT_SCALE,
        width * PORTRAIT_SCALE,
        height * PORTRAIT_SCALE,
        color,
    );
}

fn fill_circle_scaled(image: &mut Image, cx: i32, cy: i32, radius: i32, color: Color) {
    fill_circle(
        image,
        cx * PORTRAIT_SCALE,
        cy * PORTRAIT_SCALE,
        radius * PORTRAIT_SCALE,
        color,
    );
}

fn fill_ellipse_scaled(image: &mut Image, cx: i32, cy: i32, rx: i32, ry: i32, color: Color) {
    fill_ellipse(
        image,
        cx * PORTRAIT_SCALE,
        cy * PORTRAIT_SCALE,
        rx * PORTRAIT_SCALE,
        ry * PORTRAIT_SCALE,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::art::profiles::SURVIVOR_ART_PROFILES;

    #[test]
    fn test_portrait_generation_uses_opaque_pixels() {
        let image = generate_portrait(SURVIVOR_ART_PROFILES[0], 0);

        assert!(PORTRAIT_SIZE >= 128);
        assert_eq!(image.width, PORTRAIT_SIZE);
        assert_eq!(image.height, PORTRAIT_SIZE);
        assert!(image.get_pixel(64, 56).a > 0.9);
        assert_ne!(image.get_pixel(44, 94), image.get_pixel(80, 94));
    }
}
