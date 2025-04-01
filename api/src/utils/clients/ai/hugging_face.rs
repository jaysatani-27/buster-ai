use anyhow::{anyhow, Result};
use serde::Serialize;
use std::env;

use axum::http::HeaderMap;

#[derive(Serialize)]
pub struct HuggingFaceEmbeddingRequest {
    pub inputs: Vec<String>,
}

pub async fn hugging_face_embedding(prompts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    let hugging_face_url = env::var("HUGGING_FACE_URL").expect("HUGGING_FACE_URL must be set");
    let hugging_face_api_key =
        env::var("HUGGING_FACE_API_KEY").expect("HUGGING_FACE_API_KEY must be set");

    let client = match reqwest::Client::builder().build() {
        Ok(client) => client,
        Err(e) => {
            return Err(anyhow!("Error creating reqwest client: {:?}", e));
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", hugging_face_api_key).parse().unwrap(),
    );

    let req = HuggingFaceEmbeddingRequest { inputs: prompts };

    let res = match client
        .post(hugging_face_url)
        .headers(headers)
        .json(&req)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return Err(anyhow!("Error sending Ollama request: {:?}", e));
        }
    };

    let embeddings = match res.json::<Vec<Vec<f32>>>().await {
        Ok(res) => res,
        Err(e) => {
            return Err(anyhow!("Error parsing Ollama response: {:?}", e));
        }
    };

    Ok(embeddings)
}
