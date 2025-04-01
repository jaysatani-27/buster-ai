use anyhow::{anyhow, Result};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::models::User,
    routes::ws::{
        permissions::permissions_router::{PermissionEvent, PermissionRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::permissions_utils::{get_permission_group_state, PermissionGroupState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetPermissionGroupReq {
    pub id: Uuid,
}

pub async fn get_permission_group(user: &User, req: GetPermissionGroupReq) -> Result<()> {
    let user_permission_group_state = match get_permission_group_handler(&req.id).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error getting user permissions: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::GetPermissionGroup),
                WsEvent::Permissions(PermissionEvent::GetPermissionGroup),
                WsErrorCode::InternalServerError,
                "Failed to get permission group.".to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let get_user_permissions_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::GetPermissionGroup),
        WsEvent::Permissions(PermissionEvent::GetPermissionGroup),
        user_permission_group_state,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &get_user_permissions_message).await {
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

async fn get_permission_group_handler(perm_group_id: &Uuid) -> Result<PermissionGroupState> {
    get_permission_group_state(perm_group_id).await
}
