use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{
    insert_into, query_builder::AsChangeset, update, BoolExpressionMethods, ExpressionMethods,
    QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;

use uuid::Uuid;

use crate::{
    database::{
        enums::AssetType,
        lib::get_pg_pool,
        models::{Message, ThreadToDashboard, User},
        schema::{messages, threads, threads_to_dashboards},
    },
    routes::ws::{
        ws::{SubscriptionRwLock, WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{get_key_value, send_error_message, send_ws_message, subscribe_to_stream},
    },
    utils::{
        clients::{sentry_utils::send_sentry_error, supabase_vault::create_secret},
        sharing::asset_sharing::{
            create_asset_collection_association, delete_asset_collection_association,
            update_asset_permissions, ShareWithTeamsReqObject, ShareWithUsersReqObject,
        },
    },
};

use crate::utils::serde_helpers::deserialization_helpers::deserialize_double_option;

use super::{
    messages_utils::MessageDraftState,
    thread_utils::get_thread_state_by_id,
    threads_router::{ThreadEvent, ThreadRoute},
};

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateThreadRequest {
    pub id: Uuid,
    pub state_message_id: Option<Uuid>,
    pub save_draft: Option<bool>,
    pub save_to_dashboard: Option<Uuid>,
    pub remove_from_dashboard: Option<Uuid>,
    pub publicly_accessible: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub public_password: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_double_option")]
    pub public_expiry_date: Option<Option<DateTime<Utc>>>,
    pub team_permissions: Option<Vec<ShareWithTeamsReqObject>>,
    pub user_permissions: Option<Vec<ShareWithUsersReqObject>>,
    pub remove_teams: Option<Vec<Uuid>>,
    pub remove_users: Option<Vec<Uuid>>,
    pub add_to_collections: Option<Vec<Uuid>>,
    pub remove_from_collections: Option<Vec<Uuid>>,
}

pub async fn update_thread(
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
    req: UpdateThreadRequest,
) -> Result<()> {
    let subscription = format!("thread:{}", req.id);

    match subscribe_to_stream(subscriptions, &subscription, user_group, &user.id).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error subscribing to thread: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    }

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

    let thread_id = Arc::new(req.id);
    let user_id = Arc::new(user.id.clone());

    let update_thread_record_handle = if req.state_message_id.is_some()
        || (req.save_draft.is_some() && draft_session_id.is_some())
        || req.publicly_accessible.is_some()
        || req.public_password.is_some()
        || req.public_expiry_date.is_some()
        || req.save_to_dashboard.is_some()
        || req.remove_from_dashboard.is_some()
    {
        let thread_id = Arc::clone(&thread_id);
        let user_id = Arc::clone(&user_id);
        let draft_session_id = draft_session_id.clone();
        Some(tokio::spawn(async move {
            update_thread_record(
                user_id,
                thread_id,
                req.public_password,
                req.publicly_accessible,
                req.public_expiry_date,
                req.state_message_id,
                req.save_draft,
                draft_session_id,
                req.save_to_dashboard,
                req.remove_from_dashboard,
            )
            .await
        }))
    } else {
        None
    };

    let update_thread_collections_handle =
        if req.add_to_collections.is_some() || req.remove_from_collections.is_some() {
            let thread_id = Arc::clone(&thread_id);
            let user_id = Arc::clone(&user_id);
            Some(tokio::spawn(async move {
                match update_thread_collections(
                    thread_id,
                    user_id,
                    req.add_to_collections,
                    req.remove_from_collections,
                )
                .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        return Err(e);
                    }
                }
            }))
        } else {
            None
        };

    let update_thread_permissions_handle = if req.team_permissions.is_some()
        || req.user_permissions.is_some()
        || req.remove_teams.is_some()
        || req.remove_users.is_some()
    {
        let thread_id = Arc::clone(&thread_id);
        let user = Arc::new(user.clone());
        Some(tokio::spawn(async move {
            match update_asset_permissions(
                user,
                thread_id,
                AssetType::Thread,
                req.team_permissions,
                req.user_permissions,
                req.remove_teams,
                req.remove_users,
            )
            .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(e);
                }
            }
        }))
    } else {
        None
    };

    if let Some(update_thread_permissions_handle) = update_thread_permissions_handle {
        match update_thread_permissions_handle.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Unablse to update dashboard permissions: {}", e));
            }
        }
    };

    if let Some(update_thread_record_handle) = update_thread_record_handle {
        match update_thread_record_handle.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Unable to update dashboard record: {}", e));
            }
        }
    };

    if let Some(update_thread_collections_handle) = update_thread_collections_handle {
        match update_thread_collections_handle.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Unable to update thread collections: {}", e));
            }
        }
    };

    let thread_state = match get_thread_state_by_id(&user.id, &thread_id, &draft_session_id).await {
        Ok(thread) => thread,
        Err(e) => {
            tracing::error!("Error getting thread: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Threads(ThreadRoute::Update),
                WsEvent::Threads(ThreadEvent::GetThreadState),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(anyhow!("Unable to get thread state: {}", e));
        }
    };

    let update_thread_res_message = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::Update),
        WsEvent::Threads(ThreadEvent::UpdateThreadState),
        vec![thread_state],
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&subscription, &update_thread_res_message).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending message to pubsub: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    }

    Ok(())
}

