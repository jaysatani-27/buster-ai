use axum::http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, Method};
use tower_http::cors::{Any, CorsLayer};

pub fn cors() -> CorsLayer {
    let cors = CorsLayer::new()
    .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_origin(Any)
    .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    cors
}

