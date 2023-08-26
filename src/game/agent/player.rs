use async_trait::async_trait;

use crate::game::game_state::{Clue, GameState};

use super::{Operative, Spymaster};

pub struct Player;

impl Player {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Operative for Player {
    async fn try_gen_guesses(&self, _game_state: &GameState) -> Option<Vec<String>> {
        None
    }

    fn is_player(&self) -> bool {
        tracing::debug!("Is Player True");
        true
    }
}

#[async_trait]
impl Spymaster for Player {
    async fn try_gen_clue(&self, _game_state: &GameState) -> Option<Clue> {
        None
    }

    fn is_player(&self) -> bool {
        tracing::debug!("Is Player True");
        true
    }
}
