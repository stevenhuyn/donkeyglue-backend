use std::fmt::Display;

use anyhow::Result;
use rand::seq::SliceRandom;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum Identity {
    Red,
    Blue,
    Bystander,
    Assassin,
    Hidden,
}

impl Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let identity = match self {
            Identity::Red => "Red",
            Identity::Blue => "Blue",
            Identity::Bystander => "Bystander",
            Identity::Assassin => "Assassin",
            Identity::Hidden => "Hidden",
        };

        write!(f, "{}", identity)
    }
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

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    pub fn guessed(&self) -> bool {
        self.guessed
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let card_state = match self.guessed {
            true => "Guessed",
            false => "Hidden",
        };

        write!(
            f,
            "({} - {} - {})",
            self.word(),
            self.identity(),
            card_state
        )
    }
}

#[derive(Clone, Debug, Serialize)]
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

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum Team {
    Red,
    Blue,
}

impl PartialEq<Team> for Identity {
    fn eq(&self, other: &Team) -> bool {
        matches!(
            (self, other),
            (Identity::Red, Team::Red) | (Identity::Blue, Team::Blue)
        )
    }
}

impl PartialEq<Identity> for Team {
    fn eq(&self, other: &Identity) -> bool {
        matches!(
            (self, other),
            (Team::Red, Identity::Red) | (Team::Blue, Identity::Blue)
        )
    }
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

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum Phase {
    Clue { team: Team },
    Guess { team: Team, clue: Clue },
    End,
}

// #[derive(Clone, Debug, Serialize)]
// pub enum Action {
//     Guess { word: String },
//     Clue { clue: Clue },
//     Reveal { card: Card },
// }
/// Legal moves only
#[derive(Clone, Debug, Serialize)]
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
        let mut cards: Vec<Card> = words
            .into_iter()
            .zip(Self::IDENTITIES_ARRAY)
            .map(|(word, identity)| Card::new(word, identity))
            .collect();

        tracing::debug!("{:?}", cards);

        cards.shuffle(&mut rand::thread_rng());
        let phase = Phase::Clue { team: Team::Red };

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
        // TODO: Make this an if let
        match &self.phase {
            Phase::Clue { team } => {
                tracing::debug!("Succesfully gave clue: {:?}", &clue);
                self.phase = Phase::Guess {
                    team: team.clone(),
                    clue,
                };
                tracing::debug!("New Phase: {:?}", &self.phase);

                Ok(())
            }
            _ => Err(anyhow::anyhow!("Wrong phase")),
        }
    }

    pub fn make_guess(&mut self, word: String) -> Result<()> {
        tracing::debug!("Making guess in game_state");

        // TODO: Make this an if let
        match &mut self.phase {
            Phase::Guess { team, clue } => {
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

                if card.identity == Identity::Assassin {
                    tracing::debug!("Assassin has been guessed!");
                    self.phase = Phase::End;
                    tracing::debug!("New Phase: {:?}", &self.phase);
                    return Ok(());
                }

                if clue.remaining == 0
                    // TODO: Make better comparison between Identity and Team?
                    || (card.identity == Identity::Blue && team == &Team::Red)
                    || (card.identity == Identity::Red && team == &Team::Blue)
                    || card.identity == Identity::Bystander
                {
                    self.phase = Phase::Clue { team: team.other() };
                    tracing::debug!("New Phase: {:?}", &self.phase);
                }

                if let Some(_team) = self.check_win_state() {
                    self.phase = Phase::End;
                    tracing::debug!("New Phase: {:?}", &self.phase);
                    return Ok(());
                }

                tracing::debug!("Succesfully made guess: {word}");
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Wrong phase")),
        }
    }

    pub fn board(&self) -> &Vec<Card> {
        &self.board
    }

    pub fn check_win_state(&self) -> Option<Team> {
        let red_win = self
            .board
            .iter()
            .filter(|card| card.identity == Identity::Red)
            .all(|card| card.guessed);

        let blue_win = self
            .board
            .iter()
            .filter(|card| card.identity == Identity::Blue)
            .all(|card| card.guessed);

        match (red_win, blue_win) {
            (true, false) => Some(Team::Red),
            (false, true) => Some(Team::Blue),
            (true, true) => Some(Team::Red),
            (false, false) => None,
        }
    }

    pub fn to_hidden_board(&self) -> Vec<Card> {
        self.board
            .iter()
            .map(|card| Card {
                word: card.word.clone(),
                guessed: card.guessed,
                identity: match card.guessed {
                    true => card.identity.clone(),
                    false => Identity::Hidden,
                },
            })
            .collect()
    }

    pub fn to_hidden_game_state(&self) -> Self {
        Self {
            board: self.to_hidden_board(),
            phase: self.phase.clone(),
        }
    }

    pub fn clue(&self) -> Option<&Clue> {
        match &self.phase {
            Phase::Guess { clue, .. } => Some(clue),
            _ => None,
        }
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }
}
