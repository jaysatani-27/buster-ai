use anyhow::{anyhow, Result};
use diesel::{
    alias,
    dsl::sql,
    sql_types::{Array, BigInt, Text, Uuid as DieselUuid},
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::{TeamToUserRole, UserOrganizationRole},
        lib::get_pg_pool,
        models::User,
        schema::{
            permission_groups_to_identities, teams, teams_to_users, users, users_to_organizations,
        },
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
pub struct ListUsersFilters {
    pub team_id: Option<Uuid>,
    pub permission_group_id: Option<Uuid>,
    pub belongs_to: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListUserPermissionsRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<ListUsersFilters>,
}

#[derive(Serialize, Debug, Clone)]
struct ListUserTeam {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Debug, Clone)]
struct ListUserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_group_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<UserOrganizationRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_role: Option<TeamToUserRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belongs_to: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams: Option<Vec<ListUserTeam>>,
}

pub async fn list_users(user: &User, req: ListUserPermissionsRequest) -> Result<()> {
    let user_permissions =
        match list_user_permissions_handler(user, req.page, req.page_size, req.filters).await {
            Ok(groups) => groups,
            Err(e) => {
                tracing::error!("Error listing permission groups: {}", e);
                send_sentry_error(&e.to_string(), Some(&user.id));
                send_error_message(
                    &user.id.to_string(),
                    WsRoutes::Permissions(PermissionRoute::ListPermissionGroups),
                    WsEvent::Permissions(PermissionEvent::ListPermissionGroups),
                    WsErrorCode::InternalServerError,
                    "Failed to list permission groups.".to_string(),
                    user,
                )
                .await?;
                return Err(e);
            }
        };

    let list_permission_groups_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::ListUserPermissions),
        WsEvent::Permissions(PermissionEvent::ListUserPermissions),
        user_permissions,
        None,
        user,
        WsSendMethod::SenderOnly,
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

async fn list_user_permissions_handler(
    user: &User,
    page: Option<i64>,
    page_size: Option<i64>,
    filters: Option<ListUsersFilters>,
) -> Result<Vec<ListUserInfo>> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(25);

    let organization_id = match get_user_organization_id(&user.id).await {
        Ok(organization_id) => organization_id,
        Err(e) => return Err(e),
    };

    if let Some(filters) = filters {
        if let Some(team_id) = filters.team_id {
            return list_team_users(
                organization_id,
                team_id,
                filters.belongs_to,
                page,
                page_size,
            )
            .await;
        } else if let Some(permission_group_id) = filters.permission_group_id {
            return list_permission_group_users(
                organization_id,
                permission_group_id,
                filters.belongs_to,
                page,
                page_size,
            )
            .await;
        } else {
            return list_all_users(organization_id, page, page_size).await;
        }
    } else {
        return list_all_users(organization_id, page, page_size).await;
    }
}

