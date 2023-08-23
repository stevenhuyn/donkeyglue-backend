use tokio::sync::{watch, RwLock};

use super::game_state::{GameState, Phase, Team};

pub struct GameController {
    game_state: RwLock<GameState>,
    sender: watch::Sender<GameState>,
}

impl GameController {
    pub fn new(words: Vec<String>) -> Self {
        let game_state = GameState::new(words);
        let (sender, _receiver) = watch::channel(game_state.clone());
        GameController {
            game_state: RwLock::new(game_state),
            sender,
        }
    }

    pub fn get_sender(&self) -> &watch::Sender<GameState> {
        &self.sender
    }

    async fn step_until_input(&mut self) {
        match self.game_state.read().await.get_phase() {
            Phase::Clue(Team::Red) => {}
            Phase::Clue(Team::Blue) => {}
            _ => {}
        }
    }

    pub fn get_game_state(&self) -> &RwLock<GameState> {
        &self.game_state
    }
}
