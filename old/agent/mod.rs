use async_trait::async_trait;

use crate::game::game_state::{Clue, GameState};

pub mod openai_operative;
pub mod openai_spymaster;
pub mod player;

#[async_trait]
pub trait Operative: Send + Sync {
    /// Operative tries to generate guesses, returns None if human Player
    async fn try_gen_guesses(&self, game_state: &GameState) -> Option<Vec<String>>;
}

#[async_trait]
pub trait Spymaster: Send + Sync {
    /// Spymaster tries to generate a clue, returns None if human Player
    async fn try_gen_clue(&self, game_state: &GameState) -> Option<Clue>;
}
