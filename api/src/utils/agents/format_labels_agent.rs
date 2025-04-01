use serde_json::Value;
use std::fmt;
use tokio::sync::mpsc;

use crate::utils::{
    agent_builder::nodes::{
        error_node::ErrorNode,
        prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
    },
    prompts::modify_visualization_prompts::format_label_prompt::{
        format_label_system_prompt, format_label_user_prompt,
    },
};

pub enum FormatLabelsAgentError {
    ObjectNotJson,
    MissingKey,
    PromptNodeError,
}

impl fmt::Display for FormatLabelsAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey => write!(f, "missing_key"),
            Self::ObjectNotJson => write!(f, "object_not_json"),
            Self::PromptNodeError => write!(f, "prompt_node_error"),
        }
    }
}

#[derive(Clone)]
pub struct FormatLabelsAgentOptions {
    pub format_label_instruction: String,
    pub chart_config: String,
    pub sql_statement: String,
    pub data_metadata: String,
    pub output_sender: mpsc::Sender<Value>,
}

pub struct FormatLabelsAgentResult {
    pub format_result: Value,
}

pub async fn format_labels_agent(options: FormatLabelsAgentOptions) -> Result<Value, ErrorNode> {
    let format_label_settings = PromptNodeSettings {
        messages: vec![
            PromptNodeMessage {
                role: "system".to_string(),
                content: format_label_system_prompt(),
            },
            PromptNodeMessage {
                role: "user".to_string(),
                content: format_label_user_prompt(
                    options.format_label_instruction,
                    options.chart_config,
                    options.sql_statement,
                    options.data_metadata,
                ),
            },
        ],
        prompt_name: "format_labels".to_string(),
        json_mode: true,
        ..Default::default()
    };

    let format_result = match prompt_node(format_label_settings).await {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(format_result)
}
