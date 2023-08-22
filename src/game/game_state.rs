use anyhow::Result;

#[derive(Clone, Debug)]
enum Identity {
    Red,
    Blue,
    Bystander,
    Assassin,
}

#[derive(Clone, Debug)]
struct Card {
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

#[derive(Clone, Debug)]
pub enum Team {
    Red,
    Blue,
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
    cards: Vec<Card>,
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

        GameState { cards, phase }
    }

    pub fn provide_clue(&mut self, clue: Clue) -> Result<()> {
        match &self.phase {
            Phase::Clue(team) => {
                self.phase = Phase::Guess(team.other(), clue);
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Wrong phase")),
        }
    }

    pub fn make_guess(&mut self, word: String) -> Result<()> {
        match &mut self.phase {
            Phase::Guess(team, clue) => {
                clue.remaining -= 1;

                if clue.remaining == 0 {
                    self.phase = Phase::Clue(team.other());
                }

                Ok(())
            }
            _ => Err(anyhow::anyhow!("Wrong phase")),
        }
    }
}
