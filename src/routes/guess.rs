use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::Context;

#[derive(Clone, Deserialize, Debug)]
pub struct GuessRequest {
    guess: String,
}

pub async fn post_guess(
    State(context): State<Arc<Context>>,
    Path(game_id): Path<Uuid>,
    Json(payload): Json<GuessRequest>,
) {
    let games = context.games.read().await;
    let game = games.get(&game_id).unwrap();
    let mut data = game.write().await;
    data.make_guess(payload.guess);
}
