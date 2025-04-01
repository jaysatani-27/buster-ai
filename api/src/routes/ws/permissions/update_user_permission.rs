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
        schema::{permission_groups_to_identities, teams_to_users, users, users_to_organizations},
    },
    routes::ws::{
        permissions::permissions_router::{PermissionEvent, PermissionRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{clients::sentry_utils::send_sentry_error, user::user_info::get_user_organization_id},
};

use super::permissions_utils::{get_user_permission_group_state, UserPermissionGroupState};

#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = users_to_organizations)]
pub struct UserPermissionUpdateBody {
    pub sharing_setting: Option<SharingSetting>,
    pub edit_sql: Option<bool>,
    pub upload_csv: Option<bool>,
    pub export_assets: Option<bool>,
    pub email_slack_enabled: Option<bool>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAndRole {
    pub id: Uuid,
    pub role: TeamToUserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPermissionRequest {
    pub id: Uuid,
    pub name: Option<String>,
    #[serde(flatten)]
    pub team_permission_update_body: Option<UserPermissionUpdateBody>,
    pub permission_groups: Option<Vec<Uuid>>,
    pub teams: Option<Vec<TeamAndRole>>,
}

pub async fn update_user_permission(user: &User, req: UpdateUserPermissionRequest) -> Result<()> {
    let user_permission_state = match update_user_permission_handler(
        &user.id,
        &req.id,
        req.name,
        req.team_permission_update_body,
        req.permission_groups,
        req.teams,
    )
    .await
    {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error updating user permission: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::UpdateUserPermission),
                WsEvent::Permissions(PermissionEvent::UpdateUserPermission),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let update_permission_group_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::UpdateUserPermission),
        WsEvent::Permissions(PermissionEvent::UpdateUserPermission),
        user_permission_state,
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

async fn update_user_permission_handler(
    created_by: &Uuid,
    user_id: &Uuid,
    name: Option<String>,
    update_body: Option<UserPermissionUpdateBody>,
    permission_groups: Option<Vec<Uuid>>,
    teams: Option<Vec<TeamAndRole>>,
) -> Result<UserPermissionGroupState> {
    let user_organization_id = match get_user_organization_id(created_by).await {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let created_by = Arc::new(created_by.clone());
    let user_id = Arc::new(user_id.clone());
    let user_organization_id = Arc::new(user_organization_id.clone());

    let update_user_record = if name.is_some() {
        let user_id = user_id.clone();
        let name = name.clone();
        Some(tokio::spawn(async move {
            update_user_record(user_id, name).await
        }))
    } else {
        None
    };

    let update_user_permission = {
        let user_id = user_id.clone();
        let user_organization_id = user_organization_id.clone();
        tokio::spawn(async move {
            if let Err(e) = update_user_data(user_id, user_organization_id, update_body).await {
                tracing::error!("Error in update_team_permission_data: {}", e);
            }
        })
    };

    let update_teams = {
        let user_id = user_id.clone();
        tokio::spawn(async move {
            if let Err(e) = update_team_users(user_id, teams).await {
                tracing::error!("Error in update_team_permission_identities: {}", e);
            }
        })
    };

    let update_permission_groups = {
        let user_id = user_id.clone();
        let permission_groups = permission_groups.clone();
        tokio::spawn(async move {
            if let Err(e) = update_permission_groups(created_by, user_id, permission_groups).await {
                tracing::error!("Error in update_permission_groups: {}", e);
            }
        })
    };

    let (_, _, _, _) = tokio::try_join!(
        update_user_permission,
        update_teams,
        update_permission_groups,
        update_user_record.unwrap_or(tokio::spawn(async { Ok(()) }))
    )?;

    let user_permission_state = match get_user_permission_group_state(&user_id).await {
        Ok(state) => state,
        Err(e) => return Err(e),
    };

    Ok(user_permission_state)
}

async fn update_user_data(
    user_id: Arc<Uuid>,
    organization_id: Arc<Uuid>,
    update_body: Option<UserPermissionUpdateBody>,
) -> Result<()> {
    if let Some(mut permission_group_changeset) = update_body {
        permission_group_changeset.updated_at = Some(Utc::now());

        let mut conn = get_pg_pool().get().await?;

        match diesel::update(users_to_organizations::table)
            .filter(users_to_organizations::user_id.eq(user_id.as_ref()))
            .filter(users_to_organizations::organization_id.eq(organization_id.as_ref()))
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

async fn update_team_users(user_id: Arc<Uuid>, teams: Option<Vec<TeamAndRole>>) -> Result<()> {
    if let Some(teams) = teams {
        let teams = Arc::new(teams);

        let add_identities_handle = {
            let user_id = user_id.clone();
            let teams = teams.clone();
            tokio::spawn(async move { add_user_to_teams(user_id, teams).await })
        };

        let remove_identities_handle = {
            let user_id = user_id.clone();
            let teams = teams.clone();
            tokio::spawn(async move { remove_teams_from_user(user_id, teams).await })
        };

        let (_, _) = tokio::try_join!(add_identities_handle, remove_identities_handle)?;
    }

    Ok(())
}

async fn add_user_to_teams(user_id: Arc<Uuid>, teams: Arc<Vec<TeamAndRole>>) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    let user_insert_body = teams
        .iter()
        .map(|team| TeamToUser {
            team_id: team.id.clone(),
            user_id: *user_id,
            role: team.role,
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

async fn remove_teams_from_user(user_id: Arc<Uuid>, teams: Arc<Vec<TeamAndRole>>) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    let team_ids = teams.iter().map(|team| team.id).collect::<Vec<Uuid>>();

    match update(teams_to_users::table)
        .set(teams_to_users::deleted_at.eq(Some(Utc::now())))
        .filter(not(teams_to_users::team_id.eq_any(&team_ids)))
        .filter(teams_to_users::user_id.eq(*user_id))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error removing users from team: {}", e)),
    }

    Ok(())
}

async fn update_permission_groups(
    created_by: Arc<Uuid>,
    user_id: Arc<Uuid>,
    permission_groups: Option<Vec<Uuid>>,
) -> Result<()> {
    if let Some(permission_groups) = permission_groups {
        let permission_groups = Arc::new(permission_groups);

        let add_permission_groups_handle = {
            let created_by = created_by.clone();
            let user_id = user_id.clone();
            let permission_groups = permission_groups.clone();
            tokio::spawn(async move {
                add_permission_groups(created_by, user_id, permission_groups).await
            })
        };

        let remove_permission_groups_handle = {
            let user_id = user_id.clone();
            let permission_groups = permission_groups.clone();
            tokio::spawn(async move { remove_permission_groups(user_id, permission_groups).await })
        };

        let (_, _) = tokio::try_join!(
            add_permission_groups_handle,
            remove_permission_groups_handle
        )?;
    }

    Ok(())
}

async fn add_permission_groups(
    created_by: Arc<Uuid>,
    user_id: Arc<Uuid>,
    permission_groups: Arc<Vec<Uuid>>,
) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    let permission_groups_to_identities = permission_groups
        .iter()
        .map(|permission_group| PermissionGroupToIdentity {
            permission_group_id: *permission_group,
            identity_id: *user_id,
            identity_type: IdentityType::User,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            created_by: *created_by,
            updated_by: *created_by,
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
            permission_groups_to_identities::created_by.eq(created_by.as_ref()),
            permission_groups_to_identities::updated_by.eq(created_by.as_ref()),
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
    user_id: Arc<Uuid>,
    permission_groups: Arc<Vec<Uuid>>,
) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    match update(permission_groups_to_identities::table)
        .set(permission_groups_to_identities::deleted_at.eq(Some(Utc::now())))
        .filter(permission_groups_to_identities::identity_id.eq(user_id.as_ref()))
        .filter(permission_groups_to_identities::identity_type.eq(IdentityType::User))
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

async fn update_user_record(user_id: Arc<Uuid>, name: Option<String>) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    match update(users::table)
        .set(users::name.eq(name))
        .filter(users::id.eq(user_id.as_ref()))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error updating user name: {}", e)),
    }

    Ok(())
}
