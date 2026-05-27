use macroquad::prelude::*;

const PORTRAIT_SIZE: u16 = 128;
const PORTRAIT_SCALE: i32 = PORTRAIT_SIZE as i32 / 64;
const SPRITE_WIDTH: u16 = 32;
const SPRITE_HEIGHT: u16 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpritePose {
    Idle,
    Moving,
    Working,
    Eating,
    Sleeping,
    Supported,
    SupportedReach,
    Tense,
    TenseGuarded,
}

impl SpritePose {
    const fn all() -> &'static [SpritePose] {
        &[
            SpritePose::Idle,
            SpritePose::Moving,
            SpritePose::Working,
            SpritePose::Eating,
            SpritePose::Sleeping,
            SpritePose::Supported,
            SpritePose::SupportedReach,
            SpritePose::Tense,
            SpritePose::TenseGuarded,
        ]
    }

    const fn index(self) -> usize {
        match self {
            SpritePose::Idle => 0,
            SpritePose::Moving => 1,
            SpritePose::Working => 2,
            SpritePose::Eating => 3,
            SpritePose::Sleeping => 4,
            SpritePose::Supported => 5,
            SpritePose::SupportedReach => 6,
            SpritePose::Tense => 7,
            SpritePose::TenseGuarded => 8,
        }
    }
}

#[derive(Clone, Copy)]
struct SurvivorArtProfile {
    skin: Color,
    hair: Color,
    suit: Color,
    accent: Color,
}

const SURVIVOR_ART_PROFILES: &[SurvivorArtProfile] = &[
    SurvivorArtProfile {
        skin: Color::new(0.67, 0.47, 0.33, 1.0),
        hair: Color::new(0.08, 0.06, 0.05, 1.0),
        suit: Color::new(0.28, 0.34, 0.36, 1.0),
        accent: Color::new(0.90, 0.68, 0.28, 1.0),
    },
    SurvivorArtProfile {
        skin: Color::new(0.80, 0.62, 0.43, 1.0),
        hair: Color::new(0.17, 0.09, 0.05, 1.0),
        suit: Color::new(0.22, 0.30, 0.42, 1.0),
        accent: Color::new(0.55, 0.72, 0.75, 1.0),
    },
    SurvivorArtProfile {
        skin: Color::new(0.58, 0.38, 0.27, 1.0),
        hair: Color::new(0.03, 0.03, 0.03, 1.0),
        suit: Color::new(0.33, 0.28, 0.22, 1.0),
        accent: Color::new(0.60, 0.72, 0.38, 1.0),
    },
    SurvivorArtProfile {
        skin: Color::new(0.86, 0.70, 0.54, 1.0),
        hair: Color::new(0.48, 0.32, 0.12, 1.0),
        suit: Color::new(0.24, 0.25, 0.27, 1.0),
        accent: Color::new(0.73, 0.45, 0.25, 1.0),
    },
    SurvivorArtProfile {
        skin: Color::new(0.74, 0.53, 0.39, 1.0),
        hair: Color::new(0.12, 0.11, 0.10, 1.0),
        suit: Color::new(0.18, 0.32, 0.29, 1.0),
        accent: Color::new(0.35, 0.55, 0.70, 1.0),
    },
    SurvivorArtProfile {
        skin: Color::new(0.91, 0.74, 0.58, 1.0),
        hair: Color::new(0.68, 0.56, 0.32, 1.0),
        suit: Color::new(0.35, 0.31, 0.38, 1.0),
        accent: Color::new(0.72, 0.40, 0.48, 1.0),
    },
];

pub struct PlaceholderArt {
    colonist_sprites: Vec<Texture2D>,
    colonist_portraits: Vec<Texture2D>,
}

impl PlaceholderArt {
    pub fn new() -> Self {
        let colonist_sprites = SURVIVOR_ART_PROFILES
            .iter()
            .enumerate()
            .flat_map(|(index, profile)| {
                SpritePose::all().iter().map(move |pose| {
                    texture_from_image(generate_sprite(*profile, index, *pose), FilterMode::Nearest)
                })
            })
            .collect();

        let colonist_portraits = SURVIVOR_ART_PROFILES
            .iter()
            .enumerate()
            .map(|(index, profile)| {
                texture_from_image(generate_portrait(*profile, index), FilterMode::Linear)
            })
            .collect();

        Self {
            colonist_sprites,
            colonist_portraits,
        }
    }

    pub fn colonist_sprite(&self, colonist_id: u32) -> Option<&Texture2D> {
        self.colonist_sprite_for_pose(colonist_id, SpritePose::Idle)
    }

    pub fn colonist_sprite_for_pose(
        &self,
        colonist_id: u32,
        pose: SpritePose,
    ) -> Option<&Texture2D> {
        if self.colonist_sprites.is_empty() {
            return None;
        }

        let pose_count = SpritePose::all().len();
        let profile_index = colonist_id as usize % SURVIVOR_ART_PROFILES.len();
        self.colonist_sprites
            .get(profile_index * pose_count + pose.index())
    }

