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
        schema::teams,
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
pub struct DeleteTeamsRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTeamsResponse {
    pub ids: Vec<Uuid>,
}

pub async fn delete_team_permission(user: &User, req: DeleteTeamsRequest) -> Result<()> {
    let deleted_ids = match delete_teams_handler(&req.ids).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("Error deleting teams: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::DeleteTeamPermission),
                WsEvent::Permissions(PermissionEvent::DeleteTeamPermission),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let delete_teams_response = DeleteTeamsResponse { ids: deleted_ids };

    let delete_teams_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::DeleteTeamPermission),
        WsEvent::Permissions(PermissionEvent::DeleteTeamPermission),
        delete_teams_response,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &delete_teams_message).await {
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

async fn delete_teams_handler(ids: &Vec<Uuid>) -> Result<Vec<Uuid>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let deleted_count = update(teams::table)
        .set(teams::deleted_at.eq(Some(Utc::now())))
        .filter(teams::id.eq_any(ids))
        .execute(&mut conn)
        .await?;

    if deleted_count == 0 {
        return Err(anyhow!("No teams were deleted"));
    }

    Ok(ids.clone())
}
