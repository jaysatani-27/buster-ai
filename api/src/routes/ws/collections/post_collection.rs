use anyhow::{anyhow, Result};
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::{AssetPermissionRole, AssetType, IdentityType},
        lib::{get_pg_pool, get_sqlx_pool},
        models::{AssetPermission, Collection, User},
        schema::{asset_permissions, collections},
    },
    routes::ws::{
        ws::{SubscriptionRwLock, WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message, subscribe_to_stream},
    },
    utils::{clients::sentry_utils::send_sentry_error, user::user_info::get_user_organization_id},
};

use super::{
    collection_utils::CollectionState,
    collections_router::{CollectionEvent, CollectionRoute},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCollectionRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn post_collection(
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
    req: PostCollectionRequest,
) -> Result<()> {
    let collection_id = Uuid::new_v4();

    let collection_subscription = format!("collection:{}", collection_id);

    match subscribe_to_stream(
        subscriptions,
        &collection_subscription,
        user_group,
        &user.id,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error subscribing to stream: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    };

    let collection = match post_collection_handler(&user.id, &collection_id, req).await {
        Ok(collection) => collection,
        Err(e) => {
            tracing::error!("Error posting collection: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Collections(CollectionRoute::Post),
                WsEvent::Collections(CollectionEvent::CollectionState),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let post_collection_message = WsResponseMessage::new(
        WsRoutes::Collections(CollectionRoute::Post),
        WsEvent::Collections(CollectionEvent::CollectionState),
        collection,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&collection_subscription, &post_collection_message).await {
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

async fn post_collection_handler(
    user_id: &Uuid,
    collection_id: &Uuid,
    req: PostCollectionRequest,
) -> Result<CollectionState> {
    let organization_id = get_user_organization_id(user_id).await?;

    let collection = Collection {
        id: *collection_id,
        name: req.name,
        description: req.description,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: user_id.clone(),
        updated_by: user_id.clone(),
        deleted_at: None,
        organization_id,
    };

    let insert_task_user_id = user_id.clone();
    let insert_task_collection = collection.clone();

    let collection_insert = tokio::spawn(async move {
        let mut conn = match get_pg_pool().get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Error getting pg connection: {}", e);
                return;
            }
        };

        let asset_permissions = AssetPermission {
            identity_id: insert_task_user_id,
            identity_type: IdentityType::User,
            asset_id: insert_task_collection.id,
            asset_type: AssetType::Collection,
            role: AssetPermissionRole::Owner,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            created_by: insert_task_user_id,
            updated_by: insert_task_user_id,
        };

        match insert_into(collections::table)
            .values(&insert_task_collection)
            .execute(&mut conn)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error inserting collection: {}", e);
                send_sentry_error(&e.to_string(), Some(&insert_task_user_id));
            }
        };

        match insert_into(asset_permissions::table)
            .values(asset_permissions)
            .execute(&mut conn)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error inserting asset permissions: {}", e);
                send_sentry_error(&e.to_string(), Some(&insert_task_user_id));
            }
        }
    });

    let collection_id = collection.id.clone();
    let collection_name = collection.name.clone();
    let organization_id = collection.organization_id.clone();

    let collection_search_handle = tokio::spawn(async move {
        let mut conn = match get_pg_pool().get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Unable to get connection from pool: {:?}", e);
                send_sentry_error(&e.to_string(), None);
                return;
            }
        };

        let query = diesel::sql_query(
            "INSERT INTO asset_search (asset_id, asset_type, content, organization_id)
            VALUES ($1, 'collection', $2, $3)
            ON CONFLICT (asset_id, asset_type) 
            DO UPDATE SET
                content = EXCLUDED.content,
                updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(collection_id)
        .bind::<diesel::sql_types::Text, _>(collection_name)
        .bind::<diesel::sql_types::Uuid, _>(organization_id);

        if let Err(e) = query.execute(&mut conn).await {
            tracing::error!("Failed to update asset search: {:?}", e);
            send_sentry_error(&e.to_string(), None);
        }
    });

    if let Err(e) = collection_insert.await {
        return Err(anyhow!("Error in collection insert: {:?}", e));
    }

    if let Err(e) = collection_search_handle.await {
        return Err(anyhow!("Error in collection search insert: {:?}", e));
    }

    Ok(CollectionState {
        collection,
        assets: None,
        permission: AssetPermissionRole::Owner,
        individual_permissions: None,
        team_permissions: None,
        organization_permissions: false,
    })
}
