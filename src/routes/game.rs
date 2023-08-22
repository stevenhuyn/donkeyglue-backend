use std::{convert::Infallible, sync::Arc, time::Duration};

use anyhow::{anyhow, Error, Result};
use axum::{
    extract::{Path, State},
    response::{sse::Event, Sse},
    Json, TypedHeader,
};
use axum_macros::debug_handler;
use futures::Stream;
use serde::Serialize;
use tokio::sync::RwLock;
use tokio_stream::{wrappers::WatchStream, StreamExt};
use uuid::Uuid;

use crate::{app_error::AppError, game::game_controller::GameController, GameEnvironment};

#[derive(Serialize, Debug)]
pub struct PostGameResponse {
    game_id: Uuid,
}

#[debug_handler]
pub async fn post_game(
    State(game_env): State<Arc<GameEnvironment>>,
) -> Result<Json<PostGameResponse>, AppError> {
    tracing::info!("post_game");

    let game_id = Uuid::new_v4();
    let words = game_env.word_bank.get_word_set(25);
    let controller = GameController::new(words);

    {
        let mut controllers = game_env.controllers.write().await;
        controllers
            .entry(game_id)
            .or_insert(Arc::new(RwLock::new(controller)));
    }

    Ok(Json(PostGameResponse { game_id }))
}

pub async fn get_game(
    Path(game_id): Path<Uuid>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(game_env): State<Arc<GameEnvironment>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    tracing::info!("get_game: {:?}", user_agent);

    let controllers = game_env.controllers.read().await;
    if let Some(controller) = controllers.get(&game_id) {
        let receiver = controller.read().await.get_sender().subscribe();
        let stream_receiver = WatchStream::new(receiver);

        let stream = stream_receiver
            .map(|game_state| Ok(Event::default().data(format!("{:?}", game_state))))
            .throttle(Duration::from_secs(3));

        return Ok(Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(1))
                .text("keep-alive-text"),
        ));
    }

    let err = Error::msg("Could not find the game");
    tracing::warn!("{}", err);
    Err(AppError(err))
}
