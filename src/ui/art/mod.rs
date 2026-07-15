use macroquad::prelude::*;

mod portrait;
mod profiles;
mod sprite;

use portrait::generate_portrait;
use profiles::SURVIVOR_ART_PROFILES;
use sprite::generate_sprite;

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
    pub(super) const fn all() -> &'static [SpritePose] {
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
