use std::fmt::Display;

use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use serde::Serialize;

use crate::operative::openai_operative::OpenaiOperative;
use crate::operative::openai_spymaster::OpenaiSpymaster;
use crate::operative::Operative;
use crate::operative::Spymaster;

use super::seed_words::SeedWords;

#[derive(Clone, Debug, PartialEq)]
pub enum Team {
    Red,
    Blue,
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let team = match self {
            Team::Red => "Red",
            Team::Blue => "Blue",
        };

        write!(f, "{}", team)
    }
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
pub enum Phase {
    RedSpymasterClueing {
        codenames: Vec<Codename>,
    },
    RedOperativeChoosing {
        codenames: Vec<Codename>,
        clue: Clue,
        remaining_guesses: u8,
    },
    BlueSpymasterClueing {
        codenames: Vec<Codename>,
    },
    BlueOperativeChoosing {
        codenames: Vec<Codename>,
        clue: Clue,
        remaining_guesses: u8,
    },
    GameOver {
        winner: Team,
        codenames: Vec<Codename>,
    },
}
#[derive(Clone, Debug)]
pub struct GameState {
    pub phase: Phase,
    history: Vec<Action>,
}

#[derive(Clone, Debug)]
pub enum Action {
    ProvideClue(Clue),
    MakeGuess(String),
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

        let phase = match rand::thread_rng().gen_range(0..2) {
            0 => Phase::RedSpymasterClueing { codenames },
            1 => Phase::BlueSpymasterClueing { codenames },
            // TODO: use .choose() instead of gen_range?
            _ => unreachable!(),
        };

        GameState {
            phase,
            history: vec![],
        }
    }

    pub fn provide_clue(&mut self, clue: Clue) {
        match &self.phase {
            Phase::RedSpymasterClueing { codenames }
            | Phase::BlueSpymasterClueing { codenames } => {
                // Determine if the clue is already a word in the list
                if codenames.iter().any(|word| word.word == clue.word) {
                    tracing::debug!("Guessed a word already in the list!");
                    return;
                };

                match &mut self.phase {
                    Phase::RedSpymasterClueing { codenames } => {
                        self.phase = Phase::RedOperativeChoosing {
                            codenames: codenames.clone(),
                            remaining_guesses: clue.number,
                            clue,
                        };
                    }
                    Phase::BlueSpymasterClueing { codenames } => {
                        self.phase = Phase::BlueOperativeChoosing {
                            codenames: codenames.clone(),
                            remaining_guesses: clue.number,
                            clue,
                        };
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                tracing::debug!("Not in the right phase to provide a clue!");
            }
        }
    }

    pub fn make_guess(&mut self, guess: String) {
        match &mut self.phase.clone() {
            Phase::BlueOperativeChoosing {
                codenames,
                remaining_guesses,
                ..
            }
            | Phase::RedOperativeChoosing {
                codenames,
                remaining_guesses,
                ..
            } => {
                if !codenames.iter().any(|codename| codename.word == guess) {
                    tracing::debug!("guess: {guess} not found in codenames!");
                    return;
                };

                let codename = codenames.iter_mut().find(|codename| codename.word == guess);
                if let Some(codename) = codename {
                    if !codename.guessed {
                        codename.guessed = true;
                        *remaining_guesses -= 1;
                        tracing::debug!(
                            "Successfully guessed {guess}! {remaining_guesses} guesses remaining."
                        );

                        if *remaining_guesses == 0u8 {
                            self.phase = match self.phase {
                                Phase::BlueOperativeChoosing { .. } => Phase::RedSpymasterClueing {
                                    codenames: codenames.clone(),
                                },
                                Phase::RedOperativeChoosing { .. } => Phase::BlueSpymasterClueing {
                                    codenames: codenames.clone(),
                                },
                                _ => unreachable!(),
                            };
                        } else {
                            match &self.phase.clone() {
                                Phase::BlueOperativeChoosing { .. }
                                | Phase::RedOperativeChoosing { .. } => {
                                    self.phase = Phase::BlueOperativeChoosing {
                                        codenames: codenames.clone(),
                                        remaining_guesses: *remaining_guesses,
                                        clue: self.get_clue().unwrap(),
                                    };
                                    self.history.push(Action::MakeGuess(guess.clone()));
                                }
                                _ => unreachable!(),
                            }
                        }
                    } else {
                        tracing::debug!("Already guessed this word!")
                    }
                }
            }
            _ => {
                tracing::debug!("Not in the right phase to make a guess!");
            }
        }
    }

    pub fn get_hidden_board(&self) -> Vec<Codename> {
        let codenames: Vec<Codename> = match &self.phase {
            Phase::RedSpymasterClueing { codenames, .. } => codenames.iter().cloned(),
            Phase::RedOperativeChoosing { codenames, .. } => codenames.iter().cloned(),
            Phase::BlueSpymasterClueing { codenames, .. } => codenames.iter().cloned(),
            Phase::BlueOperativeChoosing { codenames, .. } => codenames.iter().cloned(),
            Phase::GameOver { codenames, .. } => codenames.iter().cloned(),
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

    pub fn get_board(&self) -> &Vec<Codename> {
        match &self.phase {
            Phase::RedSpymasterClueing { codenames, .. } => codenames,
            Phase::RedOperativeChoosing { codenames, .. } => codenames,
            Phase::BlueSpymasterClueing { codenames, .. } => codenames,
            Phase::BlueOperativeChoosing { codenames, .. } => codenames,
            Phase::GameOver { codenames, .. } => codenames,
        }
    }

    pub fn get_clue(&self) -> Option<Clue> {
        match &self.phase {
            Phase::RedOperativeChoosing { clue, .. } => Some(clue.clone()),
            Phase::BlueOperativeChoosing { clue, .. } => Some(clue.clone()),
            _ => None,
        }
    }
}
