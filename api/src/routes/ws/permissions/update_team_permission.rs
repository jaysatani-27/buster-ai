use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{dsl::not, insert_into, update, upsert::excluded, AsChangeset, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    database::{
        enums::{IdentityType, SharingSetting, TeamToUserRole},
        lib::get_pg_pool,
        models::{PermissionGroupToIdentity, TeamToUser, User},
        schema::{permission_groups_to_identities, teams, teams_to_users},
    },
    routes::ws::{
        permissions::permissions_router::{PermissionEvent, PermissionRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::permissions_utils::{get_team_permission_group_state, TeamPermissionGroupState};

#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = teams)]
pub struct TeamPermissionUpdateBody {
    pub name: Option<String>,
    pub sharing_setting: Option<SharingSetting>,
    pub edit_sql: Option<bool>,
    pub upload_csv: Option<bool>,
    pub export_assets: Option<bool>,
    pub email_slack_enabled: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamUser {
    pub id: Uuid,
    pub role: TeamToUserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTeamPermissionRequest {
    pub id: Uuid,
    #[serde(flatten)]
    pub team_permission_update_body: Option<TeamPermissionUpdateBody>,
    pub permission_groups: Option<Vec<Uuid>>,
    pub users: Option<Vec<TeamUser>>,
}

pub async fn update_team_permission(user: &User, req: UpdateTeamPermissionRequest) -> Result<()> {
    let team_permission_state = match update_team_permission_handler(
        &user.id,
        &req.id,
        req.team_permission_update_body,
        req.permission_groups,
        req.users,
    )
    .await
    {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error updating team permission: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::UpdateTeamPermission),
                WsEvent::Permissions(PermissionEvent::UpdateTeamPermission),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let update_permission_group_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::UpdateTeamPermission),
        WsEvent::Permissions(PermissionEvent::UpdateTeamPermission),
        team_permission_state,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &update_permission_group_message).await {
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

async fn update_team_permission_handler(
    user_id: &Uuid,
    id: &Uuid,
    update_body: Option<TeamPermissionUpdateBody>,
    permission_groups: Option<Vec<Uuid>>,
    users: Option<Vec<TeamUser>>,
) -> Result<TeamPermissionGroupState> {
    let user_id = Arc::new(user_id.clone());
    let team_permission_id = Arc::new(id.clone());

    let update_team_permission = {
        let team_permission_id = team_permission_id.clone();
        tokio::spawn(async move {
            if let Err(e) = update_team_permission_data(team_permission_id, update_body).await {
                tracing::error!("Error in update_team_permission_data: {}", e);
            }
        })
    };

    let update_identities = {
        let team_permission_id = team_permission_id.clone();
        tokio::spawn(async move {
            if let Err(e) = update_team_users(team_permission_id, users).await {
                tracing::error!("Error in update_team_permission_identities: {}", e);
            }
        })
    };

    let update_permission_groups = {
        let team_permission_id = team_permission_id.clone();
        let permission_groups = permission_groups.clone();
        tokio::spawn(async move {
            if let Err(e) =
                update_permission_groups(user_id, team_permission_id, permission_groups).await
            {
                tracing::error!("Error in update_permission_groups: {}", e);
            }
        })
    };

    let (_, _, _) = tokio::try_join!(
        update_team_permission,
        update_identities,
        update_permission_groups
    )?;

    let team_permission_state = match get_team_permission_group_state(&team_permission_id).await {
        Ok(state) => state,
        Err(e) => return Err(e),
    };

    Ok(team_permission_state)
}

async fn update_team_permission_data(
    team_permission_id: Arc<Uuid>,
    update_body: Option<TeamPermissionUpdateBody>,
) -> Result<()> {
    if let Some(mut permission_group_changeset) = update_body {
        permission_group_changeset.updated_at = Some(Utc::now());

        let mut conn = get_pg_pool().get().await?;

        match diesel::update(teams::table)
            .filter(teams::id.eq(*team_permission_id))
            .set(&permission_group_changeset)
            .execute(&mut conn)
            .await
        {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Error updating permission group: {}", e)),
        }
    }
    Ok(())
}

async fn update_team_users(
    team_permission_id: Arc<Uuid>,
    users: Option<Vec<TeamUser>>,
) -> Result<()> {
    if let Some(users) = users {
        let users = Arc::new(users);

        let add_identities_handle = {
            let team_permission_id = team_permission_id.clone();
            let users = users.clone();
            tokio::spawn(async move { add_users_to_team(team_permission_id, users).await })
        };

        let remove_identities_handle = {
            let team_permission_id = team_permission_id.clone();
            let users = users.clone();
            tokio::spawn(async move { remove_users_from_team(team_permission_id, users).await })
        };

        let (_, _) = tokio::try_join!(add_identities_handle, remove_identities_handle)?;
    }

    Ok(())
}

async fn add_users_to_team(team_id: Arc<Uuid>, users: Arc<Vec<TeamUser>>) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    let user_insert_body = users
        .iter()
        .map(|user| TeamToUser {
            team_id: *team_id,
            user_id: user.id.clone(),
            role: user.role,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        })
        .collect::<Vec<TeamToUser>>();

    match insert_into(teams_to_users::table)
        .values(&user_insert_body)
        .on_conflict((teams_to_users::user_id, teams_to_users::team_id))
        .do_update()
        .set((
            teams_to_users::role.eq(excluded(teams_to_users::role)),
            teams_to_users::updated_at.eq(Utc::now()),
            teams_to_users::deleted_at.eq(None::<DateTime<Utc>>),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error adding users to team: {}", e)),
    };

    Ok(())
}

async fn remove_users_from_team(team_id: Arc<Uuid>, users: Arc<Vec<TeamUser>>) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    let user_ids = users.iter().map(|user| user.id).collect::<Vec<Uuid>>();

    match update(teams_to_users::table)
        .set(teams_to_users::deleted_at.eq(Some(Utc::now())))
        .filter(not(teams_to_users::user_id.eq_any(&user_ids)))
        .filter(teams_to_users::team_id.eq(*team_id))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error removing users from team: {}", e)),
    }

    Ok(())
}

async fn update_permission_groups(
    user_id: Arc<Uuid>,
    team_permission_id: Arc<Uuid>,
    permission_groups: Option<Vec<Uuid>>,
) -> Result<()> {
    if let Some(permission_groups) = permission_groups {
        let permission_groups = Arc::new(permission_groups);

        let add_permission_groups_handle = {
            let user_id = user_id.clone();
            let team_permission_id = team_permission_id.clone();
            let permission_groups = permission_groups.clone();
            tokio::spawn(async move {
                add_permission_groups(user_id, team_permission_id, permission_groups).await
            })
        };

        let remove_permission_groups_handle = {
            let team_permission_id = team_permission_id.clone();
            let permission_groups = permission_groups.clone();
            tokio::spawn(async move {
                remove_permission_groups(team_permission_id, permission_groups).await
            })
        };

        let (_, _) = tokio::try_join!(
            add_permission_groups_handle,
            remove_permission_groups_handle
        )?;
    }

    Ok(())
}

async fn add_permission_groups(
    user_id: Arc<Uuid>,
    team_permission_id: Arc<Uuid>,
    permission_groups: Arc<Vec<Uuid>>,
) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    let permission_groups_to_identities = permission_groups
        .iter()
        .map(|permission_group| PermissionGroupToIdentity {
            permission_group_id: *permission_group,
            identity_id: *team_permission_id,
            identity_type: IdentityType::Team,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            created_by: *user_id,
            updated_by: *user_id,
        })
        .collect::<Vec<PermissionGroupToIdentity>>();

    match insert_into(permission_groups_to_identities::table)
        .values(&permission_groups_to_identities)
        .on_conflict((
            permission_groups_to_identities::permission_group_id,
            permission_groups_to_identities::identity_id,
            permission_groups_to_identities::identity_type,
        ))
        .do_update()
        .set((
            permission_groups_to_identities::updated_at.eq(Utc::now()),
            permission_groups_to_identities::deleted_at.eq(None::<DateTime<Utc>>),
            permission_groups_to_identities::created_by
                .eq(excluded(permission_groups_to_identities::created_by)),
            permission_groups_to_identities::updated_by
                .eq(excluded(permission_groups_to_identities::updated_by)),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error adding permission groups to team: {}", e)),
    };

    Ok(())
}

async fn remove_permission_groups(
    team_permission_id: Arc<Uuid>,
    permission_groups: Arc<Vec<Uuid>>,
) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    match update(permission_groups_to_identities::table)
        .set(permission_groups_to_identities::deleted_at.eq(Some(Utc::now())))
        .filter(permission_groups_to_identities::identity_id.eq(team_permission_id.as_ref()))
        .filter(permission_groups_to_identities::identity_type.eq(IdentityType::Team))
        .filter(not(
            permission_groups_to_identities::permission_group_id.eq_any(permission_groups.as_ref())
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error removing permission groups from team: {}", e)),
    }

    Ok(())
}
