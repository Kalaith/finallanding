use macroquad::prelude::*;

const PORTRAIT_SIZE: u16 = 64;
const SPRITE_WIDTH: u16 = 32;
const SPRITE_HEIGHT: u16 = 64;

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
            .map(|(index, profile)| texture_from_image(generate_sprite(*profile, index)))
            .collect();

        let colonist_portraits = SURVIVOR_ART_PROFILES
            .iter()
            .enumerate()
            .map(|(index, profile)| texture_from_image(generate_portrait(*profile, index)))
            .collect();

        Self {
            colonist_sprites,
            colonist_portraits,
        }
    }

    pub fn colonist_sprite(&self, colonist_id: u32) -> Option<&Texture2D> {
        if self.colonist_sprites.is_empty() {
            return None;
        }

        self.colonist_sprites
            .get(colonist_id as usize % self.colonist_sprites.len())
    }

    pub fn colonist_portrait(&self, colonist_id: u32) -> Option<&Texture2D> {
        if self.colonist_portraits.is_empty() {
            return None;
        }

        self.colonist_portraits
            .get(colonist_id as usize % self.colonist_portraits.len())
    }
}

fn texture_from_image(image: Image) -> Texture2D {
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
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
    fill_rect(
        &mut image,
        0,
        0,
        PORTRAIT_SIZE as i32,
        16,
        Color::new(0.08, 0.11, 0.12, 1.0),
    );
    fill_circle(
        &mut image,
        32,
        34,
        28,
        Color::new(profile.accent.r, profile.accent.g, profile.accent.b, 0.16),
    );
    fill_rect(&mut image, 18, 46, 28, 12, profile.suit);
    fill_rect(&mut image, 24, 41, 16, 8, profile.skin);
    fill_circle(&mut image, 32, 28, 15, profile.skin);
    fill_ellipse(&mut image, 32, 20, 17, 10, profile.hair);
    fill_rect(&mut image, 19, 23, 6, 10, profile.hair);
    fill_rect(&mut image, 39, 23, 6, 10, profile.hair);
    fill_rect(
        &mut image,
        22,
        47,
        20,
        3,
        Color::new(profile.accent.r, profile.accent.g, profile.accent.b, 0.95),
    );

    let eye_color = Color::new(0.04, 0.05, 0.05, 1.0);
    fill_rect(&mut image, 26, 28, 3, 2, eye_color);
    fill_rect(&mut image, 36, 28, 3, 2, eye_color);
    fill_rect(&mut image, 30, 36, 7, 1, Color::new(0.22, 0.12, 0.10, 0.85));

    if index % 2 == 0 {
        fill_rect(&mut image, 22, 17, 20, 3, profile.hair);
    } else {
        fill_rect(&mut image, 21, 15, 23, 4, profile.hair);
        fill_rect(&mut image, 39, 18, 5, 10, profile.hair);
    }

    add_noise(&mut image, index as u32, 0.035);
    image
}

fn generate_sprite(profile: SurvivorArtProfile, index: usize) -> Image {
    let mut image = Image::gen_image_color(SPRITE_WIDTH, SPRITE_HEIGHT, transparent());
    fill_ellipse(&mut image, 16, 57, 10, 4, Color::new(0.0, 0.0, 0.0, 0.28));
    fill_rect(&mut image, 11, 28, 10, 21, profile.suit);
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
    draw_line_pixels(&mut image, 11, 31, 6 + index as i32 % 2, 42, profile.suit);
    draw_line_pixels(&mut image, 21, 31, 26 - index as i32 % 2, 42, profile.suit);
    draw_line_pixels(
        &mut image,
        13,
        49,
        10,
        58,
        Color::new(0.12, 0.13, 0.13, 1.0),
    );
    draw_line_pixels(
        &mut image,
        19,
        49,
        22,
        58,
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

        assert_eq!(image.width, PORTRAIT_SIZE);
        assert_eq!(image.height, PORTRAIT_SIZE);
        assert!(image.get_pixel(32, 28).a > 0.9);
    }

    #[test]
    fn test_sprite_generation_preserves_transparent_edges() {
        let image = generate_sprite(SURVIVOR_ART_PROFILES[0], 0);

        assert_eq!(image.width, SPRITE_WIDTH);
        assert_eq!(image.height, SPRITE_HEIGHT);
        assert_eq!(image.get_pixel(0, 0).a, 0.0);
        assert!(image.get_pixel(16, 36).a >= 0.89);
    }
}
