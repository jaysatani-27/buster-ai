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

use super::permissions_utils::{get_user_permission_group_state, UserPermissionGroupState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetUserPermissionsReq {
    pub id: Uuid,
}

pub async fn get_user_permissions(user: &User, req: GetUserPermissionsReq) -> Result<()> {
    let user_permission_group_state = match get_user_permissions_handler(&req.id).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error getting user permissions: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::GetUserPermissions),
                WsEvent::Permissions(PermissionEvent::GetUserPermissions),
                WsErrorCode::InternalServerError,
                "Failed to get user permissions.".to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let get_user_permissions_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::GetUserPermissions),
        WsEvent::Permissions(PermissionEvent::GetUserPermissions),
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

async fn get_user_permissions_handler(user_id: &Uuid) -> Result<UserPermissionGroupState> {
    get_user_permission_group_state(user_id).await
}
