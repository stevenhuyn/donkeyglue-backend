use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        CreateChatCompletionRequestArgs, ResponseFormat,
    },
    Client,
};
use backoff::ExponentialBackoffBuilder;
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::game::{
    agent::utils::board_string,
    game_state::{Clue, GameState, Identity, Team},
};

type OpenaiOperativeResponse = Vec<OpenaiOperativeGuess>;

#[derive(Debug, Clone, Deserialize)]
struct OpenaiOperativeGuess {
    guess: String,
    #[serde(rename = "justification")]
    _justification: String,
    #[serde(rename = "confidence")]
    _confidence: f32,
}

#[derive(Deserialize, Serialize, Debug)]
struct OpenaiSpymasterResponse {
    word: String,
    number: u8,
    justification: String,
    associations: Vec<String>,
}

// TODO: Set GPT version
pub struct ChatGpt {
    client: Client<OpenAIConfig>,
    team: Team,
}

const OPERATIVE_STEP_1: &str = r#"
You are an expert player of the game Codenames.
You are playing as the operative role on the <TEAM> team.
Discuss your options and what your guesses should be based on the current game board and clue.
<BOARD>
<CLUE>

The words left to guess are
<REMAINING>
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

const SPYMASTER_STEP_1: &str = r#"
You are an expert player of the game Codenames.
You are playing as the spymaster role for the <TEAM> team.
Discuss your options and what would be the best clue based on the current game board.
<BOARD>

The remaining cards you are trying to get your operative to guess are:
<REMAINING>
"#;

const SPYMASTER_STEP_2: &str = r#"
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

impl ChatGpt {
    pub fn new(team: Team) -> Self {
        let backoff = ExponentialBackoffBuilder::new()
            .with_max_elapsed_time(Some(std::time::Duration::from_secs(60)))
            .build();

        Self {
            client: Client::new().with_backoff(backoff),
            team,
        }
    }

    pub async fn try_gen_guesses(&self, game_state: &GameState) -> Option<Vec<String>> {
        tracing::info!("Openai Operative making guess");

        let clue = format!("{:?}", game_state.clue().unwrap());
        let hidden_board = game_state.to_hidden_board();
        let board = board_string(&hidden_board);
        let remaining_cards = hidden_board
            .into_iter()
            .filter(|card| card.identity() == &Identity::Hidden)
            .map(|card| card.word().to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let system_prompt = OPERATIVE_STEP_1
            .replace("<TEAM>", &self.team.to_string())
            .replace("<BOARD>", &board)
            .replace("<CLUE>", &clue)
            .replace("<REMAINING>", &remaining_cards);

        // tracing::info!("Openai Operative first prompt: {system_prompt}");

        let messages: [ChatCompletionRequestMessage; 1] =
            [ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .unwrap()
                .into()];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-4o")
            .messages(messages)
            .response_format(ResponseFormat::JsonObject)
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

        let messages: [ChatCompletionRequestMessage; 1] =
            [ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .unwrap()
                .into()];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-4o")
            .messages(messages)
            .response_format(ResponseFormat::JsonObject)
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

        tracing::info!("Openai Operative Guesses: {json_guesses}");

        let guesses = serde_json::from_str::<OpenaiOperativeResponse>(&json_guesses)
            .unwrap()
            .into_iter()
            .map(|guess| guess.guess)
            .collect();

        tracing::debug!("Guess: {:?}", guesses);
        Some(guesses)
    }

    pub async fn try_gen_clue(&self, game_state: &GameState) -> Option<Clue> {
        tracing::info!("Openai Spymaster creating clue");

        let board = board_string(game_state.board());
        let remaining_cards: String = game_state
            .board()
            .iter()
            .filter(|card| card.identity() == &self.team && !card.guessed())
            .map(|card| card.word())
            .join(", ");

        let system_prompt = SPYMASTER_STEP_1
            .replace("<TEAM>", &self.team.to_string())
            .replace("<BOARD>", &board)
            .replace("<REMAINING>", &remaining_cards);

        tracing::info!("Openai Spymaster first prompt: {system_prompt}");

        let messages: [ChatCompletionRequestMessage; 1] =
            [ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .unwrap()
                .into()];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-4o")
            .messages(messages)
            .response_format(ResponseFormat::JsonObject)
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

        let system_prompt = SPYMASTER_STEP_2.replace("<CHAIN>", &response_content);

        // tracing::info!("Openai Spymaster second prompt: {system_prompt}");

        let messages: [ChatCompletionRequestMessage; 1] =
            [ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .unwrap()
                .into()];

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-4o")
            .messages(messages)
            .response_format(ResponseFormat::JsonObject)
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

        // tracing::info!("Openai Spymaster second 2: {system_prompt}");

        let re = Regex::new(r"\{[^\}]*\}").unwrap();

        let json_guesses = re
            .captures(&response_content)
            .unwrap()
            .get(0)
            .unwrap()
            .as_str()
            .to_string();

        // tracing::info!("Openai Spymaster Guesses: {json_guesses}");
        let clue: OpenaiSpymasterResponse = serde_json::from_str(&json_guesses).unwrap();

        tracing::debug!("Clue Justifications: {clue:?}");

        let clue = Clue::new(clue.word, clue.number);
        tracing::info!("Openai Spymaster Clue: {clue:?}");
        Some(clue)
    }
}
