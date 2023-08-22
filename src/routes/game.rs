use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    extract::{Path, State},
    response::{sse::Event, Sse},
    TypedHeader,
};
use futures::{stream, Stream};
use tokio::sync::{watch, RwLock};
use tokio_stream::{wrappers::WatchStream, StreamExt};
use uuid::Uuid;

use crate::{game::game_state::GameState, Context};

pub async fn post_game(State(context): State<Arc<Context>>) -> String {
    tracing::info!("post_game");

    let mut games = context.games.write().await;
    let uuid = Uuid::new_v4();
    let new_game = GameState::new(&context.seed_words);
    games
        .entry(uuid)
        .or_insert(Arc::new(RwLock::new(new_game.clone())));

    let (sender, _receiver1) = watch::channel(new_game.clone());
    let mut publishers = context.publishers.write().await;
    publishers.entry(uuid).or_insert(sender);

    uuid.to_string()
}

pub async fn get_game(
    Path(game_id): Path<Uuid>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(context): State<Arc<Context>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("get_game: {:?}", user_agent);

    let context_clone = context.clone();
    tokio::spawn(async move {
        let games = context_clone.games.read().await;
        let game = games.get(&game_id).unwrap();
        let game = &mut game.write().await;
        let simulator = &context_clone.simulator;
        simulator.step_until_player(game).await;
    });

    let publishers = context.publishers.read().await;
    let publisher = publishers.get(&game_id).unwrap();
    let receiver = publisher.subscribe();
    let stream_receiver = WatchStream::new(receiver);

    let stream = stream_receiver
        .map(|game_state| Ok(Event::default().data(format!("{:?}", game_state))))
        .throttle(Duration::from_secs(3));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
