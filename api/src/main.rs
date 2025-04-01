mod buster_middleware;
pub mod database;
mod routes;
mod types;
pub mod utils;

use std::env;
use std::sync::Arc;

use axum::{middleware, Extension, Router};
use buster_middleware::{auth::auth, cors::cors};
use database::lib::get_pg_pool;
use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use rustls::crypto::ring;
use tokio::sync::broadcast;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[tokio::main]
#[allow(unused)]
async fn main() {
    dotenv().ok();

    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    ring::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    let _guard = sentry::init(("https://a417fbed1de30d2714a8afbe38d5bc1b@o4505360096428032.ingest.us.sentry.io/4507360721043456", sentry::ClientOptions {
        release: sentry::release_name!(),
        environment: Some(environment.into()),
        ..Default::default()
    }));

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(tracing::Level::DEBUG.to_string())),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize global pools
    if let Err(e) = database::lib::init_pools().await {
        tracing::error!("Failed to initialize global pools: {}", e);
        return;
    }

    tracing::info!("Running database migrations");

    if let Err(e) = run_migrations().await {
        tracing::error!("Failed to run database migrations: {}", e);
        return;
    }

    tracing::info!("Successfully ran database migrations");

    let protected_router = Router::new().nest("/api/v1", routes::protected_router());
    let public_router = Router::new().route("/health", axum::routing::get(|| async { "OK" }));

    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let shutdown_tx = Arc::new(shutdown_tx);

    let app = Router::new()
        .merge(protected_router)
        .merge(public_router)
        .layer(TraceLayer::new_for_http())
        .layer(cors())
        .layer(CompressionLayer::new())
        .layer(Extension(shutdown_tx.clone()));

    let port_number: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string())
        .parse()
        .unwrap();

    let listener = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port_number)).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to port {}: {}", port_number, e);
            return;
        }
    };

    let server = axum::serve(listener, app);

    tokio::select! {
        _ = server => {},
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutdown signal received, starting graceful shutdown");
            shutdown_tx.send(()).unwrap_or_default();
        }
    }
}

async fn run_migrations() -> Result<(), anyhow::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|e| anyhow::anyhow!("Failed to get DATABASE_URL: {}", e))?;

    let mut connection = PgConnection::establish(&database_url)
        .map_err(|e| anyhow::anyhow!("Failed to establish database connection: {}", e))?;

    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

    Ok(())
}
