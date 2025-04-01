use anyhow::{anyhow, Result};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::models::User,
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::{
    terms_router::{TermEvent, TermRoute},
    terms_utils::{get_term_state, TermState},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTermRequest {
    pub id: Uuid,
}

pub async fn get_term(user: &User, req: GetTermRequest) -> Result<()> {
    let term_state = match get_term_handler(&user.id, &req.id).await {
        Ok(term_state) => term_state,
        Err(e) => {
            tracing::error!("Error getting term: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Terms(TermRoute::Get),
                WsEvent::Terms(TermEvent::GetTerm),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let get_term_message = WsResponseMessage::new(
        WsRoutes::Terms(TermRoute::Get),
        WsEvent::Terms(TermEvent::GetTerm),
        term_state,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &get_term_message).await {
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

async fn get_term_handler(user_id: &Uuid, term_id: &Uuid) -> Result<TermState> {
    get_term_state(user_id, term_id).await
}
