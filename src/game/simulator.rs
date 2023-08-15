use crate::operative::{
    openai_operative::OpenaiOperative, openai_spymaster::OpenaiSpymaster, player::Player,
    Operative, Spymaster,
};

use super::{
    game_state::{Clue, GameState, Phase, Role, Team},
    seed_words::SeedWords,
};

pub struct Simulator {
    red_spymaster: Box<dyn Spymaster>,
    red_operative: Box<dyn Operative>,
    blue_spymaster: Box<dyn Spymaster>,
    blue_operative: Box<dyn Operative>,
}

impl Simulator {
    pub fn new(player_role: Role) -> Self {
        let (red_spymaster, red_operative): (Box<dyn Spymaster>, Box<dyn Operative>) =
            match player_role {
                Role::Spymaster => (Box::new(Player), Box::new(OpenaiOperative::new(Team::Red))),
                Role::Operative => (Box::new(OpenaiSpymaster::new(Team::Red)), Box::new(Player)),
            };

        Simulator {
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

    pub async fn step_simulation(&mut self, game_state: &mut GameState) -> Option<()> {
        match game_state.phase {
            Phase::RedSpymasterClueing { .. } => {
                let clue = self.red_spymaster.provide_clue(game_state).await;
                if let Some(clue) = clue {
                    game_state.provide_clue(clue);
                    return Some(());
                } else {
                    return None;
                }
            }
            Phase::BlueSpymasterClueing { .. } => {
                let clue = self.blue_spymaster.provide_clue(game_state).await;
                if let Some(clue) = clue {
                    game_state.provide_clue(clue);
                    return Some(());
                } else {
                    return None;
                }
            }
            Phase::BlueOperativeChoosing { .. } => {
                let guess = self.blue_operative.make_guess(game_state).await;
                if let Some(guess) = guess {
                    game_state.make_guess(guess);
                    return Some(());
                } else {
                    return None;
                }
            }
            Phase::RedOperativeChoosing { .. } => {
                let guess = self.red_operative.make_guess(game_state).await;
                if let Some(guess) = guess {
                    game_state.make_guess(guess);
                    return Some(());
                } else {
                    return None;
                }
            }
            Phase::GameOver { .. } => return None,
        }
    }

    pub async fn step_until_player(&mut self, game_state: &mut GameState) {
        while self.step_simulation(game_state).await.is_some() {}
    }
}
