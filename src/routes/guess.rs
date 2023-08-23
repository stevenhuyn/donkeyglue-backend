use std::sync::Arc;

use anyhow::Error;
use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
use serde::Deserialize;
use uuid::Uuid;

use crate::{app_error::AppError, GameEnvironment};

#[derive(Clone, Deserialize, Debug)]
pub struct PostGuessRequest {
    guess: String,
}

#[debug_handler]
pub async fn post_guess(
    Path(game_id): Path<Uuid>,
    State(game_env): State<Arc<GameEnvironment>>,
    Json(payload): Json<PostGuessRequest>,
) -> Result<(), AppError> {
    tracing::info!("post_game");

    let game_env_clone = game_env.clone();
    let controllers = game_env.controllers.read().await;
    if let Some(controller) = controllers.get(&game_id) {
        let res = controller.player_guess(payload.guess).await;

        tokio::spawn(async move {
            let controllers = game_env_clone.controllers.read().await;
            if let Some(controller) = controllers.get(&game_id) {
                controller.step_until_input().await;
            }
        });

        return res.ok_or_else(|| {
            let err = Error::msg("Could not make guess");
            tracing::warn!("{}", err);
            AppError(err)
        });
    }

    let err = Error::msg("Could not find the game");
    tracing::warn!("{}", err);
    Err(AppError(err))
}
