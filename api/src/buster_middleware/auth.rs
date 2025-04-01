use anyhow::{anyhow, Result};
use std::{collections::HashMap, env};

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{lib::get_pg_pool, models::User};

/// Authentication is done via Bearer token with a JWT issued from Supabase.  We also offer API access that
/// is done via a Bearer token with a JWT issued from us.
///
/// The user ID is always included as the `sub` in the JWT.
///
/// In the JWT that we issue, we provide an extra field of `api_key_id` that contains the ID of the API key that is being used.
/// The reason why we have the api_key_id is because a user could have multiple API keys and we want to be able to route to the correct key and track it.

#[derive(Serialize, Deserialize, Debug)]
struct JwtClaims {
    pub aud: String,
    pub sub: String,
    pub exp: u64,
}

pub async fn auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let is_ws = req
        .headers()
        .get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false);

    let handle_auth_error = |msg: &str| {
        if is_ws {
            Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("Sec-WebSocket-Protocol", "close")
                .header("Sec-WebSocket-Close-Code", "4001") // Custom close code
                .header("Sec-WebSocket-Close-Reason", msg)
                .body(axum::body::Body::empty())
                .unwrap())
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    };

    let buster_wh_token = env::var("BUSTER_WH_TOKEN").expect("BUSTER_WH_TOKEN is not set");

    let bearer_token = req.headers().get("Authorization").and_then(|value| {
        value.to_str().ok().and_then(|v| {
            if v.starts_with("Bearer ") {
                v.split_whitespace().nth(1)
            } else {
                Some(v)
            }
        })
    });

    if let Some(token) = bearer_token {
        if token == buster_wh_token {
            return Ok(next.run(req).await);
        }
    }

    let token = if bearer_token.is_none() {
        match req
            .uri()
            .query()
            .and_then(|query| serde_urlencoded::from_str::<HashMap<String, String>>(query).ok())
            .and_then(|params| params.get("authentication").cloned())
        {
            Some(token) => token,
            None => {
                tracing::error!("No token found in request");
                return handle_auth_error("No token found");
            }
        }
    } else {
        bearer_token.unwrap().to_string()
    };

    let user = match authorize_current_user(&token).await {
        Ok(user) => match user {
            Some(user) => user,
            None => return Err(StatusCode::UNAUTHORIZED),
        },
        Err(e) => {
            tracing::error!("Authorization error: {}", e);
            return handle_auth_error("invalid jwt");
        }
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

async fn authorize_current_user(token: &str) -> Result<Option<User>> {
    let pg_pool = get_pg_pool();
    
    let _conn = pg_pool.get().await.map_err(|e| {
        tracing::error!("Pool connection error in auth: {:?}", e);
        anyhow!("Database connection error in auth")
    })?;

    let key = env::var("JWT_SECRET").expect("JWT_SECRET is not set");

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["authenticated", "api"]);

    let token_data =
        match decode::<JwtClaims>(token, &DecodingKey::from_secret(key.as_ref()), &validation) {
            Ok(jwt_claims) => jwt_claims.claims,
            Err(e) => {
                return Err(anyhow!("Error while decoding the token: {}", e));
            }
        };

    let user = match token_data.aud.contains("api") {
        true => User::find_by_api_key(token, &pg_pool).await,
        false => User::find_by_id(&Uuid::parse_str(&token_data.sub).unwrap(), &pg_pool).await,
    };

    let user = match user {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Error while querying user: {}", e);
            return Err(anyhow!("Error while querying user: {}", e));
        }
    };

    Ok(user)
}
