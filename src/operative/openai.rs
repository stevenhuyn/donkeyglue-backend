use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use async_trait::async_trait;
use regex::Regex;

use crate::game::game_state::GameState;

use super::Operative;

pub struct OpenaiOperative {
    client: Client<OpenAIConfig>,
}

impl OpenaiOperative {
    pub fn new() -> Self {
        OpenaiOperative {
            client: Client::new(),
        }
    }
}

const OPERATIVE_STEP_1: &str = r#"
You are an expert player of the game Codenames. 
Currently you are playing as the operative role.
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
        "confidence": "CONFIDENCE IN THE GUESS"
    },
    ...
]
```
"#;
#[async_trait]
impl Operative for OpenaiOperative {
    async fn make_guess(&self, game_state: &GameState) -> String {
        tracing::info!("Openai Operative making guess");

        let clue = format!("{:?}", game_state);
        let board = serde_json::to_string(&game_state.get_hidden_board()).unwrap();
        let system_prompt = OPERATIVE_STEP_1
            .replace("<BOARD>", &board)
            .replace("<CLUE>", &clue);

        tracing::info!("Openai Operative first prompt: {system_prompt}");

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

        tracing::debug!("Openai Operative response 1: {response_content}");

        let system_prompt = OPERATIVE_STEP_2.replace("<CHAIN>", &response_content);

        tracing::info!("Openai Operative second prompt: {system_prompt}");

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

        tracing::info!("Openai Operative second 2: {system_prompt}");

        let re = Regex::new(r"\[[^\]]*\]").unwrap();

        let json_guesses = re
            .captures(&response_content)
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
            .to_string();

        tracing::info!("Openai Operative Guesses: {json_guesses}");

        json_guesses
    }
}
