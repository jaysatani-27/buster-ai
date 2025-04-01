mod post_data_sources;

use axum::{routing::post, Router};

pub fn router() -> Router {
    Router::new()
        .route("/", post(post_data_sources::post_data_sources))
}
