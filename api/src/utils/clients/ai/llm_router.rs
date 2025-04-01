use std::env;

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use uuid::Uuid;

use super::{
    anthropic::{
        anthropic_chat, anthropic_chat_stream, AnthropicChatMessage, AnthropicChatModel,
        AnthropicChatRole, AnthropicContent, AnthropicContentType,
    },
    langfuse::{send_langfuse_request, PromptName},
    openai::{
        openai_chat, openai_chat_stream, OpenAiChatContent, OpenAiChatMessage, OpenAiChatModel,
        OpenAiChatRole,
    },
};
use lazy_static::lazy_static;

lazy_static! {
    static ref MONITORING_ENABLED: bool = env::var("MONITORING_ENABLED")
        .unwrap_or(String::from("true"))
        .parse()
        .expect("MONITORING_ENABLED must be a boolean");
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
pub enum LlmModel {
    Anthropic(AnthropicChatModel),
    OpenAi(OpenAiChatModel),
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LlmRole {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LlmMessage {
    pub role: LlmRole,
    pub content: String,
}

impl LlmMessage {
    pub fn new(role: String, content: String) -> Self {
        let role = match role.to_lowercase().as_str() {
            "system" => LlmRole::System,
            "user" => LlmRole::User,
            "assistant" => LlmRole::Assistant,
            _ => LlmRole::User,
        };
        Self { role, content }
    }
}

pub async fn llm_chat(
    model: LlmModel,
    messages: &Vec<LlmMessage>,
    temperature: f32,
    max_tokens: u32,
    timeout: u64,
    stop: Option<Vec<String>>,
    json_mode: bool,
    json_schema: Option<Value>,
    session_id: &Uuid,
    user_id: &Uuid,
    prompt_name: PromptName,
) -> Result<String> {
    let start_time = Utc::now();

    let response_result = match &model {
        LlmModel::Anthropic(model) => {
            anthropic_chat_compiler(model, messages, max_tokens, temperature, timeout, stop).await
        }
        LlmModel::OpenAi(model) => {
            openai_chat_compiler(
                model,
                messages,
                7048,
                temperature,
                timeout,
                stop,
                json_mode,
                json_schema,
            )
            .await
        }
    };

    let response = match response_result {
        Ok(response) => response,
        Err(e) => return Err(anyhow!("LLM chat error: {}", e)),
    };

    let end_time = Utc::now();

    send_langfuse_request(
        session_id,
        prompt_name,
        None,
        start_time,
        end_time,
        serde_json::to_string(&messages).unwrap(),
        serde_json::to_string(&response).unwrap(),
        user_id,
        &model,
    )
    .await;

    Ok(response)
}

pub async fn llm_chat_stream(
    model: LlmModel,
    messages: Vec<LlmMessage>,
    temperature: f32,
    max_tokens: u32,
    timeout: u64,
    stop: Option<Vec<String>>,
    session_id: &Uuid,
    user_id: &Uuid,
    prompt_name: PromptName,
) -> Result<(Receiver<String>, JoinHandle<Result<String>>)> {
    let start_time = Utc::now();

    let stream_result = match &model {
        LlmModel::Anthropic(model) => {
            anthropic_chat_stream_compiler(model, &messages, max_tokens, temperature, timeout, stop)
                .await
        }
        LlmModel::OpenAi(model) => {
            openai_chat_stream_compiler(model, &messages, max_tokens, temperature, timeout, stop)
                .await
        }
    };

    let mut stream = match stream_result {
        Ok(stream) => stream,
        Err(e) => return Err(anyhow!("LLM chat error: {}", e)),
    };

    let (tx, rx) = mpsc::channel(100);

    let res_future = {
        let session_id = session_id.clone();
        let user_id = user_id.clone();

        tokio::spawn(async move {
            let mut response = String::new();
            while let Some(content) = stream.next().await {
                response.push_str(&content);

                match tx.send(content).await {
                    Ok(_) => (),
                    Err(e) => return Err(anyhow!("Streaming Error: {}", e)),
                }
            }

            let end_time = Utc::now();

            send_langfuse_request(
                &session_id,
                prompt_name,
                None,
                start_time,
                end_time,
                serde_json::to_string(&messages).unwrap(),
                serde_json::to_string(&response).unwrap(),
                &user_id,
                &model,
            )
            .await;

            Ok(response)
        })
    };

    Ok((rx, res_future))
}

async fn anthropic_chat_compiler(
    model: &AnthropicChatModel,
    messages: &Vec<LlmMessage>,
    max_tokens: u32,
    temperature: f32,
    timeout: u64,
    stop: Option<Vec<String>>,
) -> Result<String> {
    let system_message = match messages.iter().find(|m| m.role == LlmRole::System) {
        Some(message) => Some(message.content.clone()),
        None => None,
    };
    let mut anthropic_messages = Vec::new();

    for message in messages {
        let anthropic_role = match message.role {
            LlmRole::System => continue,
            LlmRole::User => AnthropicChatRole::User,
            LlmRole::Assistant => AnthropicChatRole::Assistant,
        };

        let anthropic_content = AnthropicContent {
            text: message.content.clone(),
            _type: AnthropicContentType::Text,
        };

        anthropic_messages.push(AnthropicChatMessage {
            role: anthropic_role,
            content: vec![anthropic_content],
        });
    }

    let response = match anthropic_chat(
        model,
        system_message,
        &anthropic_messages,
        temperature,
        7048,
        timeout,
        stop,
    )
    .await
    {
        Ok(response) => response,
        Err(e) => return Err(anyhow!("Anthropic chat error: {}", e)),
    };

    Ok(response)
}

async fn openai_chat_compiler(
    model: &OpenAiChatModel,
    messages: &Vec<LlmMessage>,
    max_tokens: u32,
    temperature: f32,
    timeout: u64,
    stop: Option<Vec<String>>,
    json_mode: bool,
    json_schema: Option<Value>,
) -> Result<String> {
    let mut openai_messages = Vec::new();

    for message in messages {
        let openai_role = match message.role {
            LlmRole::System => OpenAiChatRole::System,
            LlmRole::User => OpenAiChatRole::User,
            LlmRole::Assistant => OpenAiChatRole::Assistant,
        };

        let openai_message = OpenAiChatMessage {
            role: openai_role,
            content: vec![OpenAiChatContent {
                type_: "text".to_string(),
                text: message.content.clone(),
            }],
        };

        openai_messages.push(openai_message);
    }

    let response = match openai_chat(
        &model,
        openai_messages,
        temperature,
        max_tokens,
        timeout,
        stop,
        json_mode,
        json_schema,
    )
    .await
    {
        Ok(response) => response,
        Err(e) => return Err(anyhow!("Anthropic chat error: {}", e)),
    };

    Ok(response)
}

async fn anthropic_chat_stream_compiler(
    model: &AnthropicChatModel,
    messages: &Vec<LlmMessage>,
    max_tokens: u32,
    temperature: f32,
    timeout: u64,
    stop: Option<Vec<String>>,
) -> Result<ReceiverStream<String>> {
    let system_message = match messages.iter().find(|m| m.role == LlmRole::System) {
        Some(message) => Some(message.content.clone()),
        None => None,
    };
    let mut anthropic_messages = Vec::new();

    for message in messages {
        let anthropic_role = match message.role {
            LlmRole::System => continue,
            LlmRole::User => AnthropicChatRole::User,
            LlmRole::Assistant => AnthropicChatRole::Assistant,
        };

        let anthropic_content = AnthropicContent {
            text: message.content.clone(),
            _type: AnthropicContentType::Text,
        };

        anthropic_messages.push(AnthropicChatMessage {
            role: anthropic_role,
            content: vec![anthropic_content],
        });
    }

    let stream = match anthropic_chat_stream(
        model,
        system_message,
        &anthropic_messages,
        temperature,
        max_tokens,
        timeout,
        stop,
    )
    .await
    {
        Ok(response) => response,
        Err(e) => return Err(anyhow!("Anthropic chat error: {}", e)),
    };

    Ok(stream)
}

async fn openai_chat_stream_compiler(
    model: &OpenAiChatModel,
    messages: &Vec<LlmMessage>,
    max_tokens: u32,
    temperature: f32,
    timeout: u64,
    stop: Option<Vec<String>>,
) -> Result<ReceiverStream<String>> {
    let mut openai_messages = Vec::new();

    for message in messages {
        let openai_role = match message.role {
            LlmRole::System => OpenAiChatRole::System,
            LlmRole::User => OpenAiChatRole::User,
            LlmRole::Assistant => OpenAiChatRole::Assistant,
        };

        let openai_message = OpenAiChatMessage {
            role: openai_role,
            content: vec![OpenAiChatContent {
                type_: "text".to_string(),
                text: message.content.clone(),
            }],
        };

        openai_messages.push(openai_message);
    }

    let stream = match openai_chat_stream(
        &model,
        &openai_messages,
        temperature,
        max_tokens,
        timeout,
        stop,
    )
    .await
    {
        Ok(response) => response,
        Err(e) => return Err(anyhow!("Anthropic chat error: {}", e)),
    };

    Ok(stream)
}
