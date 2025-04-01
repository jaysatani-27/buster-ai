use anyhow::{anyhow, Result};
use bb8_redis::{bb8, RedisConnectionManager};
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::bb8::Pool as DieselPool;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use futures::future::BoxFuture;
use futures::FutureExt;
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::{PgPool as SqlxPool, PgPoolOptions};
use std::env;
use std::time::Duration;
use uuid::Uuid;

use crate::utils::query_engine::data_types::DataType;

pub type PgPool = DieselPool<AsyncPgConnection>;
pub type PgPoolSqlx = SqlxPool;
pub type RedisPool = bb8::Pool<RedisConnectionManager>;

static DIESEL_POOL: OnceCell<PgPool> = OnceCell::new();
static REDIS_POOL: OnceCell<RedisPool> = OnceCell::new();
static SQLX_POOL: OnceCell<SqlxPool> = OnceCell::new();

pub async fn init_pools() -> Result<()> {
    let diesel_pool = match establish_diesel_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(anyhow!("Failed to establish diesel connection: {}", e)),
    };

    let redis_pool = match create_redis_pool().await {
        Ok(pool) => pool,
        Err(e) => return Err(anyhow!("Failed to create redis pool: {}", e)),
    };

    let sqlx_pool = match establish_sqlx_connection().await {
        Ok(pool) => pool,
        Err(e) => return Err(anyhow!("Failed to establish sqlx connection: {}", e)),
    };

    DIESEL_POOL
        .set(diesel_pool)
        .map_err(|_| anyhow!("DieselPool already initialized"))?;
    REDIS_POOL
        .set(redis_pool)
        .map_err(|_| anyhow!("RedisPool already initialized"))?;
    SQLX_POOL
        .set(sqlx_pool)
        .map_err(|_| anyhow!("SqlxPool already initialized"))?;

    Ok(())
}

pub fn get_pg_pool() -> &'static PgPool {
    DIESEL_POOL.get().expect("DieselPool not initialized")
}

pub fn get_redis_pool() -> &'static RedisPool {
    REDIS_POOL.get().expect("RedisPool not initialized")
}

pub fn get_sqlx_pool() -> &'static SqlxPool {
    SQLX_POOL.get().expect("SqlxPool not initialized")
}

pub async fn establish_diesel_connection() -> Result<PgPool> {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or("postgresql://postgres:postgres@127.0.0.1:54322/postgres".to_string());
    let max_pool_size: usize = env::var("DATABASE_POOL_SIZE")
        .unwrap_or("30".to_string())
        .parse()
        .expect("DATABASE_POOL_SIZE must be a valid usize");

    if db_url.contains("sslmode=verify-full") {
        let db_url = db_url.replace("sslmode=verify-full", "");
        let mut config = ManagerConfig::default();
        config.custom_setup = Box::new(establish_secure_connection);

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            db_url.clone(),
            config,
        );
        PgPool::builder()
            .max_size(max_pool_size as u32)
            .min_idle(Some(5))
            .max_lifetime(Some(Duration::from_secs(60 * 60 * 24)))
            .idle_timeout(Some(Duration::from_secs(60 * 2)))
            .test_on_check_out(true)
            .build(manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to establish diesel connection: {}", e);
                anyhow!("Failed to establish diesel connection: {}", e)
            })
    } else {
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
        PgPool::builder()
            .max_size(max_pool_size as u32)
            .min_idle(Some(5))
            .max_lifetime(Some(Duration::from_secs(60 * 60 * 24)))
            .idle_timeout(Some(Duration::from_secs(60 * 2)))
            .test_on_check_out(true)
            .build(manager)
            .await
            .map_err(|e| {
                tracing::error!("Failed to establish diesel connection: {}", e);
                anyhow!("Failed to establish diesel connection: {}", e)
            })
    }
}

pub async fn establish_sqlx_connection() -> Result<SqlxPool> {
    let db_url = env::var("POOLER_URL")
        .unwrap_or("postgresql://postgres:postgres@127.0.0.1:54322/postgres".to_string());
    let max_pool_size: u32 = env::var("SQLX_POOL_SIZE")
        .unwrap_or("30".to_string())
        .parse()
        .expect("SQLX_POOL_SIZE must be a valid u32");

    PgPoolOptions::new()
        .max_connections(max_pool_size)
        .min_connections(5)
        .max_lifetime(Duration::from_secs(60 * 60 * 24))
        .idle_timeout(Duration::from_secs(60 * 2))
        .test_before_acquire(true)
        .connect(&db_url)
        .await
        .map_err(|e| anyhow!("Failed to establish sqlx connection: {}", e))
}

fn establish_secure_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        let rustls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_certs())
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;

        AsyncPgConnection::try_from_client_and_connection(client, conn).await
    };
    fut.boxed()
}

fn root_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs().expect("Certs not loadable!");
    roots.add_parsable_certificates(certs);
    roots
}

pub async fn create_redis_pool() -> Result<RedisPool> {
    let redis_url = env::var("REDIS_URL").unwrap_or("redis://localhost:6379".to_string());

    let manager = match RedisConnectionManager::new(redis_url) {
        Ok(manager) => manager,
        Err(e) => {
            tracing::error!("Failed to create redis pool: {}", e);
            return Err(anyhow!("Failed to create redis pool: {}", e));
        }
    };

    let pool = match bb8::Pool::builder().max_size(10000).build(manager).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to create redis pool: {}", e);
            return Err(anyhow!("Failed to create redis pool: {}", e));
        }
    };

    Ok(pool)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum StepProgress {
    InProgress,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentifiedDataset {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentifyingDataset {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    pub dataset: Option<IdentifiedDataset>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentifiedTerm {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentifyingTerms {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    pub terms: Option<Vec<IdentifiedTerm>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneratingSql {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    pub sql_chunk: Option<String>,
    pub sql: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FixingSql {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    pub sql_chunk: Option<String>,
    pub sql: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchingData {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    pub data: Option<Vec<IndexMap<String, DataType>>>,
    pub chart_config: Option<Value>,
    pub code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneratingResponse {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    pub text_chunk: Option<String>,
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneratingMetricTitle {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    pub metric_title_chunk: Option<String>,
    pub metric_title: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneratingSummaryQuestion {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    #[serde(rename = "description_chunk")]
    pub summary_question_chunk: Option<String>,
    #[serde(rename = "description")]
    pub summary_question: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneratingTimeFrame {
    pub thread_id: Uuid,
    pub message_id: Uuid,
    pub progress: StepProgress,
    pub time_frame_chunk: Option<String>,
    pub time_frame: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Step {
    IdentifyingDataset(IdentifyingDataset),
    IdentifyingTerms(IdentifyingTerms),
    GeneratingSql(GeneratingSql),
    FixingSql(FixingSql),
    FetchingData(FetchingData),
    GeneratingResponse(GeneratingResponse),
    GeneratingMetricTitle(GeneratingMetricTitle),
    GeneratingSummaryQuestion(GeneratingSummaryQuestion),
    GeneratingTimeFrame(GeneratingTimeFrame),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContextJsonBody {
    pub steps: Vec<Step>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MinMaxValue {
    Number(f64),
    String(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColumnMetadata {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub simple_type: Option<String>,
    pub unique_values: i32,
    pub min_value: Option<MinMaxValue>,
    pub max_value: Option<MinMaxValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataMetadataJsonBody {
    pub column_count: i32,
    pub row_count: i32,
    pub column_metadata: Vec<ColumnMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageResponses {
    pub messages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserConfig {
    pub color_palettes: Option<Vec<Vec<String>>>,
    pub last_used_color_palette: Option<Vec<String>>,
}
