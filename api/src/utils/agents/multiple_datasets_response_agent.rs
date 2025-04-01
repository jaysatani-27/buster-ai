use serde_json::{json, Value};
use std::fmt;
use tokio::sync::mpsc;

use crate::utils::{
    agent_builder::nodes::{
        error_node::ErrorNode,
        prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
    },
    prompts::generate_sql_prompts::multiple_datasets_prompt::{
        multiple_datasets_system_prompt, multiple_datasets_user_prompt,
    },
};

pub enum MultipleDatasetAgentError {
    MissingKey,
    ObjectNotJson,
    PromptNodeError,
    GenericError,
}

impl fmt::Display for MultipleDatasetAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey => write!(f, "missing_key"),
            Self::ObjectNotJson => write!(f, "object_not_json"),
            Self::PromptNodeError => write!(f, "prompt_node_error"),
            Self::GenericError => write!(f, "generic_error"),
        }
    }
}

pub struct MultipleDatasetAgentOptions {
    pub input: String,
    pub datasets: String,
    pub dataset_selector_output: Value,
    pub output_sender: mpsc::Sender<Value>,
}

pub async fn handle_multiple_datasets_agent(
    options: MultipleDatasetAgentOptions,
) -> Result<Value, ErrorNode> {
    let multiple_datasets_prompt_settings = PromptNodeSettings {
        messages: create_multiple_datasets_messages(
            &options.input,
            &options.dataset_selector_output,
        ),
        stream: Some(options.output_sender.clone()),
        stream_name: Some("dataset_breakout".to_string()),
        prompt_name: "multiple_datasets_response".to_string(),
        ..Default::default()
    };

    let multiple_datasets_response = match prompt_node(multiple_datasets_prompt_settings).await {
        Ok(Value::String(response)) => response,
        Ok(_) => {
            return Err(ErrorNode::new(
                MultipleDatasetAgentError::ObjectNotJson.to_string(),
                "Multiple datasets response is not a string".to_string(),
            ));
        }
        Err(e) => {
            return Err(e);
        }
    };

    Ok(json!({
        "name": "multiple_datasets_response",
        "response": multiple_datasets_response,
        "input": options.input,
        "dataset_selector_output": options.dataset_selector_output
    }))
}

fn create_multiple_datasets_messages(
    input: &String,
    dataset_selector_output: &Value,
) -> Vec<PromptNodeMessage> {
    let dataset_selector_output = dataset_selector_output.to_string();

    vec![
        PromptNodeMessage {
            role: "system".to_string(),
            content: multiple_datasets_system_prompt(),
        },
        PromptNodeMessage {
            role: "user".to_string(),
            content: multiple_datasets_user_prompt(input, &dataset_selector_output),
        },
    ]
}
