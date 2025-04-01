mod embeddings;

use axum::{routing::post, Router};

use self::embeddings::create_embedding::create_record_embedding;

pub fn router() -> Router {
    Router::new().route("/embeddings", post(create_record_embedding))
}