async fn list_all_users(
    organization_id: Uuid,
    page: i64,
    page_size: i64,
) -> Result<Vec<ListUserInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let team_pgi = alias!(permission_groups_to_identities as team_pgi);

    let user_permissions: Vec<(Uuid, Option<String>, String, i64, i64, UserOrganizationRole)> = match users::table
        .left_join(teams_to_users::table.on(users::id.eq(teams_to_users::user_id).and(teams_to_users::deleted_at.is_null())))
        .inner_join(users_to_organizations::table.on(users::id.eq(users_to_organizations::user_id).and(users_to_organizations::deleted_at.is_null())))
        .left_join(
            permission_groups_to_identities::table
                .on(users::id.eq(permission_groups_to_identities::identity_id).and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .left_join(
            team_pgi.on(teams_to_users::team_id.eq(team_pgi.fields(permission_groups_to_identities::identity_id)).and(team_pgi.fields(permission_groups_to_identities::deleted_at).is_null())),
        )
        .select((
            users::id,
            users::name.nullable(),
            users::email,
            sql::<BigInt>("count(distinct teams_to_users.team_id) filter (where teams_to_users.deleted_at is null)"),
            sql::<BigInt>("count(distinct coalesce(permission_groups_to_identities.permission_group_id, team_pgi.permission_group_id))"),
            users_to_organizations::role,
        ))
        .filter(users_to_organizations::organization_id.eq(&organization_id))
        .group_by((users::id, users::name, users::email, users_to_organizations::role))
        .order(users::name.asc())
        .limit(page_size)
        .offset(page * page_size)
        .load::<(Uuid, Option<String>, String, i64, i64, UserOrganizationRole)>(&mut conn)
        .await
    {
        Ok(user_permissions) => user_permissions,
        Err(e) => return Err(anyhow!("Error loading user permissions: {}", e)),
    };

    let user_permissions = user_permissions
        .into_iter()
        .map(
            |(id, name, email, team_count, permission_group_count, role)| ListUserInfo {
                id,
                name: name.unwrap_or(email.clone()),
                email,
                permission_group_count: Some(permission_group_count),
                team_count: Some(team_count),
                role: Some(role),
                team_role: None,
                belongs_to: None,
                teams: None,
            },
        )
        .collect();

    Ok(user_permissions)
}

async fn list_team_users(
    organization_id: Uuid,
    team_id: Uuid,
    belongs_to: Option<bool>,
    page: i64,
    page_size: i64,
) -> Result<Vec<ListUserInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let team_user_records_res = if let Some(_) = belongs_to {
        users::table
            .inner_join(
                teams_to_users::table.on(users::id
                    .eq(teams_to_users::user_id)
                    .and(teams_to_users::deleted_at.is_null())),
            )
            .select((
                users::id,
                users::name.nullable(),
                users::email,
                teams_to_users::role.nullable(),
            ))
            .filter(teams_to_users::team_id.eq(&team_id))
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, Option<String>, String, Option<TeamToUserRole>)>(&mut conn)
            .await
    } else {
        users::table
            .inner_join(
                users_to_organizations::table.on(users::id
                    .eq(users_to_organizations::user_id)
                    .and(users_to_organizations::organization_id.eq(organization_id))
                    .and(users_to_organizations::deleted_at.is_null())),
            )
            .left_join(
                teams_to_users::table.on(users::id
                    .eq(teams_to_users::user_id)
                    .and(teams_to_users::team_id.eq(&team_id))
                    .and(teams_to_users::deleted_at.is_null())),
            )
            .select((
                users::id,
                users::name.nullable(),
                users::email,
                teams_to_users::role.nullable(),
            ))
            .filter(teams_to_users::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, Option<String>, String, Option<TeamToUserRole>)>(&mut conn)
            .await
    };

    let team_user_records = match team_user_records_res {
        Ok(team_user_records) => team_user_records,
        Err(e) => return Err(anyhow!("Error loading team user records: {}", e)),
    };

    let team_users = team_user_records
        .into_iter()
        .map(|(user_id, name, email, role)| ListUserInfo {
            id: user_id,
            name: name.unwrap_or(email.clone()),
            email,
            permission_group_count: None,
            team_count: None,
            role: None,
            team_role: role,
            belongs_to: Some(role.is_some()),
            teams: None,
        })
        .collect();

    Ok(team_users)
}

