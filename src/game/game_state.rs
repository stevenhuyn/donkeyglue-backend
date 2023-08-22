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

        let mut game_state = GameState { cards, phase };
        tracing::debug!("{:?}", game_state.phase);

        let _ = game_state.provide_clue(Clue {
            word: "test".to_string(),
            count: 1,
            remaining: 3,
        });
        tracing::debug!("{:?}", game_state.phase);

        let _ = game_state.provide_clue(Clue {
            word: "test".to_string(),
            count: 1,
            remaining: 1,
        });
        tracing::debug!("{:?}", game_state.phase);

        let _ = game_state.make_guess("bruh1".to_string());
        tracing::debug!("{:?}", game_state.phase);
        let _ = game_state.make_guess("bruh2".to_string());
        tracing::debug!("{:?}", game_state.phase);

        let _ = game_state.make_guess("bruh3".to_string());
        tracing::debug!("{:?}", game_state.phase);

        let _ = game_state.make_guess("bruh4".to_string());
        tracing::debug!("{:?}", game_state.phase);

        game_state
    }

    pub fn provide_clue(&mut self, clue: Clue) -> Result<()> {
        match self.phase {
            Phase::Clue(Team::Red) => {
                self.phase = Phase::Guess(Team::Red, clue);
                Ok(())
            }
            Phase::Clue(Team::Blue) => {
                self.phase = Phase::Guess(Team::Blue, clue);
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
                    self.phase = Phase::Clue(match team {
                        Team::Red => Team::Blue,
                        Team::Blue => Team::Red,
                    });
                }

                Ok(())
            }
            _ => Err(anyhow::anyhow!("Wrong phase")),
        }
    }
}
