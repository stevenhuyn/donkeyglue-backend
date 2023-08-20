use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    game::game_state::{Clue, GameState, Team},
    routes::clue,
};

use super::Spymaster;

pub struct OpenaiSpymaster {
    client: Client<OpenAIConfig>,
    team: Team,
}

impl OpenaiSpymaster {
    pub fn new(team: Team) -> Self {
        Self {
            client: Client::new(),
            team,
        }
    }
}

const OPERATIVE_STEP_1: &str = r#"
You are an expert player of the game Codenames. 
You are playing as the spymaster role for the <TEAM> team.
Discuss your options and what would be the best clue based on the current game board.
<BOARD>
"#;

const OPERATIVE_STEP_2: &str = r#"
You are an agent who distills the clue from a body of text that discusses the best clue from the given game state for the game Codenames.

Summarize the following into a JSON object of a clue:
<CHAIN>

The format of the response should be a JSON object of the following format

```json
{
    "word": "<clue word>",
    "number": <number of codenames associated with the clue word>
    "justification": "<why is this clue good>",
    "associations": [<array of codenames that the clue word is associated with (doesn't have to be same length as `number`)>]
}
```
"#;

#[derive(Deserialize, Serialize)]
struct OpenaiSpymasterResponse {
    word: String,
    number: u8,
    justification: String,
    associations: Vec<String>,
}

#[async_trait]
impl Spymaster for OpenaiSpymaster {
    async fn provide_clue(&self, game_state: &GameState) -> Option<Clue> {
        tracing::info!("Openai Spymaster creating clue");

        let board = serde_json::to_string(game_state.get_board()).unwrap();
        let system_prompt = OPERATIVE_STEP_1
            .replace("<TEAM>", &self.team.to_string())
            .replace("<BOARD>", &board);

        // tracing::info!("Openai Spymaster first prompt: {system_prompt}");

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

        // tracing::debug!("Openai Spymaster response 1: {response_content}");

        let system_prompt = OPERATIVE_STEP_2.replace("<CHAIN>", &response_content);

        // tracing::info!("Openai Spymaster second prompt: {system_prompt}");

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

        tracing::info!("Openai Spymaster second 2: {system_prompt}");

        let re = Regex::new(r"\{[^\}]*\}").unwrap();

        let json_guesses = re
            .captures(&response_content)
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
            .to_string();

        let clue: OpenaiSpymasterResponse = serde_json::from_str(&json_guesses).unwrap();
        let clue = Clue::new(clue.word, clue.number);
        tracing::info!("Openai Operative Clue: {clue:?}");
        Some(clue)
    }
}
