use anyhow::Result;
use axum::http::StatusCode;
use axum::Extension;
use chrono::Utc;
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

use crate::database::lib::get_pg_pool;
use crate::database::models::{ApiKey, User};
use crate::database::schema::api_keys;
use crate::routes::rest::ApiResponse;
use crate::utils::user::user_info::get_user_organization_id;

#[derive(Debug, Serialize)]
pub struct PostApiKeyResponse {
    pub api_key: String,
}

// Add this struct for JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct ApiKeyClaims {
    exp: i64,
    aud: String,
    sub: String,
}

pub async fn post_api_key(
    Extension(user): Extension<User>,
) -> Result<ApiResponse<PostApiKeyResponse>, (StatusCode, &'static str)> {
    let api_key = match post_api_key_handler(user).await {
        Ok(api_key) => api_key,
        Err(e) => {
            tracing::error!("Error creating API key: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Error creating API key"));
        }
    };

    Ok(ApiResponse::JsonData(PostApiKeyResponse { api_key }))
}

async fn post_api_key_handler(user: User) -> Result<String> {
    let jwt_secret = env::var("JWT_SECRET").map_err(|_| anyhow::anyhow!("JWT_SECRET not set"))?;

    // Create JWT claims
    let claims = ApiKeyClaims {
        exp: (Utc::now() + chrono::Duration::days(365 * 5)).timestamp(),
        aud: "api".to_string(),
        sub: user.id.to_string(),
    };

    // Generate JWT token
    let api_key = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| anyhow::anyhow!("Failed to create JWT: {}", e))?;

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting database connection: {:?}", e);
            return Err(anyhow::anyhow!("Error getting database connection"));
        }
    };

    let organization_id = match get_user_organization_id(&user.id).await {
        Ok(organization_id) => organization_id,
        Err(e) => {
            tracing::error!("Error getting organization ID: {:?}", e);
            return Err(anyhow::anyhow!("Error getting organization ID"));
        }
    };

    let api_key_record = ApiKey {
        id: Uuid::new_v4(),
        owner_id: user.id,
        key: api_key.clone(),
        organization_id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    match insert_into(api_keys::table)
        .values(api_key_record)
        .execute(&mut *conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error inserting API key: {:?}", e);
            return Err(anyhow::anyhow!("Error inserting API key"));
        }
    };

    Ok(api_key)
}
