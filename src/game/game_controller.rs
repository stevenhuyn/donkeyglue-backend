use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::{
    sync::{RwLock},
    time::sleep,
};

use crate::routes::game::GetGameResponse;

use super::{
    agent::{Agents, Operative, Spymaster},
    game_state::{Clue, GameState, Phase, Team},
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Role {
    RedOperative,
    RedSpymaster,
}

pub type GameData = GetGameResponse;

pub struct GameController {
    // TODO: make RwLock<GameState> the self method type
    game_state: RwLock<GameState>,
    agents: Agents,
    role: Role,
}

impl GameController {
    pub fn new(role: Role, words: Vec<String>) -> Self {
        let game_state = GameState::new(words);
        let agents = Agents::new(role.clone());
        GameController {
            game_state: RwLock::new(game_state),
            agents,
            role,
        }
    }

    pub async fn player_guess(&self, guess: String) -> Option<()> {
        tracing::info!("Player Clue: Init");

        if !self.is_player_turn().await {
            tracing::info!("Player Clue: Not player turn");
            return None;
        }

        let mut game_state = self.game_state.write().await;
        if let Ok(()) = game_state.make_guess(guess) {
            return Some(());
        }

        tracing::info!("Not correct phase");
        None
    }

    pub async fn player_clue(&self, word: String, count: u8) -> Option<()> {
        tracing::debug!("Player Clue: Init");
        if !self.is_player_turn().await {
            tracing::info!("Player Clue: Not player turn");
            return None;
        }

        let clue = Clue::new(word, count);
        let mut game_state = self.game_state.write().await;
        if let Ok(()) = game_state.provide_clue(clue) {
            return Some(());
        }

        tracing::info!("Not correct phase");
        None
    }

    pub async fn step_until_input(&self) {
        if self.is_player_turn().await {
            tracing::info!(
                "Stepping aborted early cause player turn {:?}",
                self.game_state.read().await.phase()
            );
            return;
        }

        tracing::info!(
            "Initiating Stepping: {:?}",
            self.game_state.read().await.phase()
        );
        while self.step_game().await.is_some() {
            tracing::info!("Stepping game: {:?}", self.game_state.read().await.phase());
        }

        tracing::info!("Player Turn: {:?}", self.game_state.read().await.phase());
    }

    async fn step_game(&self) -> Option<()> {
        let phase = self.game_state.read().await.phase().clone();
        match phase {
            Phase::Clue { team: Team::Red } => {
                self.try_apply_clue(&self.agents.red_spymaster).await
            }
            Phase::Clue { team: Team::Blue } => {
                self.try_apply_clue(&self.agents.blue_spymaster).await
            }
            Phase::Guess {
                team: Team::Blue, ..
            } => self.try_apply_guess(&self.agents.blue_operative).await,
            Phase::Guess {
                team: Team::Red, ..
            } => self.try_apply_guess(&self.agents.red_operative).await,
            Phase::End => None,
        }
    }

    async fn is_player_turn(&self) -> bool {
        {
            tracing::debug!("Is Player Turn: {:?}", self.game_state.read().await.phase());
        }

        match self.game_state.read().await.phase() {
            Phase::Clue { team: Team::Red } => {
                tracing::debug!("Red Spymaster Check");
                self.agents.red_spymaster.is_player()
            }
            Phase::Clue { team: Team::Blue } => {
                tracing::debug!("Blue Spymaster Check");
                self.agents.blue_spymaster.is_player()
            }
            Phase::Guess {
                team: Team::Blue, ..
            } => {
                tracing::debug!("Blue Operative Check");
                self.agents.blue_operative.is_player()
            }
            Phase::Guess {
                team: Team::Red, ..
            } => {
                tracing::debug!("Blue Operative Check");
                self.agents.red_operative.is_player()
            }
            Phase::End => false,
        }
    }

    async fn try_apply_clue(&self, spymaster: &Spymaster) -> Option<()> {
        let clue = {
            let game_state = self.game_state.read().await;
            spymaster.try_gen_clue(&game_state).await
        };

        if let Some(clue) = clue {
            tracing::debug!("AI Clue: {:?}", clue);
            let _ = self.game_state.write().await.provide_clue(clue);
            return Some(());
        }

        None
    }

    async fn try_apply_guess(&self, operative: &Operative) -> Option<()> {
        let guesses = {
            let game_state = self.game_state.read().await;
            operative.try_gen_guesses(&game_state).await
        };

        if let Some(guesses) = guesses {
            tracing::debug!("AI Guesses: {:?}", guesses);
            for guess in guesses {
                tracing::debug!("AI Guess: {:?}", guess);
                let guess_result = {
                    let mut game_state = self.game_state.write().await;
                    game_state.make_guess(guess)
                };

                if guess_result.is_ok() {
                    sleep(Duration::from_millis(1000)).await;
                } else {
                    break;
                }
            }

            return Some(());
        }

        None
    }

    pub fn agents(&self) -> &Agents {
        &self.agents
    }

    pub async fn game_data(&self) -> GameData {
        let mut game_state = self.game_state.read().await.clone();
        let role = self.role.clone();

        if self.agents().should_hide_board() {
            game_state = game_state.to_hidden_game_state();
        }

        GameData::Playing { game_state, role }
    }
}
