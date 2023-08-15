use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    extract::{Path, State},
    response::{sse::Event, Sse},
    TypedHeader,
};
use futures::{stream, Stream};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{game::game_state::GameState, Context};

pub async fn post_game(State(context): State<Arc<Context>>) -> String {
    let mut games = context.games.write().await;
    let uuid = Uuid::new_v4();
    let new_game = GameState::new(&context.seed_words);
    games.entry(uuid).or_insert(Arc::new(RwLock::new(new_game)));

    uuid.to_string()
}

pub async fn get_game(
    Path(game_id): Path<Uuid>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(context): State<Arc<Context>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    let context_clone = context.clone();
    tokio::spawn(async move {
        let games = context_clone.games.read().await;
        let game = games.get(&game_id).unwrap();
        let game = &mut game.write().await;
        let simulator = &context_clone.simulator;
        simulator.step_until_player(game).await;
    });

    // TODO: Convert to a tokio::watch::Receiver
    let stream = stream::unfold(context, move |context| async move {
        let data = {
            let games = context.games.read().await;
            let game = games.get(&game_id).unwrap();
            let data = game.read().await;
            format!("{:?}", data)
        };

        Some((Event::default().data(data), context))
    })
    .map(Ok)
    .throttle(Duration::from_secs(3));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
