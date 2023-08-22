use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Serialize;

use crate::{app_error::AppError, GameEnvironment};

#[derive(Clone, Serialize, Debug)]
pub struct GetRootResponse {
    message: String,
}

pub async fn get_root(
    State(_game_env): State<Arc<GameEnvironment>>,
) -> Result<Json<GetRootResponse>, AppError> {
    let response = GetRootResponse {
        message: "Pong".to_string(),
    };

    Ok(Json(response))
}
