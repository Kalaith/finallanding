pub fn init_ui_font() {
    let _ = macroquad_toolkit::ui::ensure_default_ui_font();
}

#[cfg(test)]
mod tests {
    #[test]
    fn toolkit_bundled_font_is_available() {
        assert!(macroquad_toolkit::ui::builtin_rajdhani_semibold_font_bytes().len() > 100_000);
    }
}
