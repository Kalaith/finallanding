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
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    ui::font::init_ui_font();
    let mut game: Game = Game::new().await;

    loop {
        clear_background(BLACK);

        game.update();
        game.draw();

        next_frame().await
    }
}
