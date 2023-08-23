use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use async_trait::async_trait;
use backoff::ExponentialBackoffBuilder;
use regex::Regex;
use serde::Deserialize;

use crate::game::game_state::{GameState, Team};

use super::Operative;

pub struct OpenaiOperative {
    client: Client<OpenAIConfig>,
    team: Team,
}

impl OpenaiOperative {
    pub fn new(team: Team) -> Self {
        let backoff = ExponentialBackoffBuilder::new()
        .
            .with_max_elapsed_time(Some(std::time::Duration::from_secs(60)))
            .build();

        Self {
            client: Client::new().with_backoff(backoff),
            team,
        }
    }
}

const OPERATIVE_STEP_1: &str = r#"
You are an expert player of the game Codenames. 
Currently you are playing as the operative role on the <TEAM> team.
Discuss your options and what your guesses should be based on the current game board and clue.
<BOARD>
<CLUE>
"#;

const OPERATIVE_STEP_2: &str = r#"
You are an agent who distills the guesses from a body of text that discusses of the clue and game state for the game Codenames.

Summarize the following into a JSON array of guesses:
<CHAIN>

The format of the response should be an array of guesses with justification in order of priority:

```json
[
    {
        "guess": "THE GUESS",
        "justification": "WHY THE GUESS IS CORRECT",
        "confidence": [0-1]
    },
    ...
]
```
"#;

type OpenaiOperativeResponse = Vec<OpenaiOperativeGuess>;

#[derive(Debug, Clone, Deserialize)]
struct OpenaiOperativeGuess {
    guess: String,
    justification: String,
    confidence: f32,
}

#[async_trait]
impl Operative for OpenaiOperative {
    async fn try_gen_guesses(&self, game_state: &GameState) -> Option<Vec<String>> {
        tracing::info!("Openai Operative making guess");

        let clue = format!("{:?}", game_state.get_clue().unwrap());
        let board = serde_json::to_string(&game_state.get_hidden_board()).unwrap();
        let system_prompt = OPERATIVE_STEP_1
            .replace("<TEAM>", &self.team.to_string())
            .replace("<BOARD>", &board)
            .replace("<CLUE>", &clue);

        // tracing::info!("Openai Operative first prompt: {system_prompt}");

        let messages = [ChatCompletionRequestMessageArgs::default()
            .role(Role::System)
            .content(&system_prompt)
            .build()
            .unwrap()];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-4")
            .messages(messages)
            .build()
            .unwrap();

        let openai_response = self.client.chat().create(request).await.unwrap();

        let response_content = openai_response
            .choices
            .first()
            .unwrap()
            .message
            .content
            .clone()
            .unwrap();

        // tracing::debug!("Openai Operative response 1: {response_content}");

        let system_prompt = OPERATIVE_STEP_2.replace("<CHAIN>", &response_content);

        // tracing::info!("Openai Operative second prompt: {system_prompt}");

        let messages = [ChatCompletionRequestMessageArgs::default()
            .role(Role::System)
            .content(&system_prompt)
            .build()
            .unwrap()];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-3.5-turbo")
            .messages(messages)
            .build()
            .unwrap();

        let openai_response = self.client.chat().create(request).await.unwrap();

        let response_content = openai_response
            .choices
            .first()
            .unwrap()
            .message
            .content
            .clone()
            .unwrap();

        // tracing::info!("Openai Operative second 2: {system_prompt}");

        let re = Regex::new(r"\[[^\]]*\]").unwrap();

        let json_guesses = re
            .captures(&response_content)
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
            .to_string();

        // tracing::info!("Openai Operative Guesses: {json_guesses}");

        let guesses = serde_json::from_str::<OpenaiOperativeResponse>(&json_guesses)
            .unwrap()
            .into_iter()
            .map(|guess| guess.guess)
            .collect();

        tracing::debug!("Guess: {:?}", guesses);
        Some(guesses)
    }
}
