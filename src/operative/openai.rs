use async_trait::async_trait;
use llm_chain::parameters;
use llm_chain::step::Step;
use llm_chain::traits::Executor as ExecutorTrait;
use llm_chain::{chains::sequential::Chain, prompt};
use llm_chain_openai::chatgpt::Executor;

use crate::game::game_state::GameState;

use super::Operative;

pub struct OpenaiOperative;

const OPERATIVE_STEP_1: &str = r#"
You are an expert player of the game Codenames. 
Currently you are playing as the operative role.
Discuss your options and what your guesses should be based on the current game board and clue.
{{board}}
{{clue}}
"#;

const OPERATIVE_STEP_2: &str = r#"
You are an agent who distills the guesses from a body of text that discusses of the clue and game state for the game Codenames.

Summarize the following into a JSON array of guesses:
{{text}}

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
        // Create a new ChatGPT executor with the default settings
        let exec = Executor::new().unwrap();

        // Create a chain of steps with two prompts
        let chain: Chain = Chain::new(vec![
            Step::for_prompt_template(prompt!(system: OPERATIVE_STEP_1)),
            Step::for_prompt_template(prompt!(system: OPERATIVE_STEP_2)),
        ]);

        // Run the chain with the provided parameters
        let clue = format!("{:?}", game_state);
        let board = serde_json::to_string(&game_state.get_hidden_board()).unwrap();
        let res = chain
            .run(parameters!("board" => board, "clue" => clue), &exec)
            .await
            .unwrap();

        res.to_immediate().await.unwrap().as_content().to_string()
    }
}
