use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use axum::{
    http::{header::CONTENT_TYPE, Method},
    routing::{get, post},
    Router,
};
use game::{game_controller::GameController, word_bank::WordBank};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
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
    let _ = dotenvy::dotenv();
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

    let env = env::var("ENV").expect("No ENV=prod|dev environment variable found");
    tracing::debug!("env: {:?}", env);
    if (env != "prod") && (env != "dev") {
        panic!("ENV must be either prod or dev");
    }

    let openai_env = env::var("OPENAI_API_KEY");
    tracing::debug!("OPENAI_API_KEY exists: {:?}", openai_env.is_ok());

    let origins = match env.as_str() {
        "dev" => ["https://localhost:5173".parse().unwrap()],
        "prod" => ["https://donkeyglue.stevenhuyn.com".parse().unwrap()],
        _ => unreachable!(),
    };

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE]);

    let host = match env.as_str() {
        "dev" => [127, 0, 0, 1],
        "prod" => [0, 0, 0, 0],
        _ => unreachable!(),
    };

    let port_string = env::var("PORT").unwrap_or_else(|_| String::from("3000"));
    let port = port_string.parse::<u16>().unwrap_or(3000);
    let addr = SocketAddr::from((host, port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

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
        .with_state(game_env.clone())
        .layer(cors);

    tracing::debug!("listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
