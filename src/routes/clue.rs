use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use uuid::Uuid;

use crate::{game::game_state::Clue, Context};

#[derive(Clone, Deserialize, Debug)]
pub struct GuessRequest {
    word: String,
    number: u8,
}

#[debug_handler]
pub async fn post_clue(
    State(context): State<Arc<Context>>,
    Path(game_id): Path<Uuid>,
    Json(payload): Json<GuessRequest>,
) {
    tracing::info!("post_clue");
    tokio::spawn(async move {
        let games = context.games.read().await;
        let game = games.get(&game_id).unwrap();
        let game = &mut game.write().await;
        let clue = Clue::new(payload.word, payload.number);
        let simulator = &context.simulator;
        simulator.provide_clue(game, clue).await;
    });
}
