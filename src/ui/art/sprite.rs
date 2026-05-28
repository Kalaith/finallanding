use super::pixels::*;
use super::profiles::SurvivorArtProfile;
use super::SpritePose;
use macroquad::prelude::{Color, Image};

const SPRITE_WIDTH: u16 = 32;
const SPRITE_HEIGHT: u16 = 64;

pub(super) fn generate_sprite(
    profile: SurvivorArtProfile,
    index: usize,
    pose: SpritePose,
) -> Image {
    let mut image = Image::gen_image_color(SPRITE_WIDTH, SPRITE_HEIGHT, transparent());

    if pose == SpritePose::Sleeping {
        fill_ellipse(&mut image, 16, 55, 12, 4, Color::new(0.0, 0.0, 0.0, 0.28));
        fill_ellipse(&mut image, 17, 43, 13, 7, profile.suit);
        fill_rect(
            &mut image,
            7,
            41,
            20,
            7,
            Color::new(profile.accent.r, profile.accent.g, profile.accent.b, 0.8),
        );
        fill_circle(&mut image, 10, 37, 6, profile.skin);
        fill_ellipse(&mut image, 9, 34, 7, 4, profile.hair);
        fill_rect(&mut image, 18, 44, 8, 3, Color::new(0.12, 0.13, 0.13, 1.0));
        return image;
    }

    let moving_offset = if pose == SpritePose::Moving {
        index as i32 % 3 - 1
    } else {
        0
    };
    fill_ellipse(
        &mut image,
        16,
        57,
        if pose == SpritePose::Moving { 12 } else { 10 },
        4,
        Color::new(0.0, 0.0, 0.0, 0.28),
    );
    let torso_x = if matches!(pose, SpritePose::Tense | SpritePose::TenseGuarded) {
        12
    } else {
        11
    };
    let torso_w = if matches!(pose, SpritePose::Tense | SpritePose::TenseGuarded) {
        9
    } else {
        10
    };
    fill_rect(&mut image, torso_x, 28, torso_w, 21, profile.suit);
    fill_rect(
        &mut image,
        12,
        28,
        8,
        3,
        Color::new(profile.accent.r, profile.accent.g, profile.accent.b, 0.95),
    );
    fill_circle(&mut image, 16, 19, 8, profile.skin);
    fill_ellipse(&mut image, 16, 14, 9, 5, profile.hair);
    fill_rect(&mut image, 10, 19, 3, 7, profile.hair);
    fill_rect(&mut image, 20, 19, 3, 7, profile.hair);

    match pose {
        SpritePose::Idle => {
            draw_line_pixels(&mut image, 11, 31, 6 + index as i32 % 2, 42, profile.suit);
            draw_line_pixels(&mut image, 21, 31, 26 - index as i32 % 2, 42, profile.suit);
        }
        SpritePose::Moving => {
            draw_line_pixels(&mut image, 11, 31, 5, 38 + moving_offset, profile.suit);
            draw_line_pixels(&mut image, 21, 31, 27, 45 - moving_offset, profile.suit);
        }
        SpritePose::Working => {
            draw_line_pixels(&mut image, 11, 31, 7, 39, profile.suit);
            draw_line_pixels(&mut image, 21, 31, 26, 36, profile.suit);
            draw_line_pixels(&mut image, 24, 34, 29, 45, Color::new(0.7, 0.6, 0.38, 1.0));
            fill_rect(&mut image, 27, 44, 4, 3, profile.accent);
        }
        SpritePose::Eating => {
            draw_line_pixels(&mut image, 11, 31, 9, 38, profile.suit);
            draw_line_pixels(&mut image, 21, 31, 19, 26, profile.suit);
            fill_circle(&mut image, 20, 25, 2, profile.accent);
            fill_rect(&mut image, 10, 43, 12, 3, Color::new(0.12, 0.1, 0.07, 1.0));
        }
        SpritePose::Supported => {
            draw_line_pixels(&mut image, 11, 31, 5, 35, profile.suit);
            draw_line_pixels(&mut image, 21, 31, 27, 35, profile.suit);
            fill_circle(&mut image, 5, 35, 2, profile.accent);
            fill_circle(&mut image, 27, 35, 2, profile.accent);
            draw_line_pixels(&mut image, 12, 25, 20, 25, profile.accent);
        }
        SpritePose::SupportedReach => {
            draw_line_pixels(&mut image, 11, 31, 4, 31, profile.suit);
            draw_line_pixels(&mut image, 21, 31, 28, 29, profile.suit);
            fill_circle(&mut image, 4, 31, 2, profile.accent);
            fill_circle(&mut image, 28, 29, 2, profile.accent);
            draw_line_pixels(&mut image, 11, 24, 21, 23, profile.accent);
            fill_circle(
                &mut image,
                23,
                23,
                2,
                Color::new(profile.accent.r, profile.accent.g, profile.accent.b, 0.88),
            );
        }
        SpritePose::Tense => {
            draw_line_pixels(&mut image, 12, 31, 10, 40, profile.suit);
            draw_line_pixels(&mut image, 20, 31, 22, 40, profile.suit);
            fill_rect(&mut image, 10, 25, 12, 2, profile.hair);
            fill_rect(&mut image, 12, 36, 9, 2, Color::new(0.10, 0.08, 0.08, 1.0));
        }
        SpritePose::TenseGuarded => {
            draw_line_pixels(&mut image, 12, 31, 18, 37, profile.suit);
            draw_line_pixels(&mut image, 20, 31, 12, 38, profile.suit);
            fill_rect(&mut image, 10, 24, 12, 3, profile.hair);
            fill_rect(&mut image, 11, 36, 10, 2, Color::new(0.10, 0.08, 0.08, 1.0));
            fill_rect(&mut image, 12, 40, 9, 2, darken_color(profile.suit, 0.2));
        }
        SpritePose::Sleeping => {}
    }

    let left_foot = match pose {
        SpritePose::Moving => (8, 58),
        SpritePose::Working => (10, 57),
        SpritePose::Eating => (10, 58),
        SpritePose::Supported => (9, 58),
        SpritePose::SupportedReach => (8, 58),
        SpritePose::Tense => (12, 58),
        SpritePose::TenseGuarded => (12, 58),
        _ => (10, 58),
    };
    let right_foot = match pose {
        SpritePose::Moving => (25, 58),
        SpritePose::Working => (22, 57),
        SpritePose::Eating => (22, 58),
        SpritePose::Supported => (23, 58),
        SpritePose::SupportedReach => (24, 58),
        SpritePose::Tense => (20, 58),
        SpritePose::TenseGuarded => (20, 58),
        _ => (22, 58),
    };
    draw_line_pixels(
        &mut image,
        13,
        49,
        left_foot.0,
        left_foot.1,
        Color::new(0.12, 0.13, 0.13, 1.0),
    );
    draw_line_pixels(
        &mut image,
        19,
        49,
        right_foot.0,
        right_foot.1,
        Color::new(0.12, 0.13, 0.13, 1.0),
    );
    fill_rect(&mut image, 8, 58, 7, 2, Color::new(0.06, 0.065, 0.065, 1.0));
    fill_rect(
        &mut image,
        19,
        58,
        7,
        2,
        Color::new(0.06, 0.065, 0.065, 1.0),
    );
    fill_rect(
        &mut image,
        14,
        36,
        4,
        3,
        Color::new(profile.accent.r, profile.accent.g, profile.accent.b, 0.9),
    );
    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::art::profiles::SURVIVOR_ART_PROFILES;

    #[test]
    fn test_sprite_generation_preserves_transparent_edges() {
        let image = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Idle);

        assert_eq!(image.width, SPRITE_WIDTH);
        assert_eq!(image.height, SPRITE_HEIGHT);
        assert_eq!(image.get_pixel(0, 0).a, 0.0);
        assert!(image.get_pixel(16, 36).a >= 0.89);
    }

    #[test]
    fn test_sprite_pose_generation_changes_body_language() {
        let idle = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Idle);
        let working = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Working);
        let sleeping = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Sleeping);
        let supported = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Supported);
        let supported_reach =
            generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::SupportedReach);
        let tense = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Tense);
        let tense_guarded = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::TenseGuarded);

        assert_ne!(idle.get_pixel(29, 45), working.get_pixel(29, 45));
        assert!(sleeping.get_pixel(17, 43).a > 0.7);
        assert_eq!(sleeping.get_pixel(16, 19).a, 0.0);
        assert_ne!(idle.get_pixel(5, 35), supported.get_pixel(5, 35));
        assert_ne!(
            supported.get_pixel(23, 23),
            supported_reach.get_pixel(23, 23)
        );
        assert_ne!(idle.get_pixel(12, 36), tense.get_pixel(12, 36));
        assert_ne!(tense.get_pixel(12, 40), tense_guarded.get_pixel(12, 40));
    }
}
