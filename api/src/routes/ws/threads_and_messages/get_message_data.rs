use anyhow::{anyhow, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        lib::{FetchingData, StepProgress},
        models::User,
    },
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
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
    messages_utils::get_message_with_permission,
    threads_router::{ThreadEvent, ThreadRoute},
};

#[derive(Deserialize, Debug, Clone)]
pub struct GetMessageDataRequest {
    pub id: Uuid,
}

pub async fn get_message_data(user: &User, req: GetMessageDataRequest) -> Result<()> {
    let (message, _) = match get_message_with_permission(&req.id, &user.id).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting message: {}", e);
            let err = anyhow!("Error getting message: {}", e);
            return Err(err);
        }
    };

    if let (Some(sql), Some(dataset_id)) = (message.code, message.dataset_id) {
        match fetch_data_handler(
            &user.id.to_string(),
            user,
            &sql,
            &dataset_id,
            &message.thread_id,
            &message.id,
        )
        .await
        {
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
                message_id: message.id.clone(),
                thread_id: message.thread_id.clone(),
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
        WsSendMethod::All,
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
    match send_fetching_data_in_progress_to_sub(subscription, user, thread_id, message_id, sql)
        .await
    {
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
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
        chart_config: None,
        code: Some(sql.clone()),
    };

    let fetching_data_ws_response = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::Get),
        WsEvent::Threads(ThreadEvent::FetchingData),
        Some(fetching_data_body),
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(subscription, &fetching_data_ws_response).await {
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
