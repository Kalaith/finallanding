use super::*;

pub(crate) fn job_color(job_preference: crate::data::colonist::JobPreference) -> Color {
    match job_preference {
        crate::data::colonist::JobPreference::Explorer => PURPLE,
        crate::data::colonist::JobPreference::Builder => YELLOW,
        crate::data::colonist::JobPreference::Cook => GREEN,
        crate::data::colonist::JobPreference::Hauler => GRAY,
        crate::data::colonist::JobPreference::None => WHITE,
    }
}

pub(crate) fn colonist_activity_summary(colonist: &Colonist) -> &'static str {
    match colonist.state {
        ColonistState::Idle => "Idle",
        ColonistState::Moving { .. } => "Moving",
        ColonistState::Working => "Working",
        ColonistState::Eating => "Eating",
        ColonistState::Sleeping => "Resting",
        ColonistState::OnMission { .. } => "On mission",
    }
}

pub(crate) fn sprite_pose_for_state(state: ColonistState) -> SpritePose {
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
pub(crate) enum SocialBodyLanguage {
    Supported(i32),
    Tense(i32),
}

impl SocialBodyLanguage {
    pub(crate) fn intensity(self) -> i32 {
        match self {
            SocialBodyLanguage::Supported(value) | SocialBodyLanguage::Tense(value) => value.abs(),
        }
    }

    pub(crate) fn color(self, alpha: f32) -> Color {
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

    pub(crate) fn symbol(self) -> &'static str {
        match self {
            SocialBodyLanguage::Supported(_) => "+",
            SocialBodyLanguage::Tense(_) => "!",
        }
    }
}

pub(crate) fn sprite_pose_for_colonist_frame(
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

pub(crate) fn social_pose_uses_alternate_frame(tick: u64) -> bool {
    (tick / 45) % 2 == 1
}

pub(crate) fn strongest_relationship_value(colonist: &Colonist) -> Option<i32> {
    colonist
        .relationships
        .values()
        .max_by_key(|value| value.abs())
        .copied()
}

pub(crate) fn social_color(value: i32, alpha: f32) -> Color {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::colonist::{JobPreference, Trait};
    use crate::data::types::Position;

    #[test]
    fn test_strongest_relationship_value_uses_absolute_pressure() {
        let mut colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );
        colonist.relationships.insert(2, 26);
        colonist.relationships.insert(3, -31);

        assert_eq!(strongest_relationship_value(&colonist), Some(-31));
    }

    #[test]
    fn test_sprite_pose_tracks_colonist_state() {
        assert_eq!(sprite_pose_for_state(ColonistState::Idle), SpritePose::Idle);
        assert_eq!(
            sprite_pose_for_state(ColonistState::Moving {
                target: Position::new(1, 1)
            }),
            SpritePose::Moving
        );
        assert_eq!(
            sprite_pose_for_state(ColonistState::Working),
            SpritePose::Working
        );
        assert_eq!(
            sprite_pose_for_state(ColonistState::Eating),
            SpritePose::Eating
        );
        assert_eq!(
            sprite_pose_for_state(ColonistState::Sleeping),
            SpritePose::Sleeping
        );
    }

    #[test]
    fn test_social_body_language_overrides_idle_pose() {
        let colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );

        assert_eq!(
            sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 0),
            SpritePose::Tense
        );
        assert_eq!(
            sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Supported(28)), 0),
            SpritePose::Supported
        );
    }

    #[test]
    fn test_social_body_language_cycles_alternate_pose_frames() {
        let colonist = Colonist::new(
            1,
            "Alice".to_string(),
            Position::new(0, 0),
            Trait::HardWorker,
            JobPreference::Builder,
        );

        assert_eq!(
            sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Supported(28)), 45),
            SpritePose::SupportedReach
        );
        assert_eq!(
            sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 45),
            SpritePose::TenseGuarded
        );
        assert_eq!(
            sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 90),
            SpritePose::Tense
        );
    }
}
