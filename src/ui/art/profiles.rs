use macroquad::prelude::Color;

#[derive(Clone, Copy)]
pub(super) struct SurvivorArtProfile {
    pub(super) skin: Color,
    pub(super) hair: Color,
    pub(super) suit: Color,
    pub(super) accent: Color,
}

pub(super) const SURVIVOR_ART_PROFILES: &[SurvivorArtProfile] = &[
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_art_has_required_survivor_slots() {
        assert_eq!(SURVIVOR_ART_PROFILES.len(), 6);
    }
}
