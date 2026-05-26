use crate::data::colonist::{Colonist, ColonistState};
use crate::data::game_state::GameState;
use macroquad::prelude::*;

const TILE_SIZE: f32 = 32.0;

pub fn render_colonists(state: &GameState) {
    for colonist in &state.colonists {
        draw_colonist(colonist);
    }
}

fn draw_colonist(colonist: &Colonist) {
    // Use visual position for smooth interpolation
    let screen_x = colonist.visual_x;
    let screen_y = colonist.visual_y;

    // State-based body color
    let body_color = match colonist.state {
        ColonistState::Idle => BLUE,
        ColonistState::Moving { .. } => SKYBLUE,
        ColonistState::Working => ORANGE,
        ColonistState::Eating => GREEN,
        ColonistState::Sleeping => DARKBLUE,
    };

    // Draw body (simple circle)
    draw_circle(
        screen_x + TILE_SIZE / 2.0,
        screen_y + TILE_SIZE / 2.0,
        TILE_SIZE / 2.0 - 2.0,
        body_color,
    );

    // Draw name above head
    draw_text(&colonist.name, screen_x, screen_y - 5.0, 16.0, WHITE);

    // Draw state indicator text (small, below name)
    let state_str = match colonist.state {
        ColonistState::Idle => "Idle",
        ColonistState::Moving { .. } => "Moving",
        ColonistState::Working => "Working",
        ColonistState::Eating => "Eating",
        ColonistState::Sleeping => "Zzz",
    };
    draw_text(
        state_str,
        screen_x,
        screen_y + TILE_SIZE + 12.0,
        12.0,
        LIGHTGRAY,
    );

    // Draw job preference indicator (small colored dot)
    let job_color = match colonist.job_preference {
        crate::data::colonist::JobPreference::Explorer => PURPLE,
        crate::data::colonist::JobPreference::Builder => YELLOW,
        crate::data::colonist::JobPreference::Cook => GREEN,
        crate::data::colonist::JobPreference::Hauler => GRAY,
        crate::data::colonist::JobPreference::None => WHITE,
    };
    draw_circle(screen_x + TILE_SIZE - 5.0, screen_y + 5.0, 3.0, job_color);
}