#[derive(AsChangeset)]
#[diesel(table_name = threads)]
pub struct ThreadChangeset {
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
    pub state_message_id: Option<Uuid>,
    pub publicly_accessible: Option<bool>,
    pub publicly_enabled_by: Option<Uuid>,
    pub password_secret_id: Option<Option<Uuid>>,
    pub public_expiry_date: Option<Option<DateTime<Utc>>>,
}

async fn update_thread_record(
    user_id: Arc<Uuid>,
    thread_id: Arc<Uuid>,
    public_password: Option<Option<String>>,
    publicly_accessible: Option<bool>,
    public_expiry_date: Option<Option<DateTime<Utc>>>,
    state_message_id: Option<Uuid>,
    save_draft: Option<bool>,
    draft_session_id: Option<Uuid>,
    save_to_dashboard: Option<Uuid>,
    remove_from_dashboard: Option<Uuid>,
) -> Result<()> {
    let password_secret_id = match public_password {
        Some(Some(password)) => {
            // Password provided - create new secret
            match create_secret(&password).await {
                Ok(secret_id) => Some(Some(secret_id)),
                Err(e) => {
                    tracing::error!("Error creating secret: {}", e);
                    return Err(anyhow!("Error creating secret: {}", e));
                }
            }
        }
        Some(None) => Some(None), // Explicitly set to null
        None => None, // Not included in request
    };

    let public_expiry_date = match public_expiry_date {
        Some(Some(date)) => Some(Some(date)), // Date provided
        Some(None) => Some(None), // Explicitly set to null
        None => None, // Not included in request
    };

    let publicly_enabled_by = if let Some(publicly_accessible) = publicly_accessible {
        if publicly_accessible {
            Some(*user_id)
        } else {
            None
        }
    } else {
        None
    };

    let changeset = ThreadChangeset {
        updated_at: Utc::now(),
        updated_by: *user_id,
        state_message_id,
        publicly_accessible,
        publicly_enabled_by,
        password_secret_id,
        public_expiry_date,
    };

    let update_thread_record_handle = {
        let mut conn = match get_pg_pool().get().await {
            Ok(conn) => conn,
            Err(e) => {
                return Err(anyhow!("Error getting connection from pool: {}", e));
            }
        };

        let thread_id = Arc::clone(&thread_id);

        tokio::spawn(async move {
            match update(threads::table)
                .filter(threads::id.eq(*thread_id))
                .set(changeset)
                .execute(&mut conn)
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    return Err(anyhow!("Error updating thread: {}", e));
                }
            };

            Ok(())
        })
    };

    let save_draft_handle =
        if let (Some(_), Some(draft_session_id)) = (save_draft, draft_session_id) {
            let thread_id = Arc::clone(&thread_id);
            Some(tokio::spawn(async move {
                save_draft_handler(thread_id, draft_session_id).await
            }))
        } else {
            None
        };

    let save_to_dashboard_handle = if let Some(save_to_dashboard) = save_to_dashboard {
        let user_id = Arc::clone(&user_id);
        let thread_id = Arc::clone(&thread_id);
        let dashboard_id = save_to_dashboard;

        Some(tokio::spawn(async move {
            save_to_dashboard_handler(user_id, dashboard_id, thread_id).await
        }))
    } else {
        None
    };

    let remove_from_dashboard_handle = if let Some(remove_from_dashboard) = remove_from_dashboard {
        let thread_id = Arc::clone(&thread_id);
        let dashboard_id = remove_from_dashboard;

        Some(tokio::spawn(async move {
            remove_from_dashboard_handler(dashboard_id, thread_id).await
        }))
    } else {
        None
    };

    match update_thread_record_handle.await.unwrap() {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Unable to update thread record: {}", e));
        }
    }

    if let Some(save_to_dashboard_handle) = save_to_dashboard_handle {
        match save_to_dashboard_handle.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Unable to save to dashboard: {}", e));
            }
        }
    }

    if let Some(remove_from_dashboard_handle) = remove_from_dashboard_handle {
        match remove_from_dashboard_handle.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Unable to remove from dashboard: {}", e));
            }
        }
    }

    if let Some(save_draft_handle) = save_draft_handle {
        match save_draft_handle.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Unable to save draft:thread: {}", e));
            }
        }
    }

    Ok(())
}

async fn save_to_dashboard_handler(
    user_id: Arc<Uuid>,
    dashboard_id: Uuid,
    thread_id: Arc<Uuid>,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting connection from pool: {}", e));
        }
    };

    match insert_into(threads_to_dashboards::table)
        .values(ThreadToDashboard {
            thread_id: *thread_id,
            dashboard_id: dashboard_id,
            created_at: chrono::Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            added_by: *user_id,
        })
        .on_conflict((
            threads_to_dashboards::thread_id,
            threads_to_dashboards::dashboard_id,
        ))
        .do_update()
        .set((
            threads_to_dashboards::updated_at.eq(chrono::Utc::now()),
            threads_to_dashboards::deleted_at.eq(Option::<DateTime<Utc>>::None),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error saving to dashboard: {}", e);
            return Err(anyhow!("Unable to upsert thread to dashboard: {}", e));
        }
    }

    Ok(())
}

