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

use super::permissions_utils::{get_team_permission_group_state, TeamPermissionGroupState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTeamPermissionsReq {
    pub id: Uuid,
}

pub async fn get_team_permissions(user: &User, req: GetTeamPermissionsReq) -> Result<()> {
    let team_permission_group_state = match get_team_permissions_handler(&req.id).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error getting team permissions: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::GetTeamPermissions),
                WsEvent::Permissions(PermissionEvent::GetTeamPermissions),
                WsErrorCode::InternalServerError,
                "Failed to get team permissions.".to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let get_team_permissions_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::GetTeamPermissions),
        WsEvent::Permissions(PermissionEvent::GetTeamPermissions),
        team_permission_group_state,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &get_team_permissions_message).await {
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

async fn get_team_permissions_handler(team_id: &Uuid) -> Result<TeamPermissionGroupState> {
    get_team_permission_group_state(team_id).await
}
