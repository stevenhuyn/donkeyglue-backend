use async_trait::async_trait;

use crate::game::game_state::{Clue, GameState};

use self::{openai_operative::OpenaiOperative, openai_spymaster::OpenaiSpymaster, player::Player};

use super::{game_controller::Role, game_state::Team};

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
    pub fn new(role: Role) -> Self {
        let (red_operative, red_spymaster): (Box<dyn Operative>, Box<dyn Spymaster>) = match role {
            Role::RedOperative => (
                Box::new(Player::new()),
                Box::new(OpenaiSpymaster::new(Team::Red)),
            ),
            Role::RedSpymaster => (
                Box::new(OpenaiOperative::new(Team::Red)),
                Box::new(Player::new()),
            ),
        };

        Self {
            red_operative,
            red_spymaster,
            blue_operative: Box::new(OpenaiOperative::new(Team::Blue)),
            blue_spymaster: Box::new(OpenaiSpymaster::new(Team::Blue)),
        }
    }

    /// Determines if the board should be hidden from the player, for using with game_state.get_hidden_board()
    pub fn should_hide_board(&self) -> bool {
        if self.red_operative.is_player() || self.blue_operative.is_player() {
            return true;
        } else if self.red_spymaster.is_player() || self.blue_spymaster.is_player() {
            return false;
        }

        unreachable!("At least one player should be a player")
    }
}
