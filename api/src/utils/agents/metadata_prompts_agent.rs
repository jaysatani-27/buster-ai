use serde::Deserialize;
use serde_json::Value;
use std::fmt;
use tokio::sync::mpsc;

use crate::utils::{
    agent_builder::nodes::{
        error_node::ErrorNode,
        prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
    },
    prompts::modify_visualization_prompts::title_description_time_frame_prompts::{
        title_description_time_frame_system_prompt, title_description_time_frame_user_prompt,
    },
};

pub enum MetadataPromptsAgentError {
    MissingKey,
    ObjectNotJson,
    PromptNodeError,
}

impl fmt::Display for MetadataPromptsAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey => write!(f, "missing_key"),
            Self::ObjectNotJson => write!(f, "object_not_json"),
            Self::PromptNodeError => write!(f, "prompt_node_error"),
        }
    }
}

pub struct MetadataPromptsAgentOptions {
    pub user_message: String,
    pub sql: String,
    pub thoughts: String,
    pub output_sender: mpsc::Sender<Value>,
}

#[derive(Deserialize)]
pub struct MetricTitle {
    pub classification: String,
    pub output: String,
}

#[derive(Deserialize)]
pub struct MetricSummaryQuestion {
    pub output: String,
}

#[derive(Deserialize)]
pub struct TimeFrame {
    #[serde(rename = "type")]
    pub type_: String,
    pub output: String,
}

#[derive(Deserialize)]
pub struct MetadataPromptResult {
    pub metric_title: MetricTitle,
    pub metric_summary_question: MetricSummaryQuestion,
    pub time_frame: TimeFrame,
}

pub struct MetadataPromptsAgentResults {
    pub name: String,
    pub title: String,
    pub time_frame: String,
    pub description: String,
}

pub async fn metadata_prompts_agent(
    options: MetadataPromptsAgentOptions,
) -> Result<MetadataPromptsAgentResults, ErrorNode> {
    let metadata_prompt_settings = PromptNodeSettings {
        messages: vec![
            PromptNodeMessage {
                role: "system".to_string(),
                content: title_description_time_frame_system_prompt(),
            },
            PromptNodeMessage {
                role: "user".to_string(),
                content: title_description_time_frame_user_prompt(
                    &options.user_message,
                    &options.sql,
                    &options.thoughts,
                ),
            },
        ],
        prompt_name: "title_description_time_frame_prompt".to_string(),
        json_mode: true,
        ..Default::default()
    };

    let metadata_prompt_result = prompt_node(metadata_prompt_settings).await?;

    let metadata_prompt_result: MetadataPromptResult =
        serde_json::from_value(metadata_prompt_result).map_err(|_| {
            ErrorNode::new(
                MetadataPromptsAgentError::ObjectNotJson.to_string(),
                "Metadata prompts agent not JSON".to_string(),
            )
        })?;

    Ok(MetadataPromptsAgentResults {
        name: "metadata_prompts_agent".to_string(),
        title: metadata_prompt_result.metric_title.output,
        time_frame: metadata_prompt_result.time_frame.output,
        description: metadata_prompt_result.metric_summary_question.output,
    })
}
