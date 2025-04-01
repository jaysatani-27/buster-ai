use std::env;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::task;

use super::{
    hugging_face::hugging_face_embedding, ollama::ollama_embedding, openai::ada_bulk_embedding,
};

#[derive(Serialize)]
pub struct EmbeddingRequest {
    pub prompt: String,
}

#[derive(Deserialize, Debug)]
pub struct EmbeddingResponse {
    pub embedding: Vec<Vec<f32>>,
}

pub enum EmbeddingProvider {
    OpenAi,
    Ollama,
    HuggingFace,
}

impl EmbeddingProvider {
    pub fn get_embedding_provider() -> Result<EmbeddingProvider> {
        let embedding_provider =
            env::var("EMBEDDING_PROVIDER").expect("An embedding provider is required.");
        match embedding_provider.as_str() {
            "openai" => Ok(EmbeddingProvider::OpenAi),
            "ollama" => Ok(EmbeddingProvider::Ollama),
            "huggingface" => Ok(EmbeddingProvider::HuggingFace),
            _ => Err(anyhow!("Invalid embedding provider")),
        }
    }
}

pub async fn embedding_router(prompts: Vec<String>, for_retrieval: bool) -> Result<Vec<Vec<f32>>> {
    let embedding_provider = EmbeddingProvider::get_embedding_provider()?;

    match embedding_provider {
        EmbeddingProvider::Ollama => {
            let tasks = prompts.into_iter().map(|prompt| {
                task::spawn(async move { ollama_embedding(prompt, for_retrieval).await })
            });
            let results = futures::future::join_all(tasks).await;
            let embeddings: Result<Vec<Vec<f32>>> = results
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect();
            embeddings
        }
        EmbeddingProvider::HuggingFace => hugging_face_embedding(prompts).await,
        EmbeddingProvider::OpenAi => ada_bulk_embedding(prompts).await,
    }
}
