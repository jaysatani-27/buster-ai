use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{dsl::not, query_builder::AsChangeset, update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::{AssetPermissionRole, AssetType},
        lib::{get_pg_pool, get_sqlx_pool},
        models::{ThreadToDashboard, User},
        schema::{dashboards, threads_to_dashboards},
    },
    routes::ws::{
        dashboards::dashboards_router::{DashboardEvent, DashboardRoute},
        ws::{SubscriptionRwLock, WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message, subscribe_to_stream},
    },
    utils::{
        clients::{sentry_utils::send_sentry_error, supabase_vault::create_secret},
        sharing::asset_sharing::{
            create_asset_collection_association, delete_asset_collection_association,
            update_asset_permissions, ShareWithTeamsReqObject, ShareWithUsersReqObject,
        },
    },
};

use super::dashboard_utils::{get_dashboard_state_by_id, get_user_dashboard_permission};
use crate::utils::serde_helpers::deserialization_helpers::deserialize_double_option;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateDashboardRequest {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<Value>,
    pub threads: Option<Vec<Uuid>>,
    pub publicly_accessible: Option<bool>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_double_option")]
    pub public_password: Option<Option<String>>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_double_option")]
    pub public_expiry_date: Option<Option<chrono::NaiveDateTime>>,
    pub team_permissions: Option<Vec<ShareWithTeamsReqObject>>,
    pub user_permissions: Option<Vec<ShareWithUsersReqObject>>,
    pub remove_teams: Option<Vec<Uuid>>,
    pub remove_users: Option<Vec<Uuid>>,
    pub add_to_collections: Option<Vec<Uuid>>,
    pub remove_from_collections: Option<Vec<Uuid>>,
}

