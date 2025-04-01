use std::env;

use anyhow::{anyhow, Result};
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct OllamaEmbeddingRequest {
    pub prompt: String,
    pub model: String,
}

#[derive(Deserialize, Debug)]
pub struct OllamaEmbeddingResponse {
    pub embedding: Vec<f32>,
}

pub async fn ollama_embedding(prompt: String, for_retrieval: bool) -> Result<Vec<f32>> {
    let ollama_url = env::var("OLLAMA_URL").unwrap_or(String::from("http://localhost:11434"));
    let model = env::var("EMBEDDING_MODEL").unwrap_or(String::from("mxbai-embed-large"));

    let prompt = if model == "mxbai-embed-large" && for_retrieval {
        format!(
            "Represent this sentence for searching relevant passages:: {}",
            prompt
        )
    } else {
        prompt
    };

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

    let req = OllamaEmbeddingRequest { prompt, model };

    let res = match client
        .post(format!("{}/api/embeddings", ollama_url))
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

    let ollama_res: OllamaEmbeddingResponse = match res.json::<OllamaEmbeddingResponse>().await {
        Ok(res) => res,
        Err(e) => {
            return Err(anyhow!("Error parsing Ollama response: {:?}", e));
        }
    };

    Ok(ollama_res.embedding)
}
