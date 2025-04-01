use anyhow::{anyhow, Result};
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::SharingSetting,
        lib::get_pg_pool,
        models::{Team, User},
        schema::teams,
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
    permissions_utils::{get_team_permission_group_state, TeamPermissionGroupState},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTeamRequest {
    pub name: String,
}

pub async fn post_team(user: &User, req: PostTeamRequest) -> Result<()> {
    let team_state = match post_team_handler(&user.id, req.name).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error creating team: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::PostTeam),
                WsEvent::Permissions(PermissionEvent::PostTeam),
                WsErrorCode::InternalServerError,
                "Failed to create team.".to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let post_team_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::PostTeam),
        WsEvent::Permissions(PermissionEvent::PostTeam),
        team_state,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &post_team_message).await {
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

async fn post_team_handler(user_id: &Uuid, name: String) -> Result<TeamPermissionGroupState> {
    let user_organization_id = match get_user_organization_id(user_id).await {
        Ok(user_organization_id) => user_organization_id,
        Err(e) => return Err(anyhow!("Error getting user organization id: {}", e)),
    };

    let team = Team {
        id: Uuid::new_v4(),
        organization_id: user_organization_id,
        name,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: *user_id,
        deleted_at: None,
        sharing_setting: SharingSetting::Team,
        edit_sql: true,
        upload_csv: true,
        export_assets: true,
        email_slack_enabled: true,
    };

    let mut conn = get_pg_pool().get().await?;

    match insert_into(teams::table)
        .values(&team)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting permission group: {}", e)),
    };

    let team_permission_group_state = match get_team_permission_group_state(&team.id).await {
        Ok(team_permission_group_state) => team_permission_group_state,
        Err(e) => return Err(anyhow!("Error getting permission group state: {}", e)),
    };

    Ok(team_permission_group_state)
}
