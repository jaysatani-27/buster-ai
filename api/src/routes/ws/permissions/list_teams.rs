use anyhow::{anyhow, Result};
use diesel::{
    dsl::sql, sql_types::BigInt,
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::{IdentityType, TeamToUserRole},
        lib::get_pg_pool,
        models::User,
        schema::{permission_groups_to_identities, teams, teams_to_users},
    },
    routes::ws::{
        permissions::permissions_router::{PermissionEvent, PermissionRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{clients::sentry_utils::send_sentry_error, user::user_info::get_user_organization_id},
};

#[derive(Deserialize, Debug, Clone)]
pub struct ListTeamsFilters {
    pub permission_group_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub belongs_to: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListTeamPermissionsRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(flatten)]
    pub filters: Option<ListTeamsFilters>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TeamPermissionInfo {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_group_count: Option<i64>,
    pub member_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belongs_to: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_role: Option<TeamToUserRole>,
}

pub async fn list_teams(user: &User, req: ListTeamPermissionsRequest) -> Result<()> {
    let page = req.page.unwrap_or(0);
    let page_size = req.page_size.unwrap_or(25);

    let team_permissions =
        match list_team_permissions_handler(user, page, page_size, req.filters).await {
            Ok(team_permissions) => team_permissions,
            Err(e) => {
                tracing::error!("Error listing team permissions: {}", e);
                send_sentry_error(&e.to_string(), Some(&user.id));
                send_error_message(
                    &user.id.to_string(),
                    WsRoutes::Permissions(PermissionRoute::ListTeamPermissions),
                    WsEvent::Permissions(PermissionEvent::ListTeamPermissions),
                    WsErrorCode::InternalServerError,
                    "Failed to list team permissions.".to_string(),
                    user,
                )
                .await?;
                return Err(e);
            }
        };

    let list_permission_groups_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::ListTeamPermissions),
        WsEvent::Permissions(PermissionEvent::ListTeamPermissions),
        team_permissions,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &list_permission_groups_message).await {
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

async fn list_team_permissions_handler(
    user: &User,
    page: i64,
    page_size: i64,
    filters: Option<ListTeamsFilters>,
) -> Result<Vec<TeamPermissionInfo>> {
    let organization_id = match get_user_organization_id(&user.id).await {
        Ok(organization_id) => organization_id,
        Err(e) => return Err(e),
    };

    let team_permissions = if let Some(filters) = filters {
        if let Some(permission_group_id) = filters.permission_group_id {
            query_permission_groups_teams(
                organization_id,
                permission_group_id,
                page,
                page_size,
                filters.belongs_to,
            )
            .await?
        } else if let Some(user_id) = filters.user_id {
            query_user_teams(
                &organization_id,
                &user_id,
                page,
                page_size,
                filters.belongs_to,
            )
            .await?
        } else {
            list_all_teams(page, page_size, organization_id).await?
        }
    } else {
        list_all_teams(page, page_size, organization_id).await?
    };

    Ok(team_permissions)
}

async fn list_all_teams(
    page: i64,
    page_size: i64,
    organization_id: Uuid,
) -> Result<Vec<TeamPermissionInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let team_permissions = match teams::table
        .left_join(teams_to_users::table.on(teams::id.eq(teams_to_users::team_id)))
        .left_join(
            permission_groups_to_identities::table
                .on(teams::id.eq(permission_groups_to_identities::identity_id)),
        )
        .select((
            teams::id,
            teams::name,
            sql::<BigInt>("count(distinct teams_to_users.user_id)"),
            sql::<BigInt>("count(distinct permission_groups_to_identities.permission_group_id)"),
        ))
        .filter(teams::organization_id.eq(&organization_id))
        .filter(teams::deleted_at.is_null())
        .filter(teams_to_users::deleted_at.is_null())
        .filter(permission_groups_to_identities::deleted_at.is_null())
        .group_by((teams::id, teams::name))
        .order(teams::name.asc())
        .limit(page_size)
        .offset(page * page_size)
        .load::<(Uuid, String, i64, i64)>(&mut conn)
        .await
    {
        Ok(team_permissions) => team_permissions,
        Err(e) => return Err(anyhow!("Error loading team permissions: {}", e)),
    };

    let team_permissions = team_permissions
        .into_iter()
        .map(
            |(id, name, member_count, permission_group_count)| TeamPermissionInfo {
                id,
                name,
                permission_group_count: Some(permission_group_count),
                member_count,
                belongs_to: None,
                team_role: None,
            },
        )
        .collect();

    Ok(team_permissions)
}

