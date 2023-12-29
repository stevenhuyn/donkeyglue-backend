use self::{chatgpt::ChatGpt, player::Player};

use super::{game_controller::Role, game_state::Team};

pub mod chatgpt;
pub mod player;
mod utils;

pub enum Operative {
    Player(Player),
    ChatGpt(ChatGpt),
}

impl Operative {
    pub fn is_player(&self) -> bool {
        match self {
            Self::Player(_) => true,
            Self::ChatGpt(_) => false,
        }
    }

    pub async fn try_gen_guesses(
        &self,
        game_state: &super::game_state::GameState,
    ) -> Option<Vec<String>> {
        match self {
            Self::Player(player) => player.try_gen_guesses(game_state).await,
            Self::ChatGpt(chatgpt) => chatgpt.try_gen_guesses(game_state).await,
        }
    }
}

pub enum Spymaster {
    Player(Player),
    ChatGpt(ChatGpt),
}

impl Spymaster {
    pub fn is_player(&self) -> bool {
        match self {
            Self::Player(_) => true,
            Self::ChatGpt(_) => false,
        }
    }

    pub async fn try_gen_clue(
        &self,
        game_state: &super::game_state::GameState,
    ) -> Option<super::game_state::Clue> {
        match self {
            Self::Player(player) => player.try_gen_clue(game_state).await,
            Self::ChatGpt(chatgpt) => chatgpt.try_gen_clue(game_state).await,
        }
    }
}

pub struct Agents {
    pub red_operative: Operative,
    pub blue_operative: Operative,
    pub red_spymaster: Spymaster,
    pub blue_spymaster: Spymaster,
}

impl Agents {
    pub fn new(role: Role) -> Self {
        let (red_operative, red_spymaster): (Operative, Spymaster) = match role {
            Role::RedOperative => (
                Operative::Player(Player),
                Spymaster::ChatGpt(ChatGpt::new(Team::Red)),
            ),
            Role::RedSpymaster => (
                Operative::ChatGpt(ChatGpt::new(Team::Red)),
                Spymaster::Player(Player),
            ),
        };

        Self {
            red_operative,
            red_spymaster,
            blue_operative: Operative::ChatGpt(ChatGpt::new(Team::Blue)),
            blue_spymaster: Spymaster::ChatGpt(ChatGpt::new(Team::Blue)),
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
