pub mod building_system;
pub mod colonist_ai;
pub mod colonist_spawner;

use crate::state::menu_state::MenuState;

use crate::state::game_state::GameplayState;
use crate::state::{State, StateTransition};

pub enum GameStateEnum {
    Gameplay(GameplayState),
    Menu(MenuState),
}

pub struct Game {
    state: GameStateEnum,
}

impl Game {
    pub async fn new() -> Self {
        let state = if should_start_gameplay() {
            GameStateEnum::Gameplay(GameplayState::new())
        } else {
            GameStateEnum::Menu(MenuState::new())
        };

        Self { state }
    }

    pub fn update(&mut self) {
        match &mut self.state {
            GameStateEnum::Gameplay(state) => {
                let transition = state.update();
                match transition {
                    StateTransition::ToGameplay(new_state) => {
                        self.state = GameStateEnum::Gameplay(new_state);
                    }
                    StateTransition::Quit => std::process::exit(0),
                    _ => {}
                }
            }
            GameStateEnum::Menu(state) => {
                let transition = state.update_with_input();
                match transition {
                    StateTransition::ToGameplay(new_state) => {
                        self.state = GameStateEnum::Gameplay(new_state);
                    }
                    StateTransition::Quit => std::process::exit(0),
                    _ => {}
                }
            }
        }
    }

    pub fn draw(&self) {
        match &self.state {
            GameStateEnum::Gameplay(state) => state.draw(),
            GameStateEnum::Menu(state) => state.draw_ui(),
        }
    }
}

fn should_start_gameplay() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("TFL_START_GAMEPLAY").is_ok_and(|value| value != "0")
    }

    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}
