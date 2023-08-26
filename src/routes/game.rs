use std::{sync::Arc, time::Duration};

use anyhow::{Error, Result};
use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    Json, TypedHeader,
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use tokio_stream::{wrappers::WatchStream, StreamExt};
use uuid::Uuid;

use crate::{
    app_error::AppError,
    game::game_controller::{ChannelEvent, GameController, Role},
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

pub async fn get_game(
    Path(game_id): Path<Uuid>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(game_env): State<Arc<GameEnvironment>>,
) -> impl IntoResponse {
    tracing::info!("get_game: {:?}", user_agent);

    let controllers = game_env.controllers.read().await;
    if let Some(controller) = controllers.get(&game_id) {
        let receiver = controller.sender().subscribe();
        let stream_receiver = WatchStream::new(receiver);

        let board_hidden = controller.agents().should_hide_board();

        let stream = stream_receiver.map(move |mut channel_event| match &mut channel_event {
            ChannelEvent::Playing { game_state, .. } => {
                if board_hidden {
                    *game_state = game_state.to_hidden_game_state();
                }

                Event::default().json_data(&channel_event)
            }
        });

        return Sse::new(stream)
            .keep_alive(
                KeepAlive::default()
                    .interval(Duration::from_secs(15))
                    .text(":\n\n"),
            )
            .into_response();
    }

    let err = Error::msg("Could not find the game");
    tracing::warn!("{}", err);
    AppError(err).into_response()
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
