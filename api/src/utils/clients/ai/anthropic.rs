use anyhow::{anyhow, Result};
use futures::StreamExt;
use std::{env, time::Duration};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;

use serde::{Deserialize, Serialize};

use crate::utils::clients::sentry_utils::send_sentry_error;

const ANTHROPIC_CHAT_URL: &str = "https://api.anthropic.com/v1/messages";

lazy_static::lazy_static! {
    static ref ANTHROPIC_API_KEY: String = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set");
    static ref MONITORING_ENABLED: bool = env::var("MONITORING_ENABLED")
        .unwrap_or(String::from("true"))
        .parse()
        .expect("MONITORING_ENABLED must be a boolean");
}

#[derive(Serialize, Clone)]
pub enum AnthropicChatModel {
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus20240229,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AnthropicChatRole {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AnthropicContentType {
    Text,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnthropicContent {
    #[serde(rename = "type")]
    pub _type: AnthropicContentType,
    pub text: String,
}

#[derive(Serialize, Clone)]
pub struct AnthropicChatMessage {
    pub role: AnthropicChatRole,
    pub content: Vec<AnthropicContent>,
}

#[derive(Serialize, Clone)]
pub struct AnthropicChatRequest {
    pub model: AnthropicChatModel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<AnthropicChatMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChatCompletionResponse {
    pub content: Vec<Content>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Content {
    #[serde(rename = "type")]
    pub _type: String,
    pub text: String,
}

pub async fn anthropic_chat(
    model: &AnthropicChatModel,
    system: Option<String>,
    messages: &Vec<AnthropicChatMessage>,
    temperature: f32,
    max_tokens: u32,
    timeout: u64,
    stop: Option<Vec<String>>,
) -> Result<String> {
    let chat_request = AnthropicChatRequest {
        model: model.clone(),
        system,
        messages: messages.clone(),
        temperature,
        max_tokens,
        stop_sequences: stop,
        stream: false,
    };

    let client = reqwest::Client::new();

    let headers = {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            format!("{}", ANTHROPIC_API_KEY.to_string())
                .parse()
                .unwrap(),
        );
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
        headers
    };

    let response = match client
        .post(ANTHROPIC_CHAT_URL)
        .headers(headers)
        .json(&chat_request)
        .timeout(Duration::from_secs(timeout))
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Unable to send request to Anthropic: {:?}", e);
            let err = anyhow!("Unable to send request to Anthropic: {}", e);
            send_sentry_error(&err.to_string(), None);
            return Err(err);
        }
    };

    let completion_res = match response.json::<ChatCompletionResponse>().await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Unable to parse response from Anthropic: {:?}", e);
            let err = anyhow!("Unable to parse response from Anthropic: {}", e);
            send_sentry_error(&err.to_string(), None);
            return Err(err);
        }
    };

    let content = match completion_res.content.get(0) {
        Some(content) => content.text.clone(),
        None => return Err(anyhow!("No content returned from Anthropic")),
    };

    Ok(content)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicChatDelta {
    #[serde(rename = "type")]
    pub _type: String,
    pub delta: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnthropicChatStreamResponse {
    #[serde(rename = "type")]
    pub _type: String,
    pub delta: Option<AnthropicChatDelta>,
}

pub async fn anthropic_chat_stream(
    model: &AnthropicChatModel,
    system: Option<String>,
    messages: &Vec<AnthropicChatMessage>,
    temperature: f32,
    max_tokens: u32,
    timeout: u64,
    stop: Option<Vec<String>>,
) -> Result<ReceiverStream<String>> {
    let chat_request = AnthropicChatRequest {
        model: model.clone(),
        system,
        messages: messages.clone(),
        temperature,
        max_tokens,
        stream: true,
        stop_sequences: stop,
    };

    let client = reqwest::Client::new();

    let headers = {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", ANTHROPIC_API_KEY.to_string())
                .parse()
                .unwrap(),
        );
        headers
    };

    let (_tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel(100);

    tokio::spawn(async move {
        let response = client
            .post(ANTHROPIC_CHAT_URL)
            .headers(headers)
            .json(&chat_request)
            .timeout(Duration::from_secs(timeout))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Unable to send request to Anthropic: {:?}", e);
                let err = anyhow!("Unable to send request to Anthropic: {}", e);
                send_sentry_error(&err.to_string(), None);
                err
            });

        if let Err(_e) = response {
            return;
        }

        let response = response.unwrap();
        let mut stream = response.bytes_stream();

        let _buffer = String::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let chunk = String::from_utf8(bytes.to_vec()).unwrap();
                    println!("----------------------");
                    println!("Chunk: {}", chunk);
                    println!("----------------------");
                }
                Err(e) => {
                    tracing::error!("Error while streaming response: {:?}", e);
                    let err = anyhow!("Error while streaming response: {}", e);
                    send_sentry_error(&err.to_string(), None);
                    break;
                }
            }
        }
    });

    Ok(ReceiverStream::new(rx))
}
