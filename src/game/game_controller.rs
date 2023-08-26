use serde::{Deserialize, Serialize};
use tokio::sync::{watch, RwLock};

use super::{
    agent::{Agents, Operative, Spymaster},
    game_state::{Clue, GameState, Phase, Team},
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Role {
    RedOperative,
    RedSpymaster,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ChannelEvent {
    Playing {
        #[serde(rename = "gameState")]
        game_state: GameState,
        role: Role,
    },
}

pub struct GameController {
    game_state: RwLock<GameState>,
    sender: watch::Sender<ChannelEvent>,

    // Adding a single receiver so sender can send while no SSE sessions are active
    _receiver: watch::Receiver<ChannelEvent>,
    agents: Agents,
    role: Role,
}

impl GameController {
    pub fn new(role: Role, words: Vec<String>) -> Self {
        let game_state = GameState::new(words);
        let initial_channel_event = ChannelEvent::Playing {
            game_state: game_state.clone(),
            role: role.clone(),
        };
        let (sender, receiver) = watch::channel(initial_channel_event);
        let agents = Agents::new(role.clone());
        GameController {
            game_state: RwLock::new(game_state),
            sender,
            _receiver: receiver,
            agents,
            role,
        }
    }

    pub fn sender(&self) -> &watch::Sender<ChannelEvent> {
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
            let channel_event = ChannelEvent::Playing {
                game_state: game_state.clone(),
                role: self.role.clone(),
            };
            self.sender.send(channel_event).unwrap();
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
            let channel_event = ChannelEvent::Playing {
                game_state: game_state.clone(),
                role: self.role.clone(),
            };
            let res = self.sender.send(channel_event);
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
                self.try_apply_clue(self.agents.red_spymaster.as_ref())
                    .await
            }
            Phase::Clue { team: Team::Blue } => {
                self.try_apply_clue(self.agents.blue_spymaster.as_ref())
                    .await
            }
            Phase::Guess {
                team: Team::Blue, ..
            } => {
                self.try_apply_guess(self.agents.blue_operative.as_ref())
                    .await
            }
            Phase::Guess {
                team: Team::Red, ..
            } => {
                self.try_apply_guess(self.agents.red_operative.as_ref())
                    .await
            }
            Phase::End => None,
        }
    }

    async fn is_player_turn(&self) -> bool {
        match self.game_state.read().await.phase() {
            Phase::Clue { team: Team::Red } => self.agents.red_spymaster.is_player(),
            Phase::Clue { team: Team::Blue } => self.agents.blue_spymaster.is_player(),
            Phase::Guess {
                team: Team::Blue, ..
            } => self.agents.blue_operative.is_player(),
            Phase::Guess {
                team: Team::Red, ..
            } => self.agents.red_operative.is_player(),
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

            let channel_event = ChannelEvent::Playing {
                game_state: self.game_state.read().await.clone(),
                role: self.role.clone(),
            };
            self.sender.send(channel_event).unwrap();
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
            for guess in guesses {
                tracing::debug!("Attempting to provide guess: {:?}", guess);
                let guess_result = {
                    let mut game_state = self.game_state.write().await;
                    game_state.make_guess(guess)
                };

                if guess_result.is_ok() {
                    tracing::debug!("Attempting to update SSE");
                    let channel_event = ChannelEvent::Playing {
                        game_state: self.game_state.read().await.clone(),
                        role: self.role.clone(),
                    };
                    self.sender.send(channel_event).unwrap();
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
}
