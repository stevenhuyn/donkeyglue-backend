use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use game::simulator::Simulator;
use tokio::sync::{watch, Mutex, RwLock};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::game::game_state::{GameState, Role};
use crate::game::seed_words::SeedWords;
use crate::routes::{
    clue::post_clue,
    game::{get_game, post_game},
    guess::post_guess,
};

pub struct Context {
    games: RwLock<HashMap<Uuid, Arc<RwLock<GameState>>>>,
    publishers: RwLock<HashMap<Uuid, watch::Sender<GameState>>>,
    simulator: Arc<Simulator>,
    seed_words: SeedWords,
}

mod game;
mod operative;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "donkeyglue=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let context = Arc::new(Context {
        games: RwLock::new(HashMap::new()),
        publishers: RwLock::new(HashMap::new()),
        simulator: Arc::new(Simulator::new(Role::Spymaster)),
        seed_words: SeedWords::new(),
    });

    tracing::info!("Building app");

    // build our application with a route
    let app = Router::new()
        .route("/game/:id", get(get_game))
        .with_state(context.clone())
        .route("/game", post(post_game))
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
