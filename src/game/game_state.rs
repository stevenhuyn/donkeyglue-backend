use std::fmt::Display;

use anyhow::Result;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum Identity {
    Red,
    Blue,
    Bystander,
    Assassin,
    Hidden,
}

#[derive(Clone, Debug, Serialize)]
pub struct Card {
    word: String,
    guessed: bool,
    identity: Identity,
}

impl Card {
    pub fn new(word: String, identity: Identity) -> Self {
        Card {
            word,
            guessed: false,
            identity,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Clue {
    word: String,
    count: u8,
    remaining: u8,
}

impl Clue {
    pub fn new(word: String, count: u8) -> Self {
        Clue {
            word,
            count,
            remaining: count,
        }
    }
}

#[derive(Clone, Debug)]
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
    pub fn other(&self) -> Self {
        match self {
            Team::Red => Team::Blue,
            Team::Blue => Team::Red,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Phase {
    Clue(Team),
    Guess(Team, Clue),
    End,
}
/// Legal moves only
#[derive(Clone, Debug)]
pub struct GameState {
    board: Vec<Card>,
    phase: Phase,
}

impl GameState {
    const IDENTITIES_ARRAY: [Identity; 25] = [
        Identity::Red,
        Identity::Red,
        Identity::Red,
        Identity::Red,
        Identity::Red,
        Identity::Red,
        Identity::Red,
        Identity::Red,
        Identity::Red,
        Identity::Blue,
        Identity::Blue,
        Identity::Blue,
        Identity::Blue,
        Identity::Blue,
        Identity::Blue,
        Identity::Blue,
        Identity::Blue,
        Identity::Bystander,
        Identity::Bystander,
        Identity::Bystander,
        Identity::Bystander,
        Identity::Bystander,
        Identity::Bystander,
        Identity::Bystander,
        Identity::Assassin,
    ];

    pub fn new(words: Vec<String>) -> Self {
        let cards: Vec<Card> = words
            .into_iter()
            .zip(Self::IDENTITIES_ARRAY)
            .map(|(word, identity)| Card::new(word, identity))
            .collect();
        let phase = Phase::Clue(Team::Red);

        GameState {
            board: cards,
            phase,
        }
    }

    pub fn provide_clue(&mut self, clue: Clue) -> Result<()> {
        if self.board.iter().any(|card| card.word == clue.word) {
            tracing::debug!("The clue is a word on the board!");
            return Err(anyhow::anyhow!("The clue is a word on the board!"));
        };

        tracing::debug!("GameState Provide Clue");
        match &self.phase {
            Phase::Clue(team) => {
                tracing::debug!("Succesfully gave clue: {:?}", &clue);
                self.phase = Phase::Guess(team.other(), clue);
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Wrong phase")),
        }
    }

    pub fn make_guess(&mut self, word: String) -> Result<()> {
        tracing::debug!("Making guess in game_state");

        match &mut self.phase {
            Phase::Guess(team, clue) => {
                let card = self.board.iter_mut().find(|card| card.word == word);

                if card.is_none() {
                    tracing::debug!("Guess is not found on the board!");
                    return Err(anyhow::anyhow!("Guess is not found on the board!"));
                };

                let card = card.unwrap();
                if card.guessed {
                    tracing::debug!("Card has already been guessed!");
                    return Err(anyhow::anyhow!("Card has already been guessed!"));
                };

                card.guessed = true;
                clue.remaining -= 1;

                if clue.remaining == 0 {
                    self.phase = Phase::Clue(team.other());
                }

                tracing::debug!("Succesfully made guess: {word}");
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Wrong phase")),
        }
    }

    pub fn get_board(&self) -> &Vec<Card> {
        &self.board
    }

    pub fn get_hidden_board(&self) -> Vec<Card> {
        tracing::debug!("Get Hidden Board");

        self.board
            .iter()
            .map(|card| Card {
                word: card.word.clone(),
                guessed: false,
                identity: match card.guessed {
                    true => card.identity.clone(),
                    false => Identity::Hidden,
                },
            })
            .collect()
    }

    pub fn get_clue(&self) -> Option<&Clue> {
        match &self.phase {
            Phase::Guess(_, clue) => Some(clue),
            _ => None,
        }
    }

    pub fn get_phase(&self) -> &Phase {
        &self.phase
    }
}
