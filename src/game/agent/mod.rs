use async_trait::async_trait;

use crate::game::game_state::{Clue, GameState};

use self::{openai_operative::OpenaiOperative, openai_spymaster::OpenaiSpymaster, player::Player};

use super::game_state::Team;

pub mod openai_operative;
pub mod openai_spymaster;
pub mod player;

#[async_trait]
pub trait Operative: Send + Sync {
    /// Operative tries to generate guesses, returns None if human Player
    async fn try_gen_guesses(&self, game_state: &GameState) -> Option<Vec<String>>;

    fn is_player(&self) -> bool {
        false
    }
}

#[async_trait]
pub trait Spymaster: Send + Sync {
    /// Spymaster tries to generate a clue, returns None if human Player
    async fn try_gen_clue(&self, game_state: &GameState) -> Option<Clue>;

    fn is_player(&self) -> bool {
        false
    }
}

pub struct Agents {
    pub red_operative: Box<dyn Operative>,
    pub blue_operative: Box<dyn Operative>,
    pub red_spymaster: Box<dyn Spymaster>,
    pub blue_spymaster: Box<dyn Spymaster>,
}

impl Agents {
    pub fn new() -> Self {
        Self {
            red_operative: Box::new(OpenaiOperative::new(Team::Red)),
            blue_operative: Box::new(OpenaiOperative::new(Team::Blue)),
            red_spymaster: Box::new(Player::new()),
            blue_spymaster: Box::new(OpenaiSpymaster::new(Team::Blue)),
        }
    }
}
