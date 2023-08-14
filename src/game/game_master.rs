use super::{
    game_state::{GameState, Role, Team},
    seed_words::SeedWords,
};

struct Player {
    role: Role,
    team: Team,
}

struct GameMaster {
    game_state: GameState,
    player: Player,
}

impl GameMaster {
    pub fn new(seed_words: SeedWords, role: Role, team: Team) -> Self {
        let game_state = GameState::new(&seed_words);
        let player = Player { role, team };
        GameMaster { game_state, player }
    }
}
