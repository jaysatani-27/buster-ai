use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::{AssetPermissionRole, AssetType, IdentityType},
        lib::get_pg_pool,
        models::User,
        schema::{asset_permissions, collections, teams_to_users, users},
    },
    routes::ws::{
        collections::collections_router::{CollectionEvent, CollectionRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListCollectionsFilter {
    pub shared_with_me: Option<bool>,
    pub owned_by_me: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListCollectionsRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(flatten)]
    pub filters: Option<ListCollectionsFilter>,
}
pub async fn list_collections(user: &User, req: ListCollectionsRequest) -> Result<()> {
    let list_collections_res = match list_collections_handler(&user.id, req).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting collections: {}", e);
            let err = anyhow!("Error getting collections: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Collections(CollectionRoute::List),
                WsEvent::Collections(CollectionEvent::ListCollections),
                WsErrorCode::InternalServerError,
                "Failed to list collections.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let list_collections_message = WsResponseMessage::new(
        WsRoutes::Collections(CollectionRoute::List),
        WsEvent::Collections(CollectionEvent::ListCollections),
        list_collections_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &list_collections_message).await {
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

async fn list_collections_handler(
    user_id: &Uuid,
    req: ListCollectionsRequest,
) -> Result<Vec<ListCollectionsCollection>> {
    let page = req.page.unwrap_or(0);
    let page_size = req.page_size.unwrap_or(25);

    let list_of_datasets = match get_permissioned_collections(user_id, page, page_size, req).await {
        Ok(datasets) => datasets,
        Err(e) => return Err(e),
    };

    Ok(list_of_datasets)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListCollectionsUser {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListCollectionsCollection {
    pub id: Uuid,
    pub name: String,
    pub last_edited: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub owner: ListCollectionsUser,
    pub is_shared: bool,
}

async fn get_permissioned_collections(
    user_id: &Uuid,
    page: i64,
    page_size: i64,
    req: ListCollectionsRequest,
) -> Result<Vec<ListCollectionsCollection>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Unable to get connection from pool: {}", e)),
    };

    let mut collections_statement = collections::table
        .inner_join(
            asset_permissions::table.on(collections::id
                .eq(asset_permissions::asset_id)
                .and(asset_permissions::asset_type.eq(AssetType::Collection))
                .and(asset_permissions::deleted_at.is_null())),
        )
        .left_join(
            teams_to_users::table.on(asset_permissions::identity_id
                .eq(teams_to_users::user_id)
                .and(asset_permissions::identity_type.eq(IdentityType::Team))
                .and(teams_to_users::deleted_at.is_null())),
        )
        .inner_join(users::table.on(users::id.eq(collections::created_by)))
        .select((
            collections::id,
            collections::name,
            collections::updated_at,
            collections::created_at,
            asset_permissions::role,
            users::id,
            users::name.nullable(),
            users::email,
        ))
        .filter(collections::deleted_at.is_null())
        .filter(
            asset_permissions::identity_id
                .eq(user_id)
                .or(teams_to_users::user_id.eq(user_id)),
        )
        .distinct()
        .order((collections::updated_at.desc(), collections::id.asc()))
        .offset(page * page_size)
        .limit(page_size)
        .into_boxed();

    if let Some(filters) = req.filters {
        tracing::info!("Filters: {:?}", filters);
        if filters.shared_with_me.unwrap_or(false) {
            tracing::info!("Filtering for shared with me");
            collections_statement = collections_statement
                .filter(asset_permissions::role.ne(AssetPermissionRole::Owner));
        }

        if filters.owned_by_me.unwrap_or(false) {
            collections_statement = collections_statement
                .filter(asset_permissions::role.eq(AssetPermissionRole::Owner));
        }
    }

    let sql = diesel::debug_query::<diesel::pg::Pg, _>(&collections_statement).to_string();
    tracing::info!("SQL: {}", sql);
    tracing::info!("User ID: {}", user_id);
    
    let collection_results = match collections_statement
        .load::<(
            Uuid,
            String,
            DateTime<Utc>,
            DateTime<Utc>,
            AssetPermissionRole,
            Uuid,
            Option<String>,
            String,
        )>(&mut conn)
        .await
    {
        Ok(collection_results) => collection_results,
        Err(e) => return Err(anyhow!("Error getting collection results: {}", e)),
    };

    let mut collections: Vec<ListCollectionsCollection> = Vec::new();

    for (id, name, updated_at, created_at, role, user_id, user_name, email) in collection_results {
        let owner = ListCollectionsUser {
            id: user_id,
            name: user_name.unwrap_or(email),
            avatar_url: None,
        };

        let collection = ListCollectionsCollection {
            id,
            name,
            last_edited: updated_at,
            created_at,
            owner,
            is_shared: role != AssetPermissionRole::Owner,
        };

        collections.push(collection);
    }

    Ok(collections)
}
