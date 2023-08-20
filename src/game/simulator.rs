use crate::operative::{
    openai_operative::OpenaiOperative, openai_spymaster::OpenaiSpymaster, player::Player,
    Operative, Spymaster,
};

use super::game_state::{Clue, GameState, Phase, Role, Team};

enum HumanRole {
    Spymaster,
    Operative,
}

pub struct Simulator {
    human_role: HumanRole,
    red_spymaster: Box<dyn Spymaster>,
    red_operative: Box<dyn Operative>,
    blue_spymaster: Box<dyn Spymaster>,
    blue_operative: Box<dyn Operative>,
}

impl Simulator {
    pub fn new(player_role: Role) -> Self {
        let (red_spymaster, red_operative): (Box<dyn Spymaster>, Box<dyn Operative>) =
            match player_role {
                Role::Spymaster => (Box::new(Player), Box::new(OpenaiOperative::new(Team::Red))),
                Role::Operative => (Box::new(OpenaiSpymaster::new(Team::Red)), Box::new(Player)),
            };

        let human_role = match player_role {
            Role::Spymaster => HumanRole::Spymaster,
            Role::Operative => HumanRole::Operative,
        };

        Simulator {
            human_role,
            red_spymaster,
            red_operative,
            blue_spymaster: Box::new(OpenaiSpymaster::new(Team::Blue)),
            blue_operative: Box::new(OpenaiOperative::new(Team::Blue)),
        }
    }

    pub async fn make_guess(&self, game_state: &mut GameState, guess: String) {
        if let Phase::RedOperativeChoosing { .. } = game_state.phase {
            if let HumanRole::Operative = self.human_role {
                tracing::info!("Simulator - Making Guess");
                game_state.make_guess(guess);
            }
        }

        // TODO: Spawn thread to do this
        self.step_until_player(game_state).await;
    }

    pub async fn provide_clue(&self, game_state: &mut GameState, clue: Clue) {
        if let Phase::RedSpymasterClueing { .. } = game_state.phase {
            if let HumanRole::Spymaster = self.human_role {
                tracing::info!("Simulator - Providing Clue");
                game_state.provide_clue(clue);
            }
        }

        // TODO: Spawn thread to do this
        self.step_until_player(game_state).await;
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
                let guess = self.blue_operative.make_guess(game_state).await;
                if let Some(guess) = guess {
                    game_state.make_guess(guess);
                    Some(())
                } else {
                    None
                }
            }
            Phase::RedOperativeChoosing { .. } => {
                let guess = self.red_operative.make_guess(game_state).await;
                if let Some(guess) = guess {
                    game_state.make_guess(guess);
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
}