async fn list_permission_group_users(
    organization_id: Uuid,
    permission_group_id: Uuid,
    belongs_to: Option<bool>,
    page: i64,
    page_size: i64,
) -> Result<Vec<ListUserInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let users_from_permission_group_res = if let Some(_) = belongs_to {
        users::table
            .inner_join(
                users_to_organizations::table.on(users::id
                    .eq(users_to_organizations::user_id)
                    .and(users_to_organizations::organization_id.eq(organization_id))
                    .and(users_to_organizations::deleted_at.is_null())),
            )
            .inner_join(
                permission_groups_to_identities::table.on(users::id
                    .eq(permission_groups_to_identities::identity_id)
                    .and(permission_groups_to_identities::deleted_at.is_null())
                    .and(
                        permission_groups_to_identities::permission_group_id
                            .eq(permission_group_id),
                    )),
            )
            .left_join(
                teams_to_users::table.on(permission_groups_to_identities::identity_id
                    .eq(teams_to_users::team_id)
                    .and(teams_to_users::deleted_at.is_null())
                    .and(permission_groups_to_identities::deleted_at.is_null())),
            )
            .left_join(
                teams::table.on(teams_to_users::team_id
                    .eq(teams::id)
                    .and(teams::deleted_at.is_null())),
            )
            .select((
                users::id,
                users::name.nullable(),
                users::email,
                permission_groups_to_identities::permission_group_id.nullable(),
                sql::<Array<DieselUuid>>("coalesce(array_agg(distinct teams.id) filter (where teams.id is not null), '{}') as team_ids"),
                sql::<Array<Text>>("coalesce(array_agg(distinct teams.name) filter (where teams.name is not null), '{}') as team_names"),
            ))
            .group_by((users::id, users::name, users::email, permission_groups_to_identities::permission_group_id))
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, Option<String>, String, Option<Uuid>, Vec<Uuid>, Vec<String>)>(&mut conn)
            .await
    } else {
        users::table
            .inner_join(
                users_to_organizations::table.on(users::id
                    .eq(users_to_organizations::user_id)
                    .and(users_to_organizations::organization_id.eq(organization_id))
                    .and(users_to_organizations::deleted_at.is_null())),
            )
            .left_join(
                permission_groups_to_identities::table.on(users::id
                    .eq(permission_groups_to_identities::identity_id)
                    .and(permission_groups_to_identities::deleted_at.is_null())
                    .and(
                        permission_groups_to_identities::permission_group_id
                            .eq(permission_group_id),
                    )),
            )
            .left_join(
                teams_to_users::table.on(permission_groups_to_identities::identity_id
                    .eq(teams_to_users::team_id)
                    .and(teams_to_users::deleted_at.is_null())
                    .and(permission_groups_to_identities::deleted_at.is_null())),
            )
            .left_join(
                teams::table.on(teams_to_users::team_id
                    .eq(teams::id)
                    .and(teams::deleted_at.is_null())),
            )
            .select((
                users::id,
                users::name.nullable(),
                users::email,
                permission_groups_to_identities::permission_group_id.nullable(),
                sql::<Array<DieselUuid>>("coalesce(array_agg(distinct teams.id) filter (where teams.id is not null), '{}') as team_ids"),
                sql::<Array<Text>>("coalesce(array_agg(distinct teams.name) filter (where teams.name is not null), '{}') as team_names"),
            ))
            .group_by((users::id, users::name, users::email, permission_groups_to_identities::permission_group_id))
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, Option<String>, String, Option<Uuid>, Vec<Uuid>, Vec<String>)>(&mut conn)
            .await
    };

    let users_from_permission_group = match users_from_permission_group_res {
        Ok(users_from_permission_group) => users_from_permission_group,
        Err(e) => return Err(anyhow!("Error loading users from permission group: {}", e)),
    };

    let user_permission_records = users_from_permission_group
        .into_iter()
        .map(
            |(user_id, name, email, permission_group_id, team_ids, team_names)| ListUserInfo {
                id: user_id,
                name: name.unwrap_or(email.clone()),
                email,
                permission_group_count: None,
                team_count: None,
                role: None,
                team_role: None,
                belongs_to: Some(permission_group_id.is_some()),
                teams: Some(
                    team_ids
                        .into_iter()
                        .zip(team_names.into_iter())
                        .map(|(id, name)| ListUserTeam { id, name })
                        .collect(),
                ),
            },
        )
        .collect();

    Ok(user_permission_records)
}
