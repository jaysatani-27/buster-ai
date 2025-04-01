mod collections;
mod dashboards;
mod data_sources;
mod datasets;
mod organizations;
mod permissions;
mod search;
mod sql;
mod teams;
mod terms;
pub mod threads_and_messages;
mod users;
pub mod ws;
pub mod ws_router;
pub mod ws_utils;

use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/", get(ws::ws))
}
