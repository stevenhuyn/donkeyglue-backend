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
pub struct GuessRequest {
    guess: String,
}

#[debug_handler]
pub async fn post_guess(
    Path(game_id): Path<Uuid>,
    State(game_env): State<Arc<GameEnvironment>>,
    Json(payload): Json<GuessRequest>,
) -> Result<(), AppError> {
    tracing::info!("post_game");

    let controllers = game_env.controllers.read().await;
    if let Some(controller) = controllers.get(&game_id) {
        {
            let mut game_state = controller.get_game_state().write().await;
            if let Ok(()) = game_state.make_guess(payload.guess) {
                // TODO: NOrmally we'd do this
                // controller.get_sender().send_if_modified(|_game_state| true);
            }
        }

        controller.get_sender().send_if_modified(|_game_state| true);
        return Ok(());
    }

    let err = Error::msg("Could not find the game");
    tracing::warn!("{}", err);
    Err(AppError(err))
}
