use super::game_state::GameState;

pub struct GameController {
    game_state: GameState,
}

impl GameController {
    pub fn new(words: Vec<String>) -> Self {
        let game_state = GameState::new(words);
        GameController { game_state }
    }
}
