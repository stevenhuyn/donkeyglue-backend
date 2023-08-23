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
        while self.step_game().await.is_some() {}
    }

    async fn step_game(&self) -> Option<()> {
        match self.game_state.read().await.get_phase() {
            Phase::Clue(Team::Red) => self.try_apply_clue(&self.agents.red_spymaster).await,
            Phase::Clue(Team::Blue) => self.try_apply_clue(&self.agents.blue_spymaster).await,
            Phase::Guess(Team::Blue, ..) => self.try_apply_guess(&self.agents.blue_operative).await,
            Phase::Guess(Team::Red, ..) => self.try_apply_guess(&self.agents.red_operative).await,
            Phase::End => None,
        }
    }

    async fn try_apply_clue(&self, spymaster: &Box<dyn Spymaster>) -> Option<()> {
        let clue = {
            let game_state = self.game_state.read().await;
            spymaster.try_gen_clue(&game_state).await
        };

        if let Some(clue) = clue {
            let _ = self.game_state.write().await.provide_clue(clue);
            return Some(());
        }

        None
    }

    async fn try_apply_guess(&self, operative: &Box<dyn Operative>) -> Option<()> {
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

            return Some(());
        }

        None
    }

    pub fn get_game_state(&self) -> &RwLock<GameState> {
        &self.game_state
    }
}
