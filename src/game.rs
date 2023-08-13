use std::{collections::HashMap, fmt};

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
}

#[derive(Clone, Debug)]
enum Role {
    Spymaster,
    Operative,
}

#[derive(Clone, Debug)]

struct Word {
    guessed: bool,
    identity: Identity,
}

#[derive(Clone, Debug)]

struct Game {
    current_team: Team,
    words: Vec<Word>,
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
        words: HashMap<String, Word>,
    },
    Guessing {
        team: Team,
        words: HashMap<String, Word>,
        clue: Clue,
        remaining_guesses: u8,
    },
    GameOver {
        winner: Team,
        words: HashMap<String, Word>,
    },
}

impl GameState {
    pub fn new() -> Self {
        let words = HashMap::new();
        GameState::WaitingForClue {
            team: Team::Red,
            words,
        }
    }

    pub fn provide_clue(&mut self, clue: Clue) {
        if let GameState::WaitingForClue { team, words } = self {
            *self = GameState::Guessing {
                team: team.clone(),
                words: words.clone(),
                remaining_guesses: clue.number,
                clue,
            };
        }
    }

    pub fn make_guess(&mut self, guess: String) {
        if let GameState::Guessing {
            team,
            words,
            clue: _,
            remaining_guesses,
        } = self
        {
            if let Some(word) = words.get_mut(&guess) {
                if !word.guessed {
                    word.guessed = true;
                    *remaining_guesses -= 1;
                    if word.identity == Identity::Black {
                        *self = GameState::GameOver {
                            winner: team.other(),
                            words: words.clone(),
                        };
                        return;
                    }
                }
            }

            if *remaining_guesses == 0 {
                *self = GameState::WaitingForClue {
                    team: team.other(),
                    words: words.clone(),
                };
            }
        }
    }
}

fn all_words_guessed(words: &Vec<Word>) -> bool {
    words.iter().all(|word| word.guessed)
}
