use serde_json::Value;

use crate::utils::{
    agent_builder::nodes::{
        error_node::ErrorNode,
        prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
    },
    prompts::modify_visualization_prompts::styling_prompts::column_styling_prompts::{
        column_styling_system_prompt, column_styling_user_prompt,
    },
};

#[derive(Clone)]
pub struct ColumnStylingAgentOptions {
    pub column_styling_instruction: String,
    pub chart_config: String,
    pub sql_statement: String,
    pub data_metadata: String,
}

pub struct FormatLabelsAgentResult {
    pub format_result: Value,
}

pub async fn column_styling_agent(options: ColumnStylingAgentOptions) -> Result<Value, ErrorNode> {
    let column_styling_settings = PromptNodeSettings {
        messages: vec![
            PromptNodeMessage {
                role: "system".to_string(),
                content: column_styling_system_prompt(),
            },
            PromptNodeMessage {
                role: "user".to_string(),
                content: column_styling_user_prompt(
                    options.column_styling_instruction,
                    options.chart_config,
                    options.sql_statement,
                    options.data_metadata,
                ),
            },
        ],
        prompt_name: "column_styling".to_string(),
        json_mode: true,
        ..Default::default()
    };

    let column_styling_result = match prompt_node(column_styling_settings).await {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(column_styling_result)
}
