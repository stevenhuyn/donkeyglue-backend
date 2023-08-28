use itertools::Itertools;

use crate::game::game_state::Card;

pub fn board_string(board: &[Card]) -> String {
    board
        .chunks(5)
        .map(|chunk| chunk.iter().map(|card| card.to_string()).join(","))
        .join("\n")
}
