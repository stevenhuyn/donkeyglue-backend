use std::sync::Arc;

use axum::{extract::State, Json};
use serde::Serialize;

use crate::{app_error::AppError, Context};

#[derive(Clone, Serialize, Debug)]
pub struct GetRootResponse {
    message: String,
}

pub async fn get_root(
    State(context): State<Arc<Context>>,
) -> Result<Json<GetRootResponse>, AppError> {
    let response = GetRootResponse {
        message: "Pong".to_string(),
    };

    Ok(Json(response))
}
