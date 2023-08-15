use crate::operative::{player::Player, Operative, Spymaster};

use super::{
    game_state::{GameState, Role, Team},
    seed_words::SeedWords,
};

struct GameMaster {
    game_state: GameState,
    red_spymaster: Box<dyn Spymaster>,
    red_operative: Box<dyn Operative>,
    blue_spymaster: Box<dyn Spymaster>,
    blue_operative: Box<dyn Operative>,
}

impl GameMaster {
    pub fn new(seed_words: SeedWords, role: Role, team: Team) -> Self {
        let game_state = GameState::new(&seed_words);
        GameMaster {
            game_state,
            red_spymaster: Box::new(Player),
            red_operative: Box::new(Player),
            blue_spymaster: Box::new(Player),
            blue_operative: Box::new(Player),
        }
    }
}
