use serde_json::{json, Value};
use std::fmt;
use tokio::sync::mpsc;

use crate::utils::{
    agent_builder::nodes::{
        error_node::ErrorNode,
        prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
    },
    prompts::custom_response_prompts::custom_response_prompt::{
        custom_response_system_prompt, custom_response_user_prompt,
    },
};

pub enum CustomResponseAgentError {
    MissingKey,
    ObjectNotJson,
    PromptNodeError,
    GenericError,
}

impl fmt::Display for CustomResponseAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey => write!(f, "missing_key"),
            Self::ObjectNotJson => write!(f, "object_not_json"),
            Self::PromptNodeError => write!(f, "prompt_node_error"),
            Self::GenericError => write!(f, "generic_error"),
        }
    }
}

pub struct CustomResponseAgentOptions {
    pub input: String,
    // This is a string of all the dataset DDLs.
    pub datasets: String,
    pub orchestrator_output: String,
    pub output_sender: mpsc::Sender<Value>,
}

pub async fn custom_response_agent(
    options: CustomResponseAgentOptions,
) -> Result<Value, ErrorNode> {
    // Create prompt settings for custom response
    let custom_response_prompt_settings = PromptNodeSettings {
        messages: create_custom_response_messages(
            &options.input,
            &options.datasets,
            &options.orchestrator_output,
        ),
        stream: Some(options.output_sender.clone()),
        stream_name: Some("custom_response".to_string()),
        prompt_name: "custom_response".to_string(),
        ..Default::default()
    };

    // Get response from LLM
    let custom_response = match prompt_node(custom_response_prompt_settings).await {
        Ok(Value::String(response)) => response,
        Ok(_) => {
            return Err(ErrorNode::new(
                CustomResponseAgentError::ObjectNotJson.to_string(),
                "Custom response is not a string".to_string(),
            ));
        }
        Err(e) => {
            return Err(e);
        }
    };

    // Return final response object
    Ok(json!({
        "name": "custom_response",
        "response": custom_response,
        "input": options.input,
        "orchestrator_output": options.orchestrator_output
    }))
}

fn create_custom_response_messages(
    input: &String,
    datasets: &String,
    orchestrator_output: &String,
) -> Vec<PromptNodeMessage> {
    vec![
        PromptNodeMessage {
            role: "system".to_string(),
            content: custom_response_system_prompt(datasets, input, orchestrator_output),
        },
        PromptNodeMessage {
            role: "user".to_string(),
            content: custom_response_user_prompt(input, orchestrator_output),
        },
    ]
}
