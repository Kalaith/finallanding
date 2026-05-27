#![allow(dead_code)]

use macroquad::prelude::*;

mod data;
mod game;
mod state;
mod systems;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "The Final Landing".to_owned(),
        window_width: env_i32("TFL_WINDOW_WIDTH", 1280),
        window_height: env_i32("TFL_WINDOW_HEIGHT", 720),
        window_resizable: true,
        fullscreen: env_bool("TFL_FULLSCREEN", false),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    ui::font::init_ui_font();
    let mut game: Game = Game::new().await;
    let capture_path = env_string("TFL_CAPTURE_PATH");
    let capture_after_frames = env_u32("TFL_CAPTURE_FRAMES", 8).max(1);
    let mut rendered_frames = 0;

    loop {
        clear_background(BLACK);

        game.update();
        game.draw();
        rendered_frames += 1;

        if let Some(path) = capture_path.as_ref() {
            if rendered_frames >= capture_after_frames {
                get_screen_data().export_png(path);
                break;
            }
        }

        next_frame().await
    }
}

fn env_i32(name: &str, fallback: i32) -> i32 {
    env_string(name)
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(fallback)
}

fn env_u32(name: &str, fallback: u32) -> u32 {
    env_string(name)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(fallback)
}

fn env_bool(name: &str, fallback: bool) -> bool {
    env_string(name)
        .map(|value| value != "0" && !value.eq_ignore_ascii_case("false"))
        .unwrap_or(fallback)
}

fn env_string(name: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var(name).ok()
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = name;
        None
    }
}
