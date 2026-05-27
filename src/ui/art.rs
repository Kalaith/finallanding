use macroquad::prelude::*;

pub struct PlaceholderArt {
    colonist_sprites: Vec<Texture2D>,
}

impl PlaceholderArt {
    pub fn new() -> Self {
        let colonist_sprites = [
            include_bytes!("../../docs/reference/tfl_guide_mvp/sprite_charlie_world.png")
                .as_slice(),
            include_bytes!("../../docs/reference/tfl_guide_mvp/sprite_ilya_world.png").as_slice(),
            include_bytes!("../../docs/reference/tfl_guide_mvp/sprite_eva_world.png").as_slice(),
            include_bytes!("../../docs/reference/tfl_guide_mvp/sprite_gate_worker_world.png")
                .as_slice(),
            include_bytes!("../../docs/reference/tfl_guide_mvp/sprite_habitat_eva_world.png")
                .as_slice(),
            include_bytes!("../../docs/reference/tfl_guide_mvp/sprite_mess_eli_world.png")
                .as_slice(),
            include_bytes!("../../docs/reference/tfl_guide_mvp/sprite_workshop_worker_world.png")
                .as_slice(),
        ]
        .iter()
        .map(|bytes| {
            let texture = Texture2D::from_file_with_format(bytes, None);
            texture.set_filter(FilterMode::Nearest);
            texture
        })
        .collect();

        Self { colonist_sprites }
    }

    pub fn colonist_sprite(&self, colonist_id: u32) -> Option<&Texture2D> {
        if self.colonist_sprites.is_empty() {
            return None;
        }

        self.colonist_sprites
            .get(colonist_id as usize % self.colonist_sprites.len())
    }
}
