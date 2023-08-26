use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use game::{game_controller::GameController, word_bank::WordBank};
use tokio::sync::RwLock;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::routes::{
    clue::post_clue,
    game::{get_game, post_game, post_game_start},
    guess::post_guess,
    root::get_root,
};

mod app_error;
mod game;
mod routes;

pub struct GameEnvironment {
    controllers: RwLock<HashMap<Uuid, GameController>>,
    word_bank: WordBank,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "donkeyglue=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let game_env = Arc::new(GameEnvironment {
        controllers: RwLock::new(HashMap::new()),
        word_bank: WordBank::new(),
    });

    // build our application with a route
    let app = Router::new()
        .route("/", get(get_root))
        .with_state(game_env.clone())
        .route("/game", post(post_game))
        .with_state(game_env.clone())
        .route("/game/:id", get(get_game))
        .with_state(game_env.clone())
        .route("/game/start/:id", post(post_game_start))
        .with_state(game_env.clone())
        .route("/guess/:id", post(post_guess))
        .with_state(game_env.clone())
        .route("/clue/:id", post(post_clue))
        .with_state(game_env.clone());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
