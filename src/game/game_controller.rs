use tokio::sync::{watch, RwLock};

use super::{
    agent::{Agents, Operative, Spymaster},
    game_state::{GameState, Phase, Team},
};

pub struct GameController {
    game_state: RwLock<GameState>,
    sender: watch::Sender<GameState>,
    agents: Agents,
}

impl GameController {
    pub fn new(words: Vec<String>) -> Self {
        let game_state = GameState::new(words);
        let (sender, _receiver) = watch::channel(game_state.clone());
        let agents = Agents::new();
        GameController {
            game_state: RwLock::new(game_state),
            sender,
            agents,
        }
    }

    pub fn get_sender(&self) -> &watch::Sender<GameState> {
        &self.sender
    }

    async fn step_until_input(&self) {
        match self.game_state.read().await.get_phase() {
            Phase::Clue(Team::Red) => self.try_apply_clue(&self.agents.red_spymaster).await,
            Phase::Clue(Team::Blue) => self.try_apply_clue(&self.agents.blue_spymaster).await,
            Phase::Guess(Team::Blue, ..) => self.try_apply_guess(&self.agents.blue_operative).await,
            Phase::Guess(Team::Red, ..) => self.try_apply_guess(&self.agents.red_operative).await,
            _ => {}
        }
    }

    async fn try_apply_clue(&self, spymaster: &Box<dyn Spymaster>) {
        let clue = {
            let game_state = self.game_state.read().await;
            spymaster.try_gen_clue(&game_state).await
        };

        if let Some(clue) = clue {
            self.game_state.write().await.provide_clue(clue).unwrap();
        }
    }

    async fn try_apply_guess(&self, operative: &Box<dyn Operative>) {
        let guesses = {
            let game_state = self.game_state.read().await;
            operative.try_gen_guesses(&game_state).await
        };

        if let Some(guesses) = guesses {
            let mut game_state = self.game_state.write().await;
            for guess in guesses {
                if game_state.make_guess(guess).is_err() {
                    break;
                }
            }
        }
    }

    pub fn get_game_state(&self) -> &RwLock<GameState> {
        &self.game_state
    }
}
