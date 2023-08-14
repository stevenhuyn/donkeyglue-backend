use std::io::BufRead;
use std::{collections::HashMap, fs::File, io, net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::game::game_state::GameState;
use crate::routes::{
    clue::post_clue,
    game::{get_game, post_game},
    guess::post_guess,
};

pub struct Context {
    games: RwLock<HashMap<Uuid, Arc<RwLock<GameState>>>>,
    seed_words: Vec<String>,
}

mod game;
mod operative;
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
        seed_words: get_seed_words(),
    });

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

fn get_seed_words() -> Vec<String> {
    let path = std::path::Path::new("assets/words.txt");
    let file = File::open(path).unwrap();
    let reader = io::BufReader::new(file);
    let words: Vec<String> = reader.lines().map_while(Result::ok).collect();
    words
}
