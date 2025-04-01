use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    database::{
        enums::{AssetType, IdentityType, UserOrganizationRole, Verification},
        lib::{get_pg_pool, PgPool},
        models::{Message, User},
        schema::{
            asset_permissions, data_sources, datasets, messages, teams_to_users, threads, users,
            users_to_organizations,
        },
    },
    routes::ws::{
        threads_and_messages::threads_router::{ThreadEvent, ThreadRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::sentry_utils::send_sentry_error,
        sharing::asset_sharing::check_if_assets_are_shared,
        user::user_info::get_user_organization_id,
    },
};

/// The request contains the page number and the page size for listing and offsetting threads.
///
/// If page or page_size not provided, we fall to our defaults.
///
/// Returns a list of threads for the current user based on the threads_to_users_relationship.
///
/// the pagination request coming from the frontend begins with 0.

#[derive(Deserialize, Debug, Clone)]
pub struct ListThreadsFilters {
    #[serde(rename = "status")]
    pub verification: Option<Vec<Verification>>,
    pub user_id: Option<Uuid>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListThreadsRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub admin_view: Option<bool>,
    pub filters: Option<ListThreadsFilters>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ThreadUser {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ThreadDataset {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ThreadWithMostRecentMessageAndUserInfo {
    pub id: Uuid,
    pub title: String,
    pub status: Verification,
    pub last_edited: DateTime<Utc>,
    pub owner: ThreadUser,
    pub dataset: ThreadDataset,
    pub is_shared: bool,
    // TODO: Clean up these responses, just in for backwards compatibility
    pub created_by_id: Uuid,
    pub created_by_name: Option<String>,
    pub created_by_email: String,
    pub dataset_id: Uuid,
    pub dataset_name: String,
}

pub async fn list_threads(user: &User, req: ListThreadsRequest) -> Result<()> {
    let list_threads_res = match list_threads_handler(
        &user.id,
        req.filters,
        req.admin_view,
        req.page,
        req.page_size,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting threads: {}", e);
            let err = anyhow!("Error getting threads: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Threads(ThreadRoute::List),
                WsEvent::Threads(ThreadEvent::GetThreadsList),
                WsErrorCode::InternalServerError,
                "Failed to list threads.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let list_threads_ws_message = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::List),
        WsEvent::Threads(ThreadEvent::GetThreadsList),
        list_threads_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &list_threads_ws_message).await {
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

async fn list_threads_handler(
    user_id: &Uuid,
    filters: Option<ListThreadsFilters>,
    admin_view: Option<bool>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<Vec<ThreadWithMostRecentMessageAndUserInfo>> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(25);
    let admin_view = admin_view.unwrap_or(false);

    let messages = match admin_view {
        true => get_admin_threads(user_id, filters, page, page_size).await,
        false => get_user_threads(user_id, filters, page, page_size).await,
    };

    let messages = match messages {
        Ok(messages) => messages,
        Err(e) => return Err(anyhow!("Error getting messages: {}", e)),
    };

    let thread_ids: Vec<Uuid> = messages
        .iter()
        .map(|((thread_id, _, _, _), _, _)| *thread_id)
        .collect();

    let threads_share_status = match tokio::spawn(async move {
        check_if_assets_are_shared(&thread_ids, AssetType::Thread).await
    })
    .await
    {
        Ok(result) => match result {
            Ok(status) => status,
            Err(e) => return Err(anyhow!("Error checking if assets are shared: {}", e)),
        },
        Err(e) => return Err(anyhow!("Error joining task: {}", e)),
    };

    let mut messages_with_threads_and_user_info = Vec::new();

    for (
        (thread_id, title, verification, created_at),
        (thread_owner_id, thread_owner_name, thread_owner_email),
        (dataset_id, dataset_name),
    ) in messages
    {
        let is_shared = threads_share_status
            .iter()
            .find(|(id, _)| id == &thread_id)
            .map(|(_, shared)| *shared)
            .unwrap_or(false);

        let thread_info = ThreadWithMostRecentMessageAndUserInfo {
            created_by_id: thread_owner_id.clone(),
            created_by_name: thread_owner_name.clone(),
            created_by_email: thread_owner_email.clone(),
            dataset_id: dataset_id.clone(),
            dataset_name: dataset_name.clone(),
            id: thread_id,
            title: title.unwrap_or_else(|| "Untitled".to_string()),
            status: verification,
            last_edited: created_at,
            owner: ThreadUser {
                id: thread_owner_id,
                name: thread_owner_name,
                email: thread_owner_email,
            },
            dataset: ThreadDataset {
                id: dataset_id,
                name: dataset_name,
            },
            is_shared,
        };

        messages_with_threads_and_user_info.push(thread_info);
    }

    Ok(messages_with_threads_and_user_info)
}

async fn get_user_threads(
    user_id: &Uuid,
    filters: Option<ListThreadsFilters>,
    page: i64,
    page_size: i64,
) -> Result<
    Vec<(
        (Uuid, Option<String>, Verification, DateTime<Utc>),
        (Uuid, Option<String>, String),
        (Uuid, String),
    )>,
> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting connection from pool: {}", e);
            return Err(anyhow!("Error getting connection: {}", e));
        }
    };

    let mut messages_statement = threads::table
        .inner_join(
            messages::table.on(messages::id
                .nullable()
                .eq(threads::state_message_id.nullable())
                .and(messages::deleted_at.is_null())),
        )
        .inner_join(datasets::table.on(datasets::id.nullable().eq(messages::dataset_id)))
        .inner_join(
            asset_permissions::table.on(threads::id
                .eq(asset_permissions::asset_id)
                .and(asset_permissions::asset_type.eq(AssetType::Thread))
                .and(asset_permissions::deleted_at.is_null())),
        )
        .left_join(
            teams_to_users::table.on(asset_permissions::identity_id
                .eq(teams_to_users::team_id)
                .and(asset_permissions::identity_type.eq(IdentityType::Team))
                .and(asset_permissions::deleted_at.is_null())),
        )
        .inner_join(users::table.on(users::id.eq(threads::created_by)))
        .select((
            (
                messages::thread_id,
                messages::title,
                messages::verification,
                messages::created_at,
            ),
            (users::id, users::name.nullable(), users::email),
            (datasets::id, datasets::name),
        ))
        .filter(
            asset_permissions::identity_id
                .eq(user_id)
                .or(teams_to_users::user_id.eq(user_id)),
        )
        .filter(threads::deleted_at.is_null())
        .filter(messages::deleted_at.is_null())
        .filter(messages::draft_session_id.is_null())
        .filter(messages::dataset_id.is_not_null())
        .filter(messages::code.is_not_null())
        .order_by(messages::created_at.desc())
        .offset(page * page_size)
        .limit(page_size)
        .into_boxed();

    if let Some(filters) = filters {
        if let Some(verification) = filters.verification {
            messages_statement =
                messages_statement.filter(messages::verification.eq_any(verification));
        }
    }

    let messages = match messages_statement
        .load::<(
            (Uuid, Option<String>, Verification, DateTime<Utc>),
            (Uuid, Option<String>, String),
            (Uuid, String),
        )>(&mut conn)
        .await
    {
        Ok(messages) => messages,
        Err(e) => {
            tracing::error!("Error getting messages: {}", e);
            return Err(anyhow!("Error getting messages: {}", e));
        }
    };

    Ok(messages)
}

async fn get_admin_threads(
    user_id: &Uuid,
    filters: Option<ListThreadsFilters>,
    page: i64,
    page_size: i64,
) -> Result<
    Vec<(
        (Uuid, Option<String>, Verification, DateTime<Utc>),
        (Uuid, Option<String>, String),
        (Uuid, String),
    )>,
> {
    let organization_id = get_user_organization_id(user_id).await?;

    let mut messages_statement = threads::table
        .inner_join(
            messages::table.on(messages::id
                .nullable()
                .eq(threads::state_message_id.nullable())),
        )
        .inner_join(datasets::table.on(datasets::id.nullable().eq(messages::dataset_id)))
        .inner_join(users::table.on(users::id.eq(threads::created_by)))
        .select((
            (
                messages::thread_id,
                messages::title,
                messages::verification,
                messages::created_at,
            ),
            (users::id, users::name.nullable(), users::email),
            (datasets::id, datasets::name),
        ))
        .filter(threads::organization_id.eq(organization_id))
        .filter(threads::deleted_at.is_null())
        .filter(messages::deleted_at.is_null())
        .filter(messages::draft_session_id.is_null())
        .filter(messages::dataset_id.is_not_null())
        .filter(messages::code.is_not_null())
        .order_by(messages::created_at.desc())
        .offset(page * page_size)
        .limit(page_size)
        .into_boxed();

    if let Some(filters) = filters {
        if let Some(verification) = filters.verification {
            messages_statement =
                messages_statement.filter(messages::verification.eq_any(verification));
        }
    }

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting connection from pool: {}", e);
            return Err(anyhow!("Error getting connection: {}", e));
        }
    };

    let messages = match messages_statement
        .load::<(
            (Uuid, Option<String>, Verification, DateTime<Utc>),
            (Uuid, Option<String>, String),
            (Uuid, String),
        )>(&mut conn)
        .await
    {
        Ok(messages) => messages,
        Err(e) => {
            tracing::error!("Error getting messages: {}", e);
            return Err(anyhow!("Error getting messages: {}", e));
        }
    };

    Ok(messages)
}
