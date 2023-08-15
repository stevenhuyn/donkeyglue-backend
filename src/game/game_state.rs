use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;

use crate::operative::openai::OpenaiOperative;
use crate::operative::Operative;

use super::seed_words::SeedWords;

#[derive(Clone, Debug)]
pub enum Team {
    Red,
    Blue,
}

impl Team {
    fn other(&self) -> Team {
        match self {
            Team::Red => Team::Blue,
            Team::Blue => Team::Red,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum Identity {
    Red,
    Blue,
    Black,
    Neutral,
    // Used to send game state to the client
    Hidden,
}

#[derive(Clone, Debug)]
pub enum Role {
    Spymaster,
    Operative,
}

#[derive(Clone, Debug, Serialize)]

pub struct Codename {
    word: String,
    guessed: bool,
    identity: Identity,
}

#[derive(Clone, Debug)]

pub struct Clue {
    word: String,
    number: u8,
}

impl Clue {
    pub fn new(word: String, number: u8) -> Self {
        Clue { word, number }
    }
}

#[derive(Clone, Debug)]

struct Human {
    role: Role,
    team: Team,
}

#[derive(Clone, Debug)]
pub enum GameState {
    WaitingForClue {
        team: Team,
        codenames: Vec<Codename>,
    },
    Guessing {
        team: Team,
        codenames: Vec<Codename>,
        clue: Clue,
        remaining_guesses: u8,
    },
    GameOver {
        winner: Team,
        codenames: Vec<Codename>,
    },
}

impl GameState {
    pub fn new(seed_words: &SeedWords) -> Self {
        let mut identities = vec![Identity::Red; 9];
        identities.extend(vec![Identity::Blue; 8]);
        identities.extend(vec![Identity::Neutral; 7]);
        identities.extend(vec![Identity::Black; 1]);

        let mut codenames: Vec<Codename> = seed_words
            .get_random_words(25)
            .into_iter()
            .zip(identities)
            .map(|(word, identity)| Codename {
                word,
                identity,
                guessed: false,
            })
            .collect();

        codenames.shuffle(&mut thread_rng());

        GameState::WaitingForClue {
            team: Team::Red,
            codenames,
        }
    }

    pub fn provide_clue(&mut self, clue: Clue) {
        if let GameState::WaitingForClue { team, codenames } = self {
            // Determine if the clue is already a word in the list
            if codenames.iter().any(|word| word.word == clue.word) {
                tracing::debug!("Guessed a word already in the list!");
                return;
            };

            *self = GameState::Guessing {
                team: team.clone(),
                codenames: codenames.clone(),
                remaining_guesses: clue.number,
                clue,
            };
        }
    }

    pub fn make_guess(&mut self, guess: String) {
        let game_state = self.clone();
        tokio::spawn(async move {
            tracing::info!("Guess made!!!");
            let operative = OpenaiOperative::new();
            let res = operative.make_guess(&game_state).await;
            tracing::info!("OpenAI response: {}", res);
        });

        if let GameState::Guessing {
            team,
            codenames,
            clue: _,
            remaining_guesses,
        } = self
        {
            if !codenames.iter().any(|codename| codename.word == guess) {
                tracing::debug!("guess: {guess} not found in codenames!");
                return;
            };

            let codename = codenames.iter_mut().find(|codename| codename.word == guess);
            if let Some(codename) = codename {
                if !codename.guessed {
                    codename.guessed = true;
                    *remaining_guesses -= 1;
                } else {
                    tracing::debug!("Already guessed this word!");
                    return;
                }

                if codename.identity == Identity::Black {
                    tracing::debug!("Selected an assassin!");

                    *self = GameState::GameOver {
                        winner: team.other(),
                        codenames: codenames.clone(),
                    };
                } else if *remaining_guesses == 0 {
                    tracing::debug!("Guesses over, changing to other team!");

                    *self = GameState::WaitingForClue {
                        team: team.other(),
                        codenames: codenames.clone(),
                    };
                }
            }
        }
    }

    pub fn get_hidden_board(&self) -> Vec<Codename> {
        let codenames: Vec<Codename> = match self {
            GameState::WaitingForClue { codenames, .. } => codenames.iter().cloned(),
            GameState::Guessing { codenames, .. } => codenames.iter().cloned(),
            GameState::GameOver { codenames, .. } => codenames.iter().cloned(),
        }
        .collect();

        codenames
            .into_iter()
            .map(|codename| Codename {
                word: codename.word,
                guessed: false,
                identity: match codename.guessed {
                    true => codename.identity,
                    false => Identity::Hidden,
                },
            })
            .collect()
    }
}