async fn query_permission_groups_teams(
    organization_id: Uuid,
    permission_group_id: Uuid,
    page: i64,
    page_size: i64,
    only_owned: Option<bool>,
) -> Result<Vec<TeamPermissionInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let permission_group_team_results = if let Some(_) = only_owned {
        teams::table
            .inner_join(
                permission_groups_to_identities::table.on(teams::id
                    .eq(permission_groups_to_identities::identity_id)
                    .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))
                    .and(permission_groups_to_identities::deleted_at.is_null())),
            )
            .inner_join(
                teams_to_users::table.on(teams::id
                    .eq(teams_to_users::team_id)
                    .and(teams_to_users::deleted_at.is_null())),
            )
            .select((
                teams::id,
                teams::name,
                permission_groups_to_identities::permission_group_id.nullable(),
                sql::<BigInt>("count(distinct teams_to_users.user_id)"),
            ))
            .group_by((
                teams::id,
                teams::name,
                permission_groups_to_identities::permission_group_id,
            ))
            .filter(permission_groups_to_identities::permission_group_id.eq(&permission_group_id))
            .filter(teams::organization_id.eq(organization_id))
            .filter(teams::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>, i64)>(&mut conn)
            .await
    } else {
        teams::table
            .left_join(
                permission_groups_to_identities::table.on(teams::id
                    .eq(permission_groups_to_identities::identity_id)
                    .and(
                        permission_groups_to_identities::identity_type
                            .eq(IdentityType::Team)
                            .and(permission_groups_to_identities::deleted_at.is_null())
                            .and(
                                permission_groups_to_identities::permission_group_id
                                    .eq(permission_group_id),
                            ),
                    )),
            )
            .inner_join(
                teams_to_users::table.on(teams::id
                    .eq(teams_to_users::team_id)
                    .and(teams_to_users::deleted_at.is_null())),
            )
            .select((
                teams::id,
                teams::name,
                permission_groups_to_identities::permission_group_id.nullable(),
                sql::<BigInt>("count(distinct teams_to_users.user_id)"),
            ))
            .group_by((
                teams::id,
                teams::name,
                permission_groups_to_identities::permission_group_id,
            ))
            .filter(teams::deleted_at.is_null())
            .filter(teams::organization_id.eq(organization_id))
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>, i64)>(&mut conn)
            .await
    };

    let permission_group_team_results: Vec<(Uuid, String, Option<Uuid>, i64)> =
        match permission_group_team_results {
            Ok(teams) => teams,
            Err(e) => return Err(anyhow!("Error getting teams: {}", e)),
        };

    let team_objects: Vec<TeamPermissionInfo> = permission_group_team_results
        .into_iter()
        .map(
            |(id, name, permission_group_id, member_count)| TeamPermissionInfo {
                id,
                name,
                permission_group_count: None,
                member_count,
                belongs_to: Some(permission_group_id.is_some()),
                team_role: None,
            },
        )
        .collect();

    Ok(team_objects)
}

async fn query_user_teams(
    organization_id: &Uuid,
    user_id: &Uuid, // this is the user being queried, not the user who made the req.
    page: i64,
    page_size: i64,
    only_owned: Option<bool>,
) -> Result<Vec<TeamPermissionInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let team_results = if let Some(true) = only_owned {
        teams::table
            .inner_join(teams_to_users::table)
            .select((
                teams::id,
                teams::name,
                teams_to_users::user_id.nullable(),
                teams_to_users::role.nullable(),
                sql::<BigInt>("count(distinct teams_to_users.user_id)"),
            ))
            .group_by((
                teams::id,
                teams::name,
                teams_to_users::user_id,
                teams_to_users::role,
            ))
            .filter(teams_to_users::user_id.eq(user_id))
            .filter(teams::organization_id.eq(organization_id))
            .filter(teams::deleted_at.is_null())
            .filter(teams_to_users::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>, Option<TeamToUserRole>, i64)>(&mut conn)
            .await
    } else {
        teams::table
            .left_join(
                teams_to_users::table.on(teams::id
                    .eq(teams_to_users::team_id)
                    .and(teams_to_users::user_id.eq(user_id))
                    .and(teams_to_users::deleted_at.is_null())),
            )
            .select((
                teams::id,
                teams::name,
                teams_to_users::user_id.nullable(),
                teams_to_users::role.nullable(),
                sql::<BigInt>("count(distinct teams_to_users.user_id)"),
            ))
            .group_by((
                teams::id,
                teams::name,
                teams_to_users::user_id,
                teams_to_users::role,
            ))
            .filter(teams::organization_id.eq(organization_id))
            .filter(teams::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>, Option<TeamToUserRole>, i64)>(&mut conn)
            .await
    };

    let team_results: Vec<(Uuid, String, Option<Uuid>, Option<TeamToUserRole>, i64)> =
        match team_results {
            Ok(teams) => teams,
            Err(e) => return Err(anyhow!("Error getting teams: {}", e)),
        };

    let team_objects = team_results
        .into_iter()
        .map(
            |(id, name, user_id, team_role, member_count)| TeamPermissionInfo {
                id,
                name,
                belongs_to: Some(user_id.is_some()),
                permission_group_count: None,
                member_count,
                team_role,
            },
        )
        .collect();

    Ok(team_objects)
}
