use crate::game::game_state::{Clue, GameState};

pub struct Player;

impl Player {
    pub async fn try_gen_guesses(&self, _game_state: &GameState) -> Option<Vec<String>> {
        None
    }

    fn is_player(&self) -> bool {
        tracing::debug!("Is Player True");
        true
    }

    pub async fn try_gen_clue(&self, _game_state: &GameState) -> Option<Clue> {
        None
    }
}
