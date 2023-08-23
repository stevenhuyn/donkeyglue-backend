use tokio::sync::watch;

use super::game_state::{GameState, Phase, Team};

pub struct GameController {
    game_state: GameState,
    sender: watch::Sender<GameState>,
}

impl GameController {
    pub fn new(words: Vec<String>) -> Self {
        let game_state = GameState::new(words);
        let (sender, _receiver) = watch::channel(game_state.clone());
        GameController { game_state, sender }
    }

    pub fn get_sender(&self) -> &watch::Sender<GameState> {
        &self.sender
    }

    fn step_until_input(&mut self) {
        match self.game_state.get_phase() {
            Phase::Clue(Team::Red) => {}
            Phase::Clue(Team::Blue) => {}
            _ => {}
        }
    }
}
