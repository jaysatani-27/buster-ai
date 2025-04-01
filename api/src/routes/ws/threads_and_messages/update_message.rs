use anyhow::{anyhow, Result};
use diesel::{update, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use uuid::Uuid;

use crate::{
    database::{
        enums::{AssetPermissionRole, MessageFeedback, Verification},
        lib::{get_pg_pool, get_sqlx_pool},
        models::User,
        schema::{messages, threads},
    },
    routes::ws::{
        ws::{SubscriptionRwLock, WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{
            get_key_value, send_error_message, send_ws_message, set_key_value, subscribe_to_stream,
        },
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::{
    messages_utils::{get_message_with_permission, MessageDraftState},
    thread_utils::{check_if_thread_saved, get_thread_state_by_id},
    threads_router::{ThreadEvent, ThreadRoute},
};

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateMessageRequest {
    pub id: Uuid,
    #[serde(rename = "sql")]
    pub code: Option<String>,
    pub title: Option<String>,
    pub feedback: Option<MessageFeedback>,
    #[serde(rename = "status")]
    pub verification: Option<Verification>,
    pub chart_config: Option<Value>,
}

pub async fn update_message(
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
    req: UpdateMessageRequest,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            let err = anyhow!("Unable to get connection from pool: {}", e);
            send_sentry_error(&err.to_string(), Some(&user.id));
            return Err(err);
        }
    };

    let thread_id = match threads::table
        .select(threads::id)
        .inner_join(messages::table.on(threads::id.eq(messages::thread_id)))
        .filter(messages::id.eq(&req.id))
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(thread_id) => thread_id,
        Err(diesel::NotFound) => {
            tracing::error!("Thread not found");
            return Err(anyhow!("Thread not found"));
        }
        Err(e) => {
            tracing::error!("Error getting thread: {}", e);
            return Err(anyhow!("Error getting thread: {}", e));
        }
    };

    let draft_session_key = format!("draft:thread:{}", req.id);

    let draft_session_id = match get_key_value(&draft_session_key).await {
        Ok(value) => match value {
            Some(value) => Some(Uuid::parse_str(&value).unwrap()),
            None => None,
        },
        Err(e) => {
            tracing::error!("Error getting draft session id: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    };

    let subscription = format!("thread:{}", thread_id);

    match subscribe_to_stream(subscriptions, &subscription, user_group, &user.id).await {
        Ok(_) => (),
        Err(e) => {
            let err = anyhow!("Error subscribing to stream: {}", e);
            send_sentry_error(&err.to_string(), Some(&user.id));
            return Err(err);
        }
    }

    let draft_session_id = match update_message_handler(
        &req.id,
        user,
        req.code,
        req.title,
        req.feedback,
        req.verification,
        req.chart_config,
        draft_session_id,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            let err = anyhow!("Error getting thread: {}", e);
            tracing::error!("Error getting thread: {}", e);
            send_sentry_error(&err.to_string(), Some(&user.id));
            send_error_message(
                &subscription,
                WsRoutes::Threads(ThreadRoute::UpdateMessage),
                WsEvent::Threads(ThreadEvent::UpdateThreadState),
                WsErrorCode::InternalServerError,
                "Failed to update thread.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let thread = match get_thread_state_by_id(&user.id, &thread_id, &draft_session_id).await {
        Ok(thread) => thread,
        Err(e) => return Err(anyhow!("Error getting thread: {}", e)),
    };

    let update_message_message = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::UpdateMessage),
        WsEvent::Threads(ThreadEvent::UpdateThreadState),
        vec![thread],
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&subscription, &update_message_message).await {
        Ok(_) => {}
        Err(e) => {
            let err = anyhow!("Error sending message to pubsub: {}", e);
            send_sentry_error(&err.to_string(), Some(&user.id));
            return Err(err);
        }
    }

    Ok(())
}

async fn update_message_handler(
    message_id: &Uuid,
    user: &User,
    code: Option<String>,
    title: Option<String>,
    feedback: Option<MessageFeedback>,
    verification: Option<Verification>,
    chart_config: Option<Value>,
    draft_session_id: Option<Uuid>,
) -> Result<Option<Uuid>> {
    let (mut message, user_role) = match get_message_with_permission(&message_id, &user.id).await {
        Ok(message) => message,
        Err(e) => {
            tracing::error!("Error getting message: {}", e);
            let err = anyhow!("Error getting message: {}", e);
            send_sentry_error(&err.to_string(), Some(&user.id));
            return Err(err);
        }
    };

    if user_role == AssetPermissionRole::Viewer {
        return Err(anyhow!("User does not have permission to update message."));
    };

    let draft_session_id = if let Some(draft_session_id) = draft_session_id {
        Some(draft_session_id)
    } else {
        let draft_session_id = if code.is_some() || chart_config.is_some() || title.is_some() {
            let thread_saved = match check_if_thread_saved(&message.thread_id).await {
                Ok(result) => result,
                Err(e) => {
                    return Err(anyhow!("Unable to check if thread is saved: {}", e));
                }
            };

            let draft_session_id = if thread_saved {
                let draft_session_key = format!("draft:thread:{}", message.thread_id);
                let draft_session_id = Uuid::new_v4();
                set_key_value(&draft_session_key, &draft_session_id.to_string()).await?;
                Some(draft_session_id)
            } else {
                None
            };

            draft_session_id
        } else {
            None
        };

        draft_session_id
    };

    if let Some(draft_session_id) = draft_session_id {
        let mut draft_state = if let Some(draft_state) = message.draft_state {
            serde_json::from_value::<MessageDraftState>(draft_state).unwrap()
        } else {
            MessageDraftState {
                draft_session_id: draft_session_id.clone(),
                title: None,
                chart_config: None,
                code: None,
                deleted_at: None,
            }
        };

        if draft_state.draft_session_id != draft_session_id {
            draft_state.draft_session_id = draft_session_id;
            draft_state.title = None;
            draft_state.chart_config = None;
            draft_state.code = None;
            draft_state.deleted_at = None;
        }

        if let Some(code) = code {
            draft_state.code = Some(code);
        }

        if let Some(chart_config) = chart_config {
            draft_state.chart_config = Some(chart_config);
        }

        if let Some(title) = &title {
            draft_state.title = Some(title.clone());
        }

        message.draft_state = Some(serde_json::to_value(draft_state).unwrap());
    } else {
        if let Some(code) = code {
            message.code = Some(code);
        }

        if let Some(chart_config) = chart_config {
            message.chart_config = Some(chart_config);
        }

        if let Some(title) = &title {
            message.title = Some(title.clone());
        }
    }

    if let Some(feedback) = feedback {
        message.feedback = Some(feedback);
    }

    if let Some(verification) = verification {
        message.verification = verification;
    }

    let message_update_handle = {
        let message = message.clone();
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => {
                    let err = anyhow!("Unable to get connection from pool: {}", e);
                    send_sentry_error(&err.to_string(), None);
                    return Err(err);
                }
            };

            match update(messages::table)
                .set(&message)
                .filter(messages::id.eq(&message.id))
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Error updating message: {}", e)),
            }
        })
    };

    let thread_search_handle = if title.is_some() {
        let thread_id = message.thread_id;
        let summary_question = message
            .summary_question
            .unwrap_or(message.title.unwrap_or_default());
        Some(tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("Unable to get connection from pool: {:?}", e);
                    send_sentry_error(&e.to_string(), None);
                    return;
                }
            };

            let query = diesel::sql_query(
                "UPDATE asset_search 
                 SET content = $1, updated_at = NOW()
                 WHERE asset_id = $2 AND asset_type = 'thread'"
            )
            .bind::<diesel::sql_types::Text, _>(summary_question)
            .bind::<diesel::sql_types::Uuid, _>(thread_id);

            if let Err(e) = query.execute(&mut conn).await {
                tracing::error!("Failed to update asset search: {:?}", e);
                send_sentry_error(&e.to_string(), None);
            }
        }))
    } else {
        None
    };

    if let Err(e) = message_update_handle.await.unwrap() {
        return Err(e);
    }

    if let Some(handle) = thread_search_handle {
        if let Err(e) = handle.await {
            return Err(anyhow!("Error in thread search update: {:?}", e));
        }
    }

    Ok(draft_session_id)
}
