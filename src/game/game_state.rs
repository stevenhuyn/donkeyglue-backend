enum Identity {
    Red,
    Blue,
    Bystander,
    Assassin,
}

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

/// Legal moves only
pub struct GameState {
    cards: Vec<Card>,
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

        GameState { cards }
    }
}
