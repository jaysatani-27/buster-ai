use anyhow::{anyhow, Result};
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        lib::get_pg_pool,
        models::{PermissionGroup, User},
        schema::permission_groups,
    },
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{clients::sentry_utils::send_sentry_error, user::user_info::get_user_organization_id},
};

use super::{
    permissions_router::{PermissionEvent, PermissionRoute},
    permissions_utils::{get_permission_group_state, PermissionGroupState},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostPermissionGroupRequest {
    pub name: String,
}

pub async fn post_permission_group(user: &User, req: PostPermissionGroupRequest) -> Result<()> {
    let permission_group_state = match post_permission_group_handler(&user.id, req.name).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error creating permission group: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::PostPermissionGroup),
                WsEvent::Permissions(PermissionEvent::PostPermissionGroup),
                WsErrorCode::InternalServerError,
                "Failed to create permission group.".to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let post_permission_group_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::PostPermissionGroup),
        WsEvent::Permissions(PermissionEvent::PostPermissionGroup),
        permission_group_state,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &post_permission_group_message).await {
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

async fn post_permission_group_handler(
    user_id: &Uuid,
    name: String,
) -> Result<PermissionGroupState> {
    let user_organization_id = match get_user_organization_id(user_id).await {
        Ok(user_organization_id) => user_organization_id,
        Err(e) => return Err(anyhow!("Error getting user organization id: {}", e)),
    };

    let permission_group = PermissionGroup {
        id: Uuid::new_v4(),
        organization_id: user_organization_id,
        name,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: *user_id,
        updated_by: *user_id,
        deleted_at: None,
    };

    let mut conn = get_pg_pool().get().await?;

    match insert_into(permission_groups::table)
        .values(&permission_group)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting permission group: {}", e)),
    };

    let permission_group_state = match get_permission_group_state(&permission_group.id).await {
        Ok(permission_group_state) => permission_group_state,
        Err(e) => return Err(anyhow!("Error getting permission group state: {}", e)),
    };

    Ok(permission_group_state)
}