pub async fn update_dashboard(
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
    req: UpdateDashboardRequest,
) -> Result<()> {
    println!("Request received in update_dashboard: {:?}", req);
    let dashboard_id = req.id;

    let dashboard_permission = match get_user_dashboard_permission(&user.id, &dashboard_id).await {
        Ok(dashboard_permission) => match dashboard_permission {
            Some(dashboard_permission) => dashboard_permission,
            None => return Err(anyhow!("No dashboard permission found")),
        },
        Err(e) => {
            tracing::error!("Error getting dashboard permission: {}", e);
            return Err(anyhow!("Error getting dashboard permission: {}", e));
        }
    };

    if dashboard_permission == AssetPermissionRole::Viewer {
        tracing::error!("User does not have permission to update dashboard");
        return Err(anyhow!("User does not have permission to update dashboard"));
    }

    let dashboard_subscription = format!("dashboard:{}", dashboard_id);

    match subscribe_to_stream(subscriptions, &dashboard_subscription, user_group, &user.id).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error subscribing to user list subscription: {}", e);
            return Err(anyhow!(
                "Error subscribing to user list subscription: {}",
                e
            ));
        }
    };

    let user_id = Arc::new(user.id.clone());
    let dashboard_id = Arc::new(dashboard_id.clone());

    let update_dashboard_record_handle = {
        let user_id = Arc::clone(&user_id);
        let dashboard_id = Arc::clone(&dashboard_id);
        let req = req.clone();
        Some(tokio::spawn(async move {
            match update_dashboard_record(
                user_id,
                dashboard_id,
                req.name,
                req.description,
                req.config,
                req.publicly_accessible,
                req.public_password,
                req.public_expiry_date,
            )
            .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(e);
                }
            }
        }))
    };

    let update_dashboard_threads_handle = if let Some(threads) = req.threads {
        let dashboard_id = Arc::clone(&dashboard_id);
        let user_id = Arc::clone(&user_id);
        Some(tokio::spawn(async move {
            update_dashboard_threads(dashboard_id, user_id, threads).await
        }))
    } else {
        None
    };

    let update_dashboard_collections_handle =
        if req.add_to_collections.is_some() || req.remove_from_collections.is_some() {
            let dashboard_id = Arc::clone(&dashboard_id);
            let user_id = Arc::clone(&user_id);
            Some(tokio::spawn(async move {
                match update_dashboard_collections(
                    dashboard_id,
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

    let update_dashboard_permissions_handle = if req.team_permissions.is_some()
        || req.user_permissions.is_some()
        || req.remove_teams.is_some()
        || req.remove_users.is_some()
    {
        let dashboard_id = Arc::clone(&dashboard_id);
        let user = Arc::new(user.clone());
        Some(tokio::spawn(async move {
            match update_asset_permissions(
                user,
                dashboard_id,
                AssetType::Dashboard,
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

    if let Some(update_dashboard_permissions_handle) = update_dashboard_permissions_handle {
        match update_dashboard_permissions_handle.await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error updating dashboard permissions: {}", e);
                send_sentry_error(&e.to_string(), None);
                return Err(anyhow!("Error updating dashboard permissions: {}", e));
            }
        }
    }

    if let Some(update_dashboard_record_handle) = update_dashboard_record_handle {
        match update_dashboard_record_handle.await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error updating dashboard record: {}", e);
                send_sentry_error(&e.to_string(), None);
                return Err(anyhow!("Error updating dashboard record: {}", e));
            }
        }
    }

    if let Some(update_dashboard_collections_handle) = update_dashboard_collections_handle {
        match update_dashboard_collections_handle.await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error updating dashboard collections: {}", e);
                send_sentry_error(&e.to_string(), None);
                return Err(anyhow!("Error updating dashboard collections: {}", e));
            }
        }
    }

    if let Some(update_dashboard_threads_handle) = update_dashboard_threads_handle {
        match update_dashboard_threads_handle.await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error updating dashboard threads: {}", e);
                send_sentry_error(&e.to_string(), None);
                return Err(anyhow!("Error updating dashboard threads: {}", e));
            }
        }
    }

    let dashboard = match get_dashboard_state_by_id(&user.id, &req.id).await {
        Ok(dashboard) => dashboard,
        Err(e) => {
            tracing::error!("Error getting dashboard: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Dashboards(DashboardRoute::Update),
                WsEvent::Dashboards(DashboardEvent::GetDashboardState),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let dashboard_message_ws_message = WsResponseMessage::new(
        WsRoutes::Dashboards(DashboardRoute::Update),
        WsEvent::Dashboards(DashboardEvent::UpdateDashboard),
        dashboard,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&dashboard_subscription, &dashboard_message_ws_message).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending message to pubsub: {}", e);
            return Err(anyhow!("Error sending message to pubsub: {}", e));
        }
    }

    Ok(())
}

#[derive(AsChangeset)]
#[diesel(table_name = dashboards)]
pub struct DashboardChangeset {
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<Value>,
    pub publicly_accessible: Option<bool>,
    pub publicly_enabled_by: Option<Uuid>,
    pub password_secret_id: Option<Option<Uuid>>,
    pub public_expiry_date: Option<Option<chrono::NaiveDateTime>>,
}

async fn update_dashboard_record(
    user_id: Arc<Uuid>,
    dashboard_id: Arc<Uuid>,
    name: Option<String>,
    description: Option<String>,
    config: Option<Value>,
    publicly_accessible: Option<bool>,
    public_password: Option<Option<String>>,
    public_expiry_date: Option<Option<chrono::NaiveDateTime>>,
) -> Result<()> {
    let password_secret_id = match public_password {
        Some(Some(password)) => match create_secret(&password).await {
            Ok(secret_id) => Some(Some(secret_id)),
            Err(e) => {
                tracing::error!("Error creating secret: {}", e);
                return Err(anyhow!("Error creating secret: {}", e));
            }
        },
        Some(None) => Some(None),
        None => None,
    };

    let public_expiry_date = match public_expiry_date {
        Some(Some(date)) => Some(Some(date)),
        Some(None) => Some(None),
        None => None,
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

    let changeset = DashboardChangeset {
        updated_at: Utc::now(),
        updated_by: *user_id,
        name: name.clone(),
        description,
        config,
        publicly_accessible,
        publicly_enabled_by,
        password_secret_id,
        public_expiry_date,
    };

    let dashboard_update = {
        let dashboard_id = dashboard_id.clone();
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("Unable to get connection from pool: {:?}", e);
                    return Err(anyhow!("Unable to get connection from pool: {}", e));
                }
            };

            match update(dashboards::table)
                .filter(dashboards::id.eq(*dashboard_id))
                .set(&changeset)
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("Unable to update dashboard in database: {:?}", e);
                    let err = anyhow!("Unable to update dashboard in database: {}", e);
                    send_sentry_error(&e.to_string(), None);
                    Err(err)
                }
            }
        })
    };

    let dashboard_search_handle = {
        let dashboard_id = dashboard_id.clone();
        let dashboard_name = name.unwrap_or_default();
        tokio::spawn(async move {
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
                WHERE asset_id = $2 AND asset_type = 'dashboard'"
            )
            .bind::<diesel::sql_types::Text, _>(dashboard_name)
            .bind::<diesel::sql_types::Uuid, _>(*dashboard_id);

            if let Err(e) = query.execute(&mut conn).await {
                tracing::error!("Failed to update asset search: {:?}", e);
                send_sentry_error(&e.to_string(), None);
            }
        })
    };

    if let Err(e) = dashboard_update.await {
        return Err(anyhow!("Error in dashboard update: {:?}", e));
    }

    if let Err(e) = dashboard_search_handle.await {
        return Err(anyhow!("Error in dashboard search update: {:?}", e));
    }

    Ok(())
}