async fn remove_from_dashboard_handler(dashboard_id: Uuid, thread_id: Arc<Uuid>) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting connection from pool: {}", e));
        }
    };

    match update(threads_to_dashboards::table)
        .filter(threads_to_dashboards::thread_id.eq(*thread_id))
        .filter(threads_to_dashboards::dashboard_id.eq(dashboard_id))
        .set(threads_to_dashboards::deleted_at.eq(Some(chrono::Utc::now())))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error saving to dashboard: {}", e)),
    }

    Ok(())
}

async fn update_thread_collections(
    thread_id: Arc<Uuid>,
    user_id: Arc<Uuid>,
    add_to_collections: Option<Vec<Uuid>>,
    remove_from_collections: Option<Vec<Uuid>>,
) -> Result<()> {
    let add_to_collection_handle = if let Some(add_to_collections) = add_to_collections {
        let thread_id = Arc::clone(&thread_id);
        let user_id = Arc::clone(&user_id);
        Some(tokio::spawn(async move {
            create_asset_collection_association(
                add_to_collections,
                thread_id.clone(),
                AssetType::Thread,
                user_id,
            )
            .await
        }))
    } else {
        None
    };

    let remove_from_collection_handle =
        if let Some(remove_from_collections) = remove_from_collections {
            let thread_id = Arc::clone(&thread_id);
            let user_id = Arc::clone(&user_id);
            Some(tokio::spawn(async move {
                delete_asset_collection_association(
                    remove_from_collections,
                    thread_id.clone(),
                    AssetType::Thread,
                    user_id,
                )
                .await
            }))
        } else {
            None
        };

    if let Some(add_to_collection_handle) = add_to_collection_handle {
        match add_to_collection_handle.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error adding to collection: {}", e);
                return Err(anyhow!("Error adding to collection: {}", e));
            }
        }
    }

    if let Some(remove_from_collection_handle) = remove_from_collection_handle {
        match remove_from_collection_handle.await.unwrap() {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error removing from collection: {}", e);
                return Err(anyhow!("Error removing from collection: {}", e));
            }
        }
    }

    Ok(())
}

async fn save_draft_handler(thread_id: Arc<Uuid>, draft_session_id: Uuid) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting connection from pool: {}", e));
        }
    };

    let mut most_recent_message = match messages::table
        .select(messages::all_columns)
        .filter(messages::thread_id.eq(*thread_id))
        .filter(
            messages::draft_session_id
                .eq(&draft_session_id)
                .or(messages::draft_session_id.is_null()),
        )
        .order(messages::created_at.desc())
        .first::<Message>(&mut conn)
        .await
    {
        Ok(message) => message,
        Err(e) => return Err(anyhow!("Error getting most recent message: {}", e)),
    };

    let update_thread_handle = {
        let thread_id = Arc::clone(&thread_id);
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => return Err(anyhow!("Error getting connection from pool: {}", e)),
            };
            match update(threads::table)
                .filter(threads::id.eq(*thread_id))
                .set(threads::state_message_id.eq(most_recent_message.id))
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Error saving draft:thread: {}", e)),
            }
        })
    };

    let update_messages_handle = if most_recent_message.draft_session_id.is_some() {
        let thread_id = Arc::clone(&thread_id);
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => return Err(anyhow!("Error getting connection from pool: {}", e)),
            };
            match update(messages::table)
                .filter(messages::thread_id.eq(*thread_id))
                .filter(messages::draft_session_id.eq(Some(&draft_session_id)))
                .set(messages::draft_session_id.eq(None::<Uuid>))
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Error saving draft:thread: {}", e)),
            }
        })
    } else {
        if let Some(message_draft_state) = &most_recent_message.draft_state {
            let message_draft_state: MessageDraftState =
                serde_json::from_value(message_draft_state.clone()).unwrap();

            if let Some(title) = message_draft_state.title {
                most_recent_message.title = Some(title);
            };

            if let Some(chart_config) = message_draft_state.chart_config {
                most_recent_message.chart_config = Some(chart_config);
            };

            if let Some(code) = message_draft_state.code {
                most_recent_message.code = Some(code);
            };

            tokio::spawn(async move {
                match update(messages::table)
                    .filter(messages::id.eq(most_recent_message.id))
                    .set(most_recent_message)
                    .execute(&mut conn)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(anyhow!("Error saving draft:thread: {}", e)),
                }
            })
        } else {
            tokio::spawn(async move { Ok(()) })
        }
    };

    match update_thread_handle.await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Err(e),
        Err(e) => return Err(anyhow!("Error joining update_thread_handle: {}", e)),
    }

    match update_messages_handle.await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => return Err(e),
        Err(e) => return Err(anyhow!("Error joining update_messages_handle: {}", e)),
    }

    Ok(())
}
