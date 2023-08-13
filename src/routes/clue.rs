use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{game::Clue, Context};

#[derive(Clone, Deserialize, Debug)]
pub struct GuessRequest {
    word: String,
    number: u8,
}

pub async fn post_clue(
    State(context): State<Arc<Context>>,
    Path(game_id): Path<Uuid>,
    Json(payload): Json<GuessRequest>,
) {
    let games = context.games.read().await;
    let game = games.get(&game_id).unwrap();
    let mut data = game.write().await;
    let clue = Clue::new(payload.word, payload.number);
    data.provide_clue(clue);
}
