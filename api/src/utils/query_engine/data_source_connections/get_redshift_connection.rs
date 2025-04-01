use std::time::Duration;

use anyhow::{anyhow, Result};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Pool, Postgres,
};

use crate::utils::query_engine::credentials::PostgresCredentials;

pub async fn get_redshift_connection(credentials: &PostgresCredentials) -> Result<Pool<Postgres>> {
    let options = PgConnectOptions::new()
        .host(credentials.host.as_str())
        .port(credentials.port)
        .username(credentials.username.as_str())
        .password(credentials.password.as_str())
        .database(credentials.database.clone().unwrap_or_default().as_str())
        .extra_float_digits(2);

    let redshift_pool = match PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(options)
        .await
    {
        Ok(redshift_pool) => redshift_pool,
        Err(e) => {
            tracing::error!("There was an issue while connecting to Redshift: {}", e);
            return Err(anyhow!(e));
        }
    };

    Ok(redshift_pool)
}
