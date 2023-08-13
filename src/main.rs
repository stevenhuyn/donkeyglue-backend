//! Run with
//!
//! ```not_rust
//! cargo run -p example-sse
//! ```

use axum::{
    extract::{Path, State, TypedHeader},
    response::sse::{Event, Sse},
    routing::{get, post},
    Router,
};

use axum_macros::debug_handler;
use futures::stream::{self, Stream};
use game::GameState;
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::routes::{clue::post_clue, guess::post_guess};

pub struct Context {
    games: RwLock<HashMap<Uuid, Arc<RwLock<GameState>>>>,
}

mod game;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_sse=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let context = Arc::new(Context {
        games: RwLock::new(HashMap::new()),
    });

    // build our application with a route
    let app = Router::new()
        .route("/game/:id", get(get_game))
        .with_state(context.clone())
        .route("/game", post(post_game))
        .with_state(context.clone())
        .route("/increment/:id", get(increment_counter))
        .with_state(context.clone())
        .route("/guess/:id", post(post_guess))
        .with_state(context.clone())
        .route("/clue/:id", post(post_clue))
        .with_state(context.clone());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn post_game(State(context): State<Arc<Context>>) -> String {
    let mut games = context.games.write().await;
    let uuid = Uuid::new_v4();
    let new_game = GameState::new();
    games.entry(uuid).or_insert(Arc::new(RwLock::new(new_game)));
    uuid.to_string()
}

async fn get_game(
    Path(game_id): Path<Uuid>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    State(context): State<Arc<Context>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

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

async fn increment_counter(Path(game_id): Path<Uuid>, State(context): State<Arc<Context>>) {
    let games = context.games.read().await;
    let game = games.get(&game_id).unwrap();
    // *game.write().await.to_string()
}
