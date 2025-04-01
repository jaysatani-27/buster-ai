use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        lib::get_pg_pool,
        models::User,
        schema::permission_groups,
    },
    routes::ws::{
        permissions::permissions_router::{PermissionEvent, PermissionRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePermissionGroupsRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePermissionGroupsResponse {
    pub ids: Vec<Uuid>,
}

pub async fn delete_permission_group(
    user: &User,
    req: DeletePermissionGroupsRequest,
) -> Result<()> {
    let deleted_ids = match delete_permission_groups_handler(&req.ids).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Error deleting permission groups: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::DeletePermissionGroup),
                WsEvent::Permissions(PermissionEvent::DeletePermissionGroup),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let delete_permission_groups_response = DeletePermissionGroupsResponse { ids: deleted_ids };

    let delete_permission_groups_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::DeletePermissionGroup),
        WsEvent::Permissions(PermissionEvent::DeletePermissionGroup),
        delete_permission_groups_response,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &delete_permission_groups_message).await {
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

async fn delete_permission_groups_handler(ids: &Vec<Uuid>) -> Result<Vec<Uuid>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let deleted_count = update(permission_groups::table)
        .set(permission_groups::deleted_at.eq(Some(Utc::now())))
        .filter(permission_groups::id.eq_any(ids))
        .execute(&mut conn)
        .await?;

    if deleted_count == 0 {
        return Err(anyhow!("No permission groups were deleted"));
    }

    Ok(ids.clone())
}
