use std::io::BufRead;
use std::{collections::HashMap, fmt, fs::File, io, path::Path};

use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::operative::Operative;

#[derive(Clone, Debug)]
enum Team {
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

#[derive(Clone, Debug, PartialEq)]
enum Identity {
    Red,
    Blue,
    Black,
    Neutral,
    Hidden,
}

#[derive(Clone, Debug)]
enum Role {
    Spymaster,
    Operative,
}

#[derive(Clone, Debug)]

struct Codename {
    word: String,
    guessed: bool,
    identity: Identity,
}

#[derive(Clone, Debug)]

struct Game {
    current_team: Team,
    words: Vec<Codename>,
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
    pub fn new(seed_words: &Vec<String>) -> Self {
        let mut identities = vec![Identity::Red; 9];
        identities.extend(vec![Identity::Blue; 8]);
        identities.extend(vec![Identity::Neutral; 7]);
        identities.extend(vec![Identity::Black; 1]);

        let mut codenames: Vec<Codename> = Self::get_random_words(seed_words)
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

    fn get_random_words(seed_words: &Vec<String>) -> Vec<String> {
        let random_words = seed_words
            .iter()
            .choose_multiple(&mut rand::thread_rng(), 25);
        random_words.into_iter().cloned().collect()
    }

    pub fn provide_clue(&mut self, clue: Clue) {
        if let GameState::WaitingForClue { team, codenames } = self {
            if !codenames.iter().any(|word| word.word == clue.word) {
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
            let operative = Operative {};
            let res = operative.guess(game_state).await;
            tracing::info!("OpenAI response: {}", res);
        });

        if let GameState::Guessing {
            team,
            codenames,
            clue,
            remaining_guesses,
        } = self
        {
            if !codenames.iter().any(|word| word.word == clue.word) {
                return;
            };

            let codename = codenames.iter_mut().find(|codename| codename.word == guess);
            if let Some(codename) = codename {
                if !codename.guessed {
                    codename.guessed = true;
                    *remaining_guesses -= 1;
                }

                if codename.identity == Identity::Black {
                    *self = GameState::GameOver {
                        winner: team.other(),
                        codenames: codenames.clone(),
                    };
                } else if *remaining_guesses == 0 {
                    *self = GameState::WaitingForClue {
                        team: team.other(),
                        codenames: codenames.clone(),
                    };
                }
            }
        }
    }
}

fn all_words_guessed(words: &Vec<Codename>) -> bool {
    words.iter().all(|word| word.guessed)
}
