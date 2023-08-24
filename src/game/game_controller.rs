use futures::future::ready;
use tokio::sync::{watch, RwLock};

use super::{
    agent::{Agents, Operative, Spymaster},
    game_state::{Clue, GameState, Phase, Team},
};

pub struct GameController {
    game_state: RwLock<GameState>,
    sender: watch::Sender<GameState>,

    // Adding a single receiver so sender can send while no SSE sessions are active
    _receiver: watch::Receiver<GameState>,
    agents: Agents,
}

impl GameController {
    pub fn new(words: Vec<String>) -> Self {
        let game_state = GameState::new(words);
        let (sender, receiver) = watch::channel(game_state.clone());
        let agents = Agents::new();
        GameController {
            game_state: RwLock::new(game_state),
            sender,
            _receiver: receiver,
            agents,
        }
    }

    pub fn get_sender(&self) -> &watch::Sender<GameState> {
        &self.sender
    }

    pub async fn player_guess(&self, guess: String) -> Option<()> {
        if !self.is_player_turn().await {
            tracing::info!("Not player turn");
            return None;
        }

        let mut game_state = self.game_state.write().await;
        if let Ok(()) = game_state.make_guess(guess) {
            tracing::debug!("Player Guess attempting to update SSE");
            self.sender.send(game_state.clone()).unwrap();
            return Some(());
        }

        tracing::info!("Not correct phase");
        None
    }

    pub async fn player_clue(&self, word: String, count: u8) -> Option<()> {
        tracing::debug!("Player providing clue");
        if !self.is_player_turn().await {
            tracing::info!("Not player turn");
            return None;
        }

        let clue = Clue::new(word, count);
        let mut game_state = self.game_state.write().await;
        if let Ok(()) = game_state.provide_clue(clue) {
            tracing::debug!("Player Clue attempting to update SSE");
            let cloned_game_state = game_state.clone();
            let res = self.sender.send(cloned_game_state);
            println!("{:?}", res);

            return Some(());
        }

        tracing::info!("Not correct phase");
        None
    }

    pub async fn step_until_input(&self) {
        if self.is_player_turn().await {
            tracing::info!(
                "Stepping aborted early cause player turn {:?}",
                self.game_state.read().await.get_phase()
            );
            return;
        }

        tracing::info!(
            "Initiating Stepping: {:?}",
            self.game_state.read().await.get_phase()
        );
        while self.step_game().await.is_some() {
            tracing::info!(
                "Stepping game: {:?}",
                self.game_state.read().await.get_phase()
            );
        }

        tracing::info!(
            "Player Turn: {:?}",
            self.game_state.read().await.get_phase()
        );
    }

    async fn step_game(&self) -> Option<()> {
        let phase = self.game_state.read().await.get_phase().clone();
        match phase {
            Phase::Clue(Team::Red) => {
                self.try_apply_clue(self.agents.red_spymaster.as_ref())
                    .await
            }
            Phase::Clue(Team::Blue) => {
                self.try_apply_clue(self.agents.blue_spymaster.as_ref())
                    .await
            }
            Phase::Guess(Team::Blue, ..) => {
                self.try_apply_guess(self.agents.blue_operative.as_ref())
                    .await
            }
            Phase::Guess(Team::Red, ..) => {
                self.try_apply_guess(self.agents.red_operative.as_ref())
                    .await
            }
            Phase::End => None,
        }
    }

    async fn is_player_turn(&self) -> bool {
        match self.game_state.read().await.get_phase() {
            Phase::Clue(Team::Red) => self.agents.red_spymaster.is_player(),
            Phase::Clue(Team::Blue) => self.agents.blue_spymaster.is_player(),
            Phase::Guess(Team::Blue, ..) => self.agents.blue_operative.is_player(),
            Phase::Guess(Team::Red, ..) => self.agents.red_operative.is_player(),
            Phase::End => false,
        }
    }

    async fn try_apply_clue(&self, spymaster: &dyn Spymaster) -> Option<()> {
        let clue = {
            let game_state = self.game_state.read().await;
            spymaster.try_gen_clue(&game_state).await
        };

        if let Some(clue) = clue {
            {
                tracing::debug!("Attempting to provide clue: {:?}", clue);
                let _ = self.game_state.write().await.provide_clue(clue);
            }

            tracing::debug!("Attempting to update SSE");

            self.sender
                .send(self.game_state.read().await.clone())
                .unwrap();
            return Some(());
        }

        None
    }

    async fn try_apply_guess(&self, operative: &dyn Operative) -> Option<()> {
        let guesses = {
            let game_state = self.game_state.read().await;
            operative.try_gen_guesses(&game_state).await
        };

        if let Some(guesses) = guesses {
            tracing::debug!("Attempting to provide guesses: {:?}", guesses);
            let mut game_state = self.game_state.write().await;
            for guess in guesses {
                tracing::debug!("Attempting to provide guess: {:?}", guess);
                if game_state.make_guess(guess).is_ok() {
                    tracing::debug!("Attempting to update SSE");
                    self.sender.send(game_state.clone()).unwrap();
                } else {
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

    pub fn get_agents(&self) -> &Agents {
        &self.agents
    }
}
