use anyhow::{anyhow, Result};
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use uuid::Uuid;

use crate::{
    database::{
        enums::{AssetPermissionRole, AssetType, IdentityType},
        lib::{get_pg_pool, FetchingData, StepProgress},
        models::{AssetPermission, Message, User},
        schema::{asset_permissions, messages, threads},
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
    thread_utils::{get_thread_state_by_id, ThreadState},
    threads_router::{ThreadEvent, ThreadRoute},
};

#[derive(Deserialize, Debug, Clone)]
pub struct DuplicateThreadRequest {
    pub id: Uuid,
    pub message_id: Uuid,
    pub share_with_same_people: Option<bool>,
}

#[derive(Serialize, Debug)]
pub struct JoinThreadResponse {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub email: String,
    pub name: Option<String>,
}

pub async fn duplicate_thread(
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
    req: DuplicateThreadRequest,
) -> Result<()> {
    let new_thread_id = Uuid::new_v4();

    let subscription = format!("thread:{}", new_thread_id);

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

    let old_thread_state = match get_thread_state_by_id(&user.id, &req.id, &draft_session_id).await
    {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting thread: {}", e);
            let err = anyhow!("Error getting thread: {}", e);
            return Err(err);
        }
    };

    let new_thread_state = match duplicate_thread_handler(
        &user.id,
        &old_thread_state,
        &req.message_id,
        &req.share_with_same_people,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error duplicating thread: {}", e);
            let err = anyhow!("Error duplicating thread: {}", e);
            return Err(err);
        }
    };

    let join_thread_response = JoinThreadResponse {
        id: user.id.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        thread_id: new_thread_state.thread.id,
    };

    let join_thread_ws_message = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::DuplicateThread),
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
        WsRoutes::Threads(ThreadRoute::DuplicateThread),
        WsEvent::Threads(ThreadEvent::GetThreadState),
        vec![&new_thread_state],
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&subscription, &get_thread_ws_message).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending message to pubsub: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(anyhow!("Error sending message to pubsub: {}", e));
        }
    }

    let state_message = if new_thread_state
        .messages
        .iter()
        .any(|message| message.message.draft_session_id.is_some())
    {
        new_thread_state.messages.last()
    } else {
        new_thread_state.messages.iter().find(|&message| {
            message.message.id
                == new_thread_state
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
        data: if data.is_empty() { None } else { Some(data) },
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

async fn duplicate_thread_handler(
    user_id: &Uuid,
    thread_state: &ThreadState,
    message_id: &Uuid,
    share_with_same_people: &Option<bool>,
) -> Result<ThreadState> {
    let mut thread_state = thread_state.clone();

    let new_thread_id = Uuid::new_v4();
    let old_thread_id = thread_state.thread.id;

    thread_state.thread.id = new_thread_id;
    thread_state.thread.parent_thread_id = Some(old_thread_id);

    let target_message = thread_state
        .messages
        .iter()
        .find(|m| m.message.id == *message_id)
        .ok_or_else(|| anyhow!("Message with specified ID not found"))?;

    let target_created_at = target_message.message.created_at;

    thread_state
        .messages
        .retain(|m| m.message.created_at <= target_created_at);

    for message in &mut thread_state.messages {
        message.message.id = Uuid::new_v4();
        message.message.title = Some(format!("{} (Copy)", message.message.title.clone().unwrap_or_default()));
        message.message.thread_id = new_thread_id;
        message.message.draft_session_id = None;
        message.message.created_at = chrono::Utc::now();
        message.message.updated_at = chrono::Utc::now();
    }

    if let Some(most_recent_message) = thread_state
        .messages
        .iter()
        .max_by_key(|m| m.message.created_at)
    {
        thread_state.thread.state_message_id = Some(most_recent_message.message.id);
    } else {
        thread_state.thread.state_message_id = None;
    }

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    match insert_into(threads::table)
        .values(&thread_state.thread)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting thread: {}", e)),
    }

    let bulk_messages: Vec<Message> = thread_state
        .messages
        .iter()
        .map(|m| m.message.clone())
        .collect();

    let insert_messages_handle = {
        let mut conn = match get_pg_pool().get().await {
            Ok(conn) => conn,
            Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
        };
        tokio::spawn(async move {
            match insert_into(messages::table)
                .values(bulk_messages)
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Error inserting messages: {}", e)),
            }
        })
    };

    let asset_permission = AssetPermission {
        asset_id: thread_state.thread.id.clone(),
        identity_id: user_id.clone(),
        identity_type: IdentityType::User,
        asset_type: AssetType::Thread,
        role: AssetPermissionRole::Owner,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        created_by: user_id.clone(),
        updated_by: user_id.clone(),
    };

    let insert_asset_permission_handle = {
        let mut conn = match get_pg_pool().get().await {
            Ok(conn) => conn,
            Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
        };
        tokio::spawn(async move {
            match insert_into(asset_permissions::table)
                .values(asset_permission)
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Error inserting asset permission: {}", e)),
            }
        })
    };

    let insert_previous_permissions_handle =
        if let Some(share_with_same_people) = share_with_same_people {
            if *share_with_same_people {
                let user_id = user_id.clone();
                let thread_state = thread_state.clone();
                let mut conn = match get_pg_pool().get().await {
                    Ok(conn) => conn,
                    Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
                };
                tokio::spawn(async move {
                    let individual_permissions =
                        if let Some(individual_permissions) = thread_state.individual_permissions {
                            individual_permissions
                                .iter()
                                .map(|p| AssetPermission {
                                    asset_id: new_thread_id.clone(),
                                    identity_id: p.id,
                                    identity_type: IdentityType::User,
                                    asset_type: AssetType::Thread,
                                    role: p.role,
                                    created_at: chrono::Utc::now(),
                                    updated_at: chrono::Utc::now(),
                                    deleted_at: None,
                                    created_by: user_id.clone(),
                                    updated_by: user_id.clone(),
                                })
                                .collect::<Vec<AssetPermission>>()
                        } else {
                            vec![]
                        };

                    let team_permissions =
                        if let Some(team_permissions) = thread_state.team_permissions {
                            team_permissions
                                .iter()
                                .map(|p| AssetPermission {
                                    asset_id: new_thread_id.clone(),
                                    identity_id: p.id,
                                    identity_type: IdentityType::Team,
                                    asset_type: AssetType::Thread,
                                    role: p.role,
                                    created_at: chrono::Utc::now(),
                                    updated_at: chrono::Utc::now(),
                                    deleted_at: None,
                                    created_by: user_id.clone(),
                                    updated_by: user_id.clone(),
                                })
                                .collect::<Vec<AssetPermission>>()
                        } else {
                            vec![]
                        };

                    let mut combined_permissions = vec![];
                    combined_permissions.extend(individual_permissions);
                    combined_permissions.extend(team_permissions);

                    match insert_into(asset_permissions::table)
                        .values(combined_permissions)
                        .execute(&mut conn)
                        .await
                    {
                        Ok(_) => (),
                        Err(e) => return Err(anyhow!("Error inserting asset permissions: {}", e)),
                    };

                    Ok(())
                })
            } else {
                tokio::spawn(async move { Ok(()) })
            }
        } else {
            tokio::spawn(async move { Ok(()) })
        };

    match insert_messages_handle.await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Err(anyhow!("Error inserting messages: {}", e)),
        Err(e) => return Err(anyhow!("Error spawning insert messages task: {}", e)),
    };

    match insert_previous_permissions_handle.await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Err(anyhow!("Error inserting previous permissions: {}", e)),
        Err(e) => {
            return Err(anyhow!(
                "Error spawning insert previous permissions task: {}",
                e
            ))
        }
    };

    match insert_asset_permission_handle.await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Err(anyhow!("Error inserting asset permission: {}", e)),
        Err(e) => {
            return Err(anyhow!(
                "Error spawning insert asset permission task: {}",
                e
            ))
        }
    };

    Ok(thread_state)
}
