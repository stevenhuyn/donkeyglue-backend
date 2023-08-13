use std::collections::HashMap;

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

enum Identity {
    Red,
    Blue,
    Black,
    Neutral,
}

enum Role {
    Spymaster,
    Operative,
}

struct Word {
    guessed: bool,
    identity: Identity,
}

struct Game {
    current_team: Team,
    words: Vec<Word>,
}

struct Clue {
    word: String,
    number: u8,
}

type Words = HashMap<String, Word>;

type Guesses = u8;

enum GameState {
    WaitingForClue(Team, Words),
    Guessing(Team, Words, Clue, Guesses),
    GameOver,
}

impl GameState {
    fn provide_clue(self, clue: String, number: u8) -> Self {
        match self {
            GameState::WaitingForClue(team, words) => {
                GameState::Guessing(team, words, Clue { word: clue, number }, number)
            }
            _ => self,
        }
    }

    fn make_guess(self, guess: String) -> Self {
        match self {
            GameState::Guessing(team, mut words, clue, mut guesses) => {
                let word = words.get_mut(&guess);
                if let Some(word) = word {
                    word.guessed = true;
                    guesses -= 1;

                    if guesses == 0 {
                        return GameState::WaitingForClue(team.other(), words);
                    } else {
                        return GameState::Guessing(team, words, clue, guesses);
                    }
                }

                GameState::Guessing(team, words, clue, guesses)
            }
            _ => self,
        }
    }
}

fn all_words_guessed(words: &Vec<Word>) -> bool {
    words.iter().all(|word| word.guessed)
}