    pub fn colonist_portrait(&self, colonist_id: u32) -> Option<&Texture2D> {
        if self.colonist_portraits.is_empty() {
            return None;
        }

        self.colonist_portraits
            .get(colonist_id as usize % self.colonist_portraits.len())
    }
}

fn texture_from_image(image: Image, filter: FilterMode) -> Texture2D {
    let texture = Texture2D::from_image(&image);
    texture.set_filter(filter);
    texture
}

fn generate_portrait(profile: SurvivorArtProfile, index: usize) -> Image {
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

fn generate_sprite(profile: SurvivorArtProfile, index: usize, pose: SpritePose) -> Image {
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

fn fill_rect(image: &mut Image, x: i32, y: i32, width: i32, height: i32, color: Color) {
    for yy in y..(y + height) {
        for xx in x..(x + width) {
            set_pixel_safe(image, xx, yy, color);
        }
    }
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

fn fill_circle(image: &mut Image, cx: i32, cy: i32, radius: i32, color: Color) {
    let radius_sq = radius * radius;
    for yy in (cy - radius)..=(cy + radius) {
        for xx in (cx - radius)..=(cx + radius) {
            let dx = xx - cx;
            let dy = yy - cy;
            if dx * dx + dy * dy <= radius_sq {
                set_pixel_safe(image, xx, yy, color);
            }
        }
    }
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

fn fill_ellipse(image: &mut Image, cx: i32, cy: i32, rx: i32, ry: i32, color: Color) {
    let rx_sq = (rx * rx).max(1);
    let ry_sq = (ry * ry).max(1);
    for yy in (cy - ry)..=(cy + ry) {
        for xx in (cx - rx)..=(cx + rx) {
            let dx = xx - cx;
            let dy = yy - cy;
            if dx * dx * ry_sq + dy * dy * rx_sq <= rx_sq * ry_sq {
                set_pixel_safe(image, xx, yy, color);
            }
        }
    }
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

fn draw_line_pixels(image: &mut Image, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        set_pixel_safe(image, x, y, color);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            error += dy;
            x += sx;
        }
        if e2 <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn add_noise(image: &mut Image, seed: u32, amount: f32) {
    for y in 0..image.height {
        for x in 0..image.width {
            let current = image.get_pixel(x as u32, y as u32);
            if current.a <= 0.0 {
                continue;
            }

            let value = noise_value(x as u32, y as u32, seed);
            let delta = (value as f32 / 255.0 - 0.5) * amount;
            image.set_pixel(
                x as u32,
                y as u32,
                Color::new(
                    (current.r + delta).clamp(0.0, 1.0),
                    (current.g + delta).clamp(0.0, 1.0),
                    (current.b + delta).clamp(0.0, 1.0),
                    current.a,
                ),
            );
        }
    }
}

fn set_pixel_safe(image: &mut Image, x: i32, y: i32, color: Color) {
    if x < 0 || y < 0 || x >= image.width as i32 || y >= image.height as i32 {
        return;
    }

    image.set_pixel(x as u32, y as u32, color);
}

fn transparent() -> Color {
    Color::new(0.0, 0.0, 0.0, 0.0)
}

fn mix_color(base: Color, top: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color::new(
        base.r + (top.r - base.r) * amount,
        base.g + (top.g - base.g) * amount,
        base.b + (top.b - base.b) * amount,
        base.a + (top.a - base.a) * amount,
    )
}

fn darken_color(color: Color, amount: f32) -> Color {
    Color::new(
        (color.r * (1.0 - amount)).clamp(0.0, 1.0),
        (color.g * (1.0 - amount)).clamp(0.0, 1.0),
        (color.b * (1.0 - amount)).clamp(0.0, 1.0),
        color.a,
    )
}

fn lighten_color(color: Color, amount: f32) -> Color {
    Color::new(
        (color.r + (1.0 - color.r) * amount).clamp(0.0, 1.0),
        (color.g + (1.0 - color.g) * amount).clamp(0.0, 1.0),
        (color.b + (1.0 - color.b) * amount).clamp(0.0, 1.0),
        color.a,
    )
}

fn noise_value(x: u32, y: u32, seed: u32) -> u8 {
    let mixed =
        x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ seed.wrapping_mul(83_492_791);
    (mixed & 0xFF) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_art_has_required_survivor_slots() {
        assert_eq!(SURVIVOR_ART_PROFILES.len(), 6);
    }

    #[test]
    fn test_portrait_generation_uses_opaque_pixels() {
        let image = generate_portrait(SURVIVOR_ART_PROFILES[0], 0);

        assert!(PORTRAIT_SIZE >= 128);
        assert_eq!(image.width, PORTRAIT_SIZE);
        assert_eq!(image.height, PORTRAIT_SIZE);
        assert!(image.get_pixel(64, 56).a > 0.9);
        assert_ne!(image.get_pixel(44, 94), image.get_pixel(80, 94));
    }

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
