use serde_json::Value;
use std::fmt;
use tokio::sync::mpsc;

use crate::utils::{
    agent_builder::nodes::{
        error_node::ErrorNode,
        prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
    },
    prompts::analyst_chat_prompts::master_response_prompt::{
        master_response_system_prompt, master_response_user_prompt,
    },
};

pub enum MasterResponseAgentError {
    MissingKey,
    ObjectNotJson,
    PromptNodeError,
}

impl fmt::Display for MasterResponseAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey => write!(f, "missing_key"),
            Self::ObjectNotJson => write!(f, "object_not_json"),
            Self::PromptNodeError => write!(f, "prompt_node_error"),
        }
    }
}

pub struct MasterResponseAgentOptions {
    pub outputs: Value,
    pub message_history: Vec<Value>,
    pub datasets: String,
    pub input: String,
    pub output_sender: mpsc::Sender<Value>,
}

pub async fn master_response_agent(
    options: MasterResponseAgentOptions,
) -> Result<Value, ErrorNode> {
    // Extract required fields from outputs, defaulting to null if not found
    let action_decisions = options
        .outputs
        .get("action_decisions")
        .and_then(|v| v.as_object())
        .and_then(|obj| serde_json::to_string(obj).ok());

    let dataset_selection = options
        .outputs
        .get("dataset_selection")
        .and_then(|v| v.as_object())
        .and_then(|obj| serde_json::to_string(obj).ok());

    let first_part_of_response = options
        .outputs
        .get("first_part_of_response")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let data_metadata = options
        .outputs
        .get("data_metadata")
        .and_then(|v| v.as_object())
        .and_then(|obj| serde_json::to_string(obj).ok());

    let chart_requirements = options
        .outputs
        .get("chart_generated")
        .and_then(|v| serde_json::to_string(v).ok());

    let chart_generated = options
        .outputs
        .get("chart_requirements")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Create prompt settings
    let master_response_prompt_settings = PromptNodeSettings {
        messages: vec![
            PromptNodeMessage {
                role: "system".to_string(),
                content: master_response_system_prompt(&options.datasets),
            },
            PromptNodeMessage {
                role: "user".to_string(),
                content: master_response_user_prompt(
                    &options.input,
                    &action_decisions,
                    &dataset_selection,
                    &first_part_of_response,
                    &data_metadata,
                    &chart_generated,
                    &chart_requirements,
                ),
            },
        ],
        stream: Some(options.output_sender),
        stream_name: Some("master_response".to_string()),
        prompt_name: "master_response".to_string(),
        ..Default::default()
    };

    // Execute prompt node
    let response = match prompt_node(master_response_prompt_settings).await {
        Ok(response) => response,
        Err(e) => {
            return Err(e);
        }
    };

    // Combine master response with first part of response
    let combined_response = match (
        response.as_str(),
        options.outputs.get("first_part_of_response"),
    ) {
        (Some(master_str), Some(first_part)) => {
            if let Some(first_part_str) = first_part.as_str() {
                Value::String(format!("{}\n\n{}", first_part_str, master_str))
            } else {
                response
            }
        }
        _ => response,
    };

    Ok(combined_response)
}
