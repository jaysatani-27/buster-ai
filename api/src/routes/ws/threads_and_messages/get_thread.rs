use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use uuid::Uuid;

use crate::{
    database::{
        enums::AssetPermissionRole,
        lib::{FetchingData, StepProgress},
        models::User,
    },
    routes::ws::{
        ws::{SubscriptionRwLock, WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{get_key_value, send_error_message, send_ws_message, subscribe_to_stream},
    },
    utils::{
        clients::{
            posthog::{
                send_posthog_event_handler, MetricViewedProperties, PosthogEventProperties,
                PosthogEventType,
            },
            sentry_utils::send_sentry_error,
        },
        query_engine::query_engine::query_engine,
    },
};

use super::{
    thread_utils::get_thread_state_by_id,
    threads_router::{ThreadEvent, ThreadRoute},
};

#[derive(Deserialize, Debug, Clone)]
pub struct GetThreadRequest {
    pub id: Uuid,
    pub password: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct JoinThreadResponse {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub email: String,
    pub name: Option<String>,
}

pub async fn get_thread(
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
    req: GetThreadRequest,
) -> Result<()> {
    let subscription = format!("thread:{}", req.id);

    let draft_session_id = match get_key_value(&format!("draft:thread:{}", req.id)).await {
        Ok(value) => match value {
            Some(value) => Some(Uuid::parse_str(&value).unwrap()),
            None => None,
        },
        Err(e) => return Err(e),
    };

    match subscribe_to_stream(subscriptions, &subscription, user_group, &user.id).await {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error subscribing to thread: {}", e)),
    };

    let mut thread_state = match get_thread_state_by_id(&user.id, &req.id, &draft_session_id).await
    {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting thread: {}", e);
            let err = anyhow!("Error getting thread: {}", e);
            return Err(err);
        }
    };

    if thread_state.permission.is_none() {
        if let Some(password) = &thread_state.public_password {
            if thread_state.thread.publicly_accessible
                && (thread_state.thread.public_expiry_date.is_none()
                    || thread_state.thread.public_expiry_date > Some(chrono::Utc::now()))
            {
                if let Some(submitted_password) = &req.password {
                    if password != submitted_password {
                        send_error_message(
                            &user.id.to_string(),
                            WsRoutes::Threads(ThreadRoute::Get),
                            WsEvent::Threads(ThreadEvent::GetThreadState),
                            WsErrorCode::Unauthorized,
                            "Invalid password".to_string(),
                            user,
                        )
                        .await?;
                        return Ok(());
                    }
                }
            }
        }

        thread_state.permission = Some(AssetPermissionRole::Viewer);
        thread_state.public_password = None;
        thread_state.organization_permissions = false;
        thread_state.individual_permissions = None;
        thread_state.team_permissions = None;
        thread_state.collections = vec![];
        thread_state.dashboards = vec![];
    };

    let join_thread_response = JoinThreadResponse {
        id: user.id.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        thread_id: thread_state.thread.id,
    };

    let join_thread_ws_message = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::Get),
        WsEvent::Threads(ThreadEvent::JoinedThread),
        vec![join_thread_response],
        None,
        user,
        WsSendMethod::AllButSender,
    );

    match send_ws_message(&subscription, &join_thread_ws_message).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending message to pubsub: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(anyhow!("Error sending message to pubsub: {}", e));
        }
    }

    let get_thread_ws_message = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::Get),
        WsEvent::Threads(ThreadEvent::GetThreadState),
        vec![&thread_state],
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&subscription, &get_thread_ws_message).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending message to pubsub: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(anyhow!("Error sending message to pubsub: {}", e));
        }
    }

    let state_message = if thread_state
        .messages
        .iter()
        .any(|message| message.message.draft_session_id.is_some())
    {
        thread_state.messages.last()
    } else {
        thread_state.messages.iter().find(|&message| {
            message.message.id
                == thread_state
                    .thread
                    .state_message_id
                    .expect("No state message id found in thread")
        })
    };

    let state_message = match state_message {
        Some(message) => message,
        None => {
            let err = anyhow!("No messages found with the specified state_message_id in thread");
            send_sentry_error(&err.to_string(), Some(&user.id));
            tracing::error!("No messages found with the specified state_message_id in thread");
            return Err(err);
        }
    };

    let sql = match &state_message.message.code {
        Some(sql) => sql,
        None => {
            let err = anyhow!("No SQL found in message");
            send_sentry_error(&err.to_string(), Some(&user.id));
            tracing::error!("No SQL found in message");
            return Err(err);
        }
    };

    let dataset_id = match &state_message.message.dataset_id {
        Some(dataset_id) => dataset_id,
        None => {
            let err = anyhow!("No dataset id found in message");
            send_sentry_error(&err.to_string(), Some(&user.id));
            tracing::error!("No dataset id found in message");
            return Err(err);
        }
    };

    let thread_id = &state_message.message.thread_id;
    let message_id = &state_message.message.id;

    match fetch_data_handler(&subscription, user, sql, dataset_id, thread_id, message_id).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error fetching data: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
        }
    }

    match send_posthog_event_handler(
        PosthogEventType::MetricViewed,
        Some(user.id),
        PosthogEventProperties::MetricViewed(MetricViewedProperties {
            message_id: message_id.clone(),
            thread_id: thread_id.clone(),
        }),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error sending posthog event: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
        }
    }

    Ok(())
}

async fn send_fetching_data_in_progress_to_sub(
    subscription: &String,
    user: &User,
    thread_id: &Uuid,
    message_id: &Uuid,
    sql: &String,
) -> Result<()> {
    let fetching_data_body = FetchingData {
        progress: StepProgress::InProgress,
        data: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
        chart_config: None,
        code: Some(sql.clone()),
    };

    let identify_dataset_ws_response = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::Get),
        WsEvent::Threads(ThreadEvent::FetchingData),
        Some(fetching_data_body),
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(subscription, &identify_dataset_ws_response).await {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    Ok(())
}

async fn fetch_data_handler(
    subscription: &String,
    user: &User,
    sql: &String,
    dataset_id: &Uuid,
    thread_id: &Uuid,
    message_id: &Uuid,
) -> Result<()> {
    match send_fetching_data_in_progress_to_sub(subscription, user, thread_id, message_id, sql).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(
                "Unable to send fetching data in progress to subscription: {:?}",
                e
            );
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    }

    let data = match query_engine(&dataset_id, &sql).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Unable to query engine: {:?}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &subscription,
                WsRoutes::Threads(ThreadRoute::Get),
                WsEvent::Threads(ThreadEvent::FetchingData),
                WsErrorCode::InternalServerError,
                "Failed to fetch results.".to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let fetching_data_body = FetchingData {
        progress: StepProgress::Completed,
        data: if data.is_empty() {
            Some(vec![])
        } else {
            Some(data)
        },
        code: Some(sql.clone()),
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
        chart_config: None,
    };

    let fetching_data_ws_response = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::Get),
        WsEvent::Threads(ThreadEvent::FetchingData),
        Some(fetching_data_body),
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&subscription, &fetching_data_ws_response).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(
                "Unable to send fetching data success to subscription: {:?}",
                e
            );
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    }

    Ok(())
}
