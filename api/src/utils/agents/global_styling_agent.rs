use serde_json::Value;

use crate::utils::{
    agent_builder::nodes::{
        error_node::ErrorNode,
        prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
    },
    prompts::modify_visualization_prompts::styling_prompts::global_styling_prompts::{
        global_styling_system_prompt, global_styling_user_prompt,
    },
};

#[derive(Clone)]
pub struct GlobalStylingAgentOptions {
    pub global_styling_instruction: String,
    pub chart_config: String,
    pub sql_statement: String,
    pub data_metadata: String,
}

pub async fn global_styling_agent(options: GlobalStylingAgentOptions) -> Result<Value, ErrorNode> {
    let global_styling_settings = PromptNodeSettings {
        messages: vec![
            PromptNodeMessage {
                role: "system".to_string(),
                content: global_styling_system_prompt(),
            },
            PromptNodeMessage {
                role: "user".to_string(),
                content: global_styling_user_prompt(
                    options.global_styling_instruction,
                    options.chart_config,
                    options.sql_statement,
                    options.data_metadata,
                ),
            },
        ],
        prompt_name: "global_styling".to_string(),
        json_mode: true,
        ..Default::default()
    };

    let global_styling_result = match prompt_node(global_styling_settings).await {
        Ok(value) => value,
        Err(e) => return Err(e),
    };

    Ok(global_styling_result)
}
