use crate::operative::{
    openai_operative::OpenaiOperative, openai_spymaster::OpenaiSpymaster, player::Player,
    Operative, Spymaster,
};

use super::{
    game_state::{Clue, GameState, Phase, Role, Team},
    seed_words::SeedWords,
};

struct GameMaster {
    pub game_state: GameState,
    red_spymaster: Box<dyn Spymaster>,
    red_operative: Box<dyn Operative>,
    blue_spymaster: Box<dyn Spymaster>,
    blue_operative: Box<dyn Operative>,
}

impl GameMaster {
    pub fn new(seed_words: SeedWords, player_role: Role) -> Self {
        let game_state = GameState::new(&seed_words);

        let (red_spymaster, red_operative): (Box<dyn Spymaster>, Box<dyn Operative>) =
            match player_role {
                Role::Spymaster => (Box::new(Player), Box::new(OpenaiOperative::new(Team::Red))),
                Role::Operative => (Box::new(OpenaiSpymaster::new(Team::Red)), Box::new(Player)),
            };

        GameMaster {
            game_state,
            red_spymaster,
            red_operative,
            blue_spymaster: Box::new(OpenaiSpymaster::new(Team::Blue)),
            blue_operative: Box::new(OpenaiOperative::new(Team::Blue)),
        }
    }

    pub fn make_guess(&mut self, guess: String) {
        todo!();
    }

    pub fn provide_clue(&mut self, clue: Clue) {
        todo!();
    }

    pub async fn step_simulation(&mut self) -> Option<()> {
        match &self.game_state.phase {
            Phase::RedSpymasterClueing { .. } => {
                let clue = self.red_spymaster.provide_clue(&self.game_state).await;
                if let Some(clue) = clue {
                    self.game_state.provide_clue(clue);
                    return Some(());
                } else {
                    return None;
                }
            }
            Phase::BlueSpymasterClueing { .. } => {
                let clue = self.blue_spymaster.provide_clue(&self.game_state).await;
                if let Some(clue) = clue {
                    self.game_state.provide_clue(clue);
                    return Some(());
                } else {
                    return None;
                }
            }
            Phase::BlueOperativeChoosing { .. } => {
                let guess = self.blue_operative.make_guess(&self.game_state).await;
                if let Some(guess) = guess {
                    self.game_state.make_guess(guess);
                    return Some(());
                } else {
                    return None;
                }
            }
            Phase::RedOperativeChoosing { .. } => {
                let guess = self.red_operative.make_guess(&self.game_state).await;
                if let Some(guess) = guess {
                    self.game_state.make_guess(guess);
                    return Some(());
                } else {
                    return None;
                }
            }
            Phase::GameOver { .. } => return None,
        }
    }

    pub async fn step_until_player(&mut self) {
        while self.step_simulation().await.is_some() {}
    }
}
