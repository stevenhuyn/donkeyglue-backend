use async_trait::async_trait;

use crate::game::game_state::{Clue, GameState, Role, Team};

pub mod player;

#[async_trait]
pub trait Operative {
    async fn make_guess(&self, game_state: &GameState) -> String;
}

#[async_trait]
pub trait Spymaster {
    async fn provide_clue(&self, game_state: &GameState) -> Clue;
}
