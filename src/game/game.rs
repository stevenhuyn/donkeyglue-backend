use tokio::sync::watch;

use crate::operative::{
    openai_operative::OpenaiOperative, openai_spymaster::OpenaiSpymaster, player::Player,
    Operative, Spymaster,
};
use anyhow::{bail, Result};

use super::{
    game_state::{Clue, GameState, Phase, Role, Team},
    seed_words::SeedWords,
};

enum HumanRole {
    Spymaster,
    Operative,
}

pub struct Game {
    human_role: HumanRole,
    red_spymaster: Box<dyn Spymaster>,
    red_operative: Box<dyn Operative>,
    blue_spymaster: Box<dyn Spymaster>,
    blue_operative: Box<dyn Operative>,
    game_state: GameState,
    sender: watch::Sender<GameState>,
}

impl Game {
    pub fn new(player_role: Role, seed_words: &SeedWords) -> Self {
        let (red_spymaster, red_operative): (Box<dyn Spymaster>, Box<dyn Operative>) =
            match player_role {
                Role::Spymaster => (Box::new(Player), Box::new(OpenaiOperative::new(Team::Red))),
                Role::Operative => (Box::new(OpenaiSpymaster::new(Team::Red)), Box::new(Player)),
            };

        let human_role = match player_role {
            Role::Spymaster => HumanRole::Spymaster,
            Role::Operative => HumanRole::Operative,
        };

        let game_state = GameState::new(seed_words);
        let (sender, _receiver) = watch::channel(game_state);

        Game {
            human_role,
            red_spymaster,
            red_operative,
            blue_spymaster: Box::new(OpenaiSpymaster::new(Team::Blue)),
            blue_operative: Box::new(OpenaiOperative::new(Team::Blue)),
            game_state,
            sender,
        }
    }

    /// Make a guess, usually from a player
    pub async fn make_guess(&self, game_state: &mut GameState, guess: String) -> Result<()> {
        if let Phase::RedOperativeChoosing { .. } = game_state.phase {
            if let HumanRole::Operative = self.human_role {
                tracing::info!("Simulator - Making Guess");
                game_state.make_guess(guess);
                self.step_until_player(game_state).await;
                return Ok(());
            }
        }

        bail!("Tried to make guess in incorrect phase")
    }

    /// Provide a clue, usually from a player
    pub async fn provide_clue(&self, game_state: &mut GameState, clue: Clue) -> Result<()> {
        if let Phase::RedSpymasterClueing { .. } = game_state.phase {
            if let HumanRole::Spymaster = self.human_role {
                tracing::info!("Simulator - Providing Clue");
                game_state.provide_clue(clue);
                self.step_until_player(game_state).await;
                return Ok(());
            }
        }

        bail!("Tried to provide clue in incorrect phase")
    }

    pub async fn step_simulation(&self, game_state: &mut GameState) -> Option<()> {
        match game_state.phase {
            Phase::RedSpymasterClueing { .. } => {
                let clue = self.red_spymaster.provide_clue(game_state).await;
                if let Some(clue) = clue {
                    game_state.provide_clue(clue);
                    Some(())
                } else {
                    None
                }
            }
            Phase::BlueSpymasterClueing { .. } => {
                let clue = self.blue_spymaster.provide_clue(game_state).await;
                if let Some(clue) = clue {
                    game_state.provide_clue(clue);
                    Some(())
                } else {
                    None
                }
            }
            Phase::BlueOperativeChoosing { .. } => {
                let guesses = self.blue_operative.make_guesses(game_state).await;
                if let Some(guesses) = guesses {
                    for guess in guesses {
                        game_state.make_guess(guess);
                    }
                    Some(())
                } else {
                    None
                }
            }
            Phase::RedOperativeChoosing { .. } => {
                let guesses = self.red_operative.make_guesses(game_state).await;
                if let Some(guesses) = guesses {
                    for guess in guesses {
                        game_state.make_guess(guess);
                    }

                    Some(())
                } else {
                    None
                }
            }
            Phase::GameOver { .. } => None,
        }
    }

    pub async fn step_until_player(&self, game_state: &mut GameState) {
        while self.step_simulation(game_state).await.is_some() {
            tracing::info!("Stepping Simulation!");
        }

        tracing::info!("STEPPING DONE!");
    }

    pub fn get_sender(&self) -> watch::Sender<GameState> {
        self.sender
    }
}
