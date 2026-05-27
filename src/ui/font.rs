use macroquad::prelude::{
    draw_text_ex, load_ttf_font_from_bytes, measure_text as macroquad_measure_text, Color, Font,
    TextDimensions, TextParams,
};
use std::cell::RefCell;

const RAJDHANI_SEMIBOLD_BYTES: &[u8] = include_bytes!("../../assets/fonts/Rajdhani-SemiBold.ttf");

thread_local! {
    static UI_FONT: RefCell<Option<Font>> = const { RefCell::new(None) };
}

pub fn init_ui_font() {
    let Ok(font) = load_ttf_font_from_bytes(RAJDHANI_SEMIBOLD_BYTES) else {
        return;
    };

    UI_FONT.with(|slot| {
        *slot.borrow_mut() = Some(font);
    });
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
    UI_FONT.with(|slot| {
        let font = slot.borrow();
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: font.as_ref(),
                font_size: font_size.round() as u16,
                color,
                ..Default::default()
            },
        )
    })
}

pub fn measure_text(
    text: &str,
    font: Option<&Font>,
    font_size: u16,
    font_scale: f32,
) -> TextDimensions {
    UI_FONT.with(|slot| {
        let fallback_font = slot.borrow();
        macroquad_measure_text(text, font.or(fallback_font.as_ref()), font_size, font_scale)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundled_font_is_embedded() {
        assert!(RAJDHANI_SEMIBOLD_BYTES.len() > 100_000);
    }
}
