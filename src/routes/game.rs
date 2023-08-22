use std::sync::Arc;

use axum::extract::State;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{game::game_controller::GameController, GameEnvironment};

pub async fn post_game(State(game_env): State<Arc<GameEnvironment>>) -> String {
    tracing::info!("post_game");

    let uuid = Uuid::new_v4();
    let words = game_env.word_bank.get_word_set(25);
    let game_controller = GameController::new(words);

    {
        let mut game_controllers = game_env.game_controllers.write().await;
        game_controllers
            .entry(uuid)
            .or_insert(Arc::new(RwLock::new(game_controller)));
    }

    uuid.to_string()
}