async fn update_dashboard_collections(
    dashboard_id: Arc<Uuid>,
    user_id: Arc<Uuid>,
    add_to_collections: Option<Vec<Uuid>>,
    remove_from_collections: Option<Vec<Uuid>>,
) -> Result<()> {
    let add_to_collection_handle = if let Some(add_to_collections) = add_to_collections {
        let dashboard_id = Arc::clone(&dashboard_id);
        let user_id = Arc::clone(&user_id);
        Some(tokio::spawn(async move {
            create_asset_collection_association(
                add_to_collections,
                dashboard_id.clone(),
                AssetType::Dashboard,
                user_id,
            )
            .await
        }))
    } else {
        None
    };

    let remove_from_collection_handle =
        if let Some(remove_from_collections) = remove_from_collections {
            let dashboard_id = Arc::clone(&dashboard_id);
            let user_id = Arc::clone(&user_id);
            Some(tokio::spawn(async move {
                delete_asset_collection_association(
                    remove_from_collections,
                    dashboard_id.clone(),
                    AssetType::Dashboard,
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

async fn update_dashboard_threads(
    dashboard_id: Arc<Uuid>,
    user_id: Arc<Uuid>,
    threads: Vec<Uuid>,
) -> Result<()> {
    let threads = Arc::new(threads);

    let upsert_handle = {
        let threads = Arc::clone(&threads);
        let dashboard_id = Arc::clone(&dashboard_id);
        let user_id = Arc::clone(&user_id);
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => return Err(anyhow!("Error getting pg pool: {:?}", e)),
            };

            let new_thread_records: Vec<ThreadToDashboard> = threads
                .iter()
                .map(|thread_id| ThreadToDashboard {
                    thread_id: *thread_id,
                    dashboard_id: *dashboard_id,
                    created_at: chrono::Utc::now(),
                    updated_at: Utc::now(),
                    deleted_at: None,
                    added_by: *user_id,
                })
                .collect();
            match diesel::insert_into(threads_to_dashboards::table)
                .values(&new_thread_records)
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
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("Error updating dashboard threads: {}", e);
                    Err(anyhow!("Unable to upsert threads to dashboard: {}", e))
                }
            }
        })
    };

    let remove_handle = {
        let threads = Arc::clone(&threads);
        let dashboard_id = Arc::clone(&dashboard_id);
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => return Err(anyhow!("Error getting pg pool: {:?}", e)),
            };

            match update(threads_to_dashboards::table)
                .filter(threads_to_dashboards::dashboard_id.eq(*dashboard_id))
                .filter(not(
                    threads_to_dashboards::thread_id.eq_any(threads.as_ref())
                ))
                .set(threads_to_dashboards::deleted_at.eq(Some(chrono::Utc::now())))
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    tracing::error!("Error removing threads from dashboard: {}", e);
                    Err(anyhow!("Error removing threads from dashboard: {}", e))
                }
            }
        })
    };

    match upsert_handle.await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => {
            tracing::error!("Error upserting threads to dashboard: {}", e);
            return Err(anyhow!("Error upserting threads to dashboard: {}", e));
        }
        Err(e) => {
            tracing::error!("Error upserting threads to dashboard: {}", e);
            return Err(anyhow!("Error upserting threads to dashboard: {}", e));
        }
    }

    match remove_handle.await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => {
            tracing::error!("Error removing threads from dashboard: {}", e);
            return Err(anyhow!("Error removing threads from dashboard: {}", e));
        }
        Err(e) => {
            tracing::error!("Error removing threads from dashboard: {}", e);
            return Err(anyhow!("Error removing threads from dashboard: {}", e));
        }
    }

    Ok(())
}
