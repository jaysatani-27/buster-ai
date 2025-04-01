mod rest;
pub mod ws;

use axum::{middleware, routing::get, Router};

use crate::buster_middleware::auth::auth;

pub fn protected_router() -> Router {
    Router::new()
        .route("/health", get(health_check_handler))
        .nest("/", rest::router())
        .merge(
            Router::new()
                .nest("/ws", ws::router())
                .route_layer(middleware::from_fn(auth)),
        )
}

async fn health_check_handler() -> String {
    "API is healthy".to_string()
}
