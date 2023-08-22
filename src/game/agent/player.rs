use async_trait::async_trait;

use crate::game::game_state::{Clue, GameState};

use super::{Operative, Spymaster};

pub struct Player;

#[async_trait]
impl Operative for Player {
    async fn try_gen_guesses(&self, _game_state: &GameState) -> Option<Vec<String>> {
        None
    }
}

#[async_trait]
impl Spymaster for Player {
    async fn try_gen_clue(&self, _game_state: &GameState) -> Option<Clue> {
        None
    }
}
