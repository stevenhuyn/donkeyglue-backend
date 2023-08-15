use async_trait::async_trait;
use llm_chain::parameters;
use llm_chain::step::Step;
use llm_chain::traits::Executor as ExecutorTrait;
use llm_chain::{chains::sequential::Chain, prompt};
use llm_chain_openai::chatgpt::Executor;


use crate::game::game_state::GameState;

use super::Operative;

pub struct OpenaiOperative;


#[async_trait]
impl Operative for OpenaiOperative {
    async fn make_guess(&self, game_state: &GameState) -> String {
        // Create a new ChatGPT executor with the default settings
        let exec = Executor::new().unwrap();

        // Create a chain of steps with two prompts
        let chain: Chain = Chain::new(vec![
            // First step: make a personalized birthday email
            Step::for_prompt_template(
                prompt!(r#"You are a codenames Operative for the game Codenames. You must guess the correct word based on the given clue"#, "Discuss your options and what your guesses should be based on these clues:\n {{clue}}")
            ),
            // Second step: summarize the email into a tweet. Importantly, the text parameter becomes the result of the previous prompt.
            Step::for_prompt_template(
                prompt!( "You are an agent who distills the proper guess for the Codenames guess", "Summarize this text into a JSON array of guesses:\n {{text}}")
            )
        ]);

        // Run the chain with the provided parameters
        let clue = format!("{:?}", game_state);
        let res = chain
            .run(
                // Create a Parameters object with key-value pairs for the placeholders
                parameters!("clue" => clue),
                &exec,
            )
            .await
            .unwrap();

        res.to_immediate().await.unwrap().as_content().to_string()
    }
}
