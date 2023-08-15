use async_trait::async_trait;

use crate::game::game_state::{Clue, GameState};

use super::{Operative, Spymaster};

pub struct Player;

#[async_trait]
impl Operative for Player {
    async fn make_guess(&self, game_state: &GameState) -> String {
        "".to_string()
    }
}

#[async_trait]
impl Spymaster for Player {
    async fn provide_clue(&self, game_state: &GameState) -> Clue {
        Clue {
            word: "".to_string(),
            number: 0,
        }
    }
}
