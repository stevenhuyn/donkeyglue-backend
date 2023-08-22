use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use game::{game_controller::GameController, word_bank::WordBank};
use tokio::sync::RwLock;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::routes::{game::post_game, root::get_root};

mod app_error;
mod game;
mod routes;

pub struct GameEnvironment {
    game_controllers: RwLock<HashMap<Uuid, Arc<RwLock<GameController>>>>,
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
        game_controllers: RwLock::new(HashMap::new()),
        word_bank: WordBank::new(),
    });

    tracing::info!("Building app");

    // build our application with a route
    let app = Router::new()
        .route("/", get(get_root))
        .with_state(game_env.clone())
        .route("/game", post(post_game))
        .with_state(game_env.clone());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
