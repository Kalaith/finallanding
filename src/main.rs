#![allow(clippy::large_enum_variant, clippy::too_many_arguments)]

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod data;
mod game;
mod state;
mod systems;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    // Built by hand (not capture::capture_window_conf) to keep TFL_FULLSCREEN
    // support; high_dpi stays at its false default, so captures are already
    // pixel-aligned.
    Conf {
        window_title: "The Final Landing".to_owned(),
        window_width: capture::env_i32("TFL_WINDOW_WIDTH", 1280),
        window_height: capture::env_i32("TFL_WINDOW_HEIGHT", 720),
        window_resizable: true,
        fullscreen: capture::env_bool("TFL_FULLSCREEN", false),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    ui::font::init_ui_font();
    if export_playthrough_report_if_requested() {
        return;
    }

    let mut game: Game = Game::new().await;

    // Screenshot harness: when TFL_CAPTURE_PATH is set, render deterministic
    // frames, write a PNG, and exit. Scenes are seeded via the TFL_START_* /
    // TFL_SEED_* env vars (see state::game_state_setup), not the scene field.
    if let Some(mut config) = capture::CaptureConfig::from_env("TFL") {
        config.frames = capture::env_u32("TFL_CAPTURE_FRAMES", 8).max(1);
        capture::run_capture(&config, |_dt| {
            clear_background(BLACK);
            game.update();
            game.draw();
        })
        .await;
        return;
    }

    loop {
        clear_background(BLACK);
        game.update();
        game.draw();
        next_frame().await
    }
}

fn export_playthrough_report_if_requested() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(path) = capture::env_string("TFL_PLAYTHROUGH_REPORT_PATH") else {
            return false;
        };

        let reports = systems::playtest_system::PlaytestSystem::capture_report_set();
        let markdown =
            systems::playtest_system::PlaytestSystem::playthrough_report_markdown(&reports);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent).expect("failed to create playthrough report directory");
        }
        std::fs::write(&path, markdown).expect("failed to write playthrough report");
        true
    }

    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}
