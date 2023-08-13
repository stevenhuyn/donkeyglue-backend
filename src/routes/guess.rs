use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::Context;

pub struct GuessRequest {}

pub async fn post_game(
    State(context): State<Arc<Context>>,
    Path(game_id): Path<Uuid>,
    Json(payload): Json<GuessRequest>,
) {
    let games = context.games.read().await;
    let game = games.get(&game_id).unwrap();
    let data = game.write().await;
}
