use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{update, ExpressionMethods};
use diesel_async::RunQueryDsl;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::AssetPermissionRole,
        lib::get_pg_pool,
        models::User,
        schema::collections,
    },
    routes::ws::{
        ws::{WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::send_ws_message,
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::{
    collection_utils::get_bulk_user_collection_permission,
    collections_router::{CollectionEvent, CollectionRoute},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCollectionRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCollectionResponse {
    pub ids: Vec<Uuid>,
}

pub async fn delete_collection(user: &User, req: DeleteCollectionRequest) -> Result<()> {
    let response = match delete_collection_handler(&user, req.ids).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Error deleting collection: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    };

    let post_collection_message = WsResponseMessage::new(
        WsRoutes::Collections(CollectionRoute::Delete),
        WsEvent::Collections(CollectionEvent::DeleteCollections),
        response,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &post_collection_message).await {
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

async fn delete_collection_handler(
    user: &User,
    ids: Vec<Uuid>,
) -> Result<DeleteCollectionResponse> {
    let roles = match get_bulk_user_collection_permission(&user.id, &ids).await {
        Ok(dashboard) => dashboard,
        Err(e) => return Err(anyhow!("Error getting dashboard: {:?}", e)),
    };

    let filtered_ids_to_delete: Vec<Uuid> = ids
        .into_iter()
        .filter(|id| match roles.get(id) {
            Some(role) if *role != AssetPermissionRole::Viewer => true,
            _ => false,
        })
        .collect();

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting connection: {}", e));
        }
    };

    let _ = match update(collections::table)
        .filter(collections::id.eq_any(&filtered_ids_to_delete))
        .set(collections::deleted_at.eq(Some(Utc::now())))
        .execute(&mut conn)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow!("Error updating dashboard: {}", e));
        }
    };

    Ok(DeleteCollectionResponse {
        ids: filtered_ids_to_delete,
    })
}
