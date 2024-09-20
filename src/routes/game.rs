use std::sync::Arc;

use anyhow::{Error, Result};
use axum::{
    extract::{Path, State},
    Json,
};
use axum_extra::TypedHeader;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_error::AppError,
    game::game_controller::{GameController, Role},
    game::game_state::GameState,
    GameEnvironment,
};

#[derive(Deserialize, Debug)]
pub struct PostGameRequest {
    role: Role,
}

#[derive(Serialize, Debug)]
pub struct PostGameResponse {
    game_id: Uuid,
}

#[debug_handler]
pub async fn post_game(
    State(game_env): State<Arc<GameEnvironment>>,
    Json(payload): Json<PostGameRequest>,
) -> Result<Json<PostGameResponse>, AppError> {
    tracing::info!("post_game");

    let game_id = Uuid::new_v4();
    let words = game_env.word_bank.get_word_set(25);
    let controller = GameController::new(payload.role, words);

    {
        let mut controllers = game_env.controllers.write().await;
        controllers.entry(game_id).or_insert(controller);
    }

    Ok(Json(PostGameResponse { game_id }))
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum GetGameResponse {
    Playing {
        #[serde(rename = "gameState")]
        game_state: GameState,
        role: Role,
    },
}

pub async fn get_game(
    Path(game_id): Path<Uuid>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(game_env): State<Arc<GameEnvironment>>,
) -> Result<Json<GetGameResponse>, AppError> {
    tracing::info!("get_game: {:?}", user_agent);

    let controllers = game_env.controllers.read().await;
    if let Some(controller) = controllers.get(&game_id) {
        let game_data: GetGameResponse = controller.game_data().await;
        return Ok(Json(game_data));
    }

    let err = Error::msg("Could not find the game");
    tracing::warn!("{}", err);
    Err(AppError(err))
}

#[debug_handler]
pub async fn post_game_start(
    Path(game_id): Path<Uuid>,
    State(game_env): State<Arc<GameEnvironment>>,
) -> Result<(), AppError> {
    tracing::info!("post_game_start");

    let controllers = game_env.controllers.read().await;
    if let Some(controller) = controllers.get(&game_id) {
        controller.step_until_input().await;
        return Ok(());
    }

    let err = Error::msg("Could not find the game");
    tracing::warn!("{}", err);
    Err(AppError(err))
}
