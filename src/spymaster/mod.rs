pub trait Spymaster {
    fn give_clue(&self, game_state: GameState) -> Clue;
}
