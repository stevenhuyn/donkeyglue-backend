use std::sync::Arc;

use anyhow::Error;
use axum::extract::{Path, State};
use axum_macros::debug_handler;
use uuid::Uuid;

use crate::{app_error::AppError, GameEnvironment};

#[debug_handler]
pub async fn post_guess(
    Path(game_id): Path<Uuid>,
    State(game_env): State<Arc<GameEnvironment>>,
) -> Result<(), AppError> {
    tracing::info!("post_game");

    let controllers = game_env.controllers.read().await;
    if let Some(controller) = controllers.get(&game_id) {
        controller
            .read()
            .await
            .get_sender()
            .send_if_modified(|_game_state| true);

        return Ok(());
    }

    let err = Error::msg("Could not find the game");
    tracing::warn!("{}", err);
    Err(AppError(err))
}
