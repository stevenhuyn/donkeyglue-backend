use crate::game::{Clue, GameState};

pub trait Operative {
    fn guess(&self, game_state: GameState) -> String;
}
