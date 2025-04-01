use anyhow::{anyhow, Result};
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        lib::get_pg_pool,
        models::User,
        schema::terms,
    },
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::terms_router::{TermEvent, TermRoute};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTermRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTermResponse {
    pub ids: Vec<Uuid>,
}

pub async fn delete_term(user: &User, req: DeleteTermRequest) -> Result<()> {
    let deleted_term_ids = match delete_term_handler(req.ids).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Error deleting term: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Terms(TermRoute::Delete),
                WsEvent::Terms(TermEvent::DeleteTerm),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let delete_term_response = DeleteTermResponse {
        ids: deleted_term_ids,
    };

    let delete_term_message = WsResponseMessage::new(
        WsRoutes::Terms(TermRoute::Delete),
        WsEvent::Terms(TermEvent::DeleteTerm),
        delete_term_response,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &delete_term_message).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error sending ws message: {}", e);
            let err = anyhow!("Error sending ws message: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(err);
        }
    }

    Ok(())
}

async fn delete_term_handler(term_ids: Vec<Uuid>) -> Result<Vec<Uuid>> {
    let term_ids = Arc::new(term_ids);

    let delete_term = {
        let term_ids = term_ids.clone();
        tokio::spawn(async move { mark_term_as_deleted(term_ids).await })
    };

    delete_term.await??;

    Ok(term_ids.to_vec())
}

async fn mark_term_as_deleted(term_ids: Arc<Vec<Uuid>>) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    match diesel::update(terms::table)
        .filter(terms::id.eq_any(term_ids.as_ref()))
        .set(terms::deleted_at.eq(Some(Utc::now())))
        .execute(&mut conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Error marking term as deleted: {}", e)),
    }
}
