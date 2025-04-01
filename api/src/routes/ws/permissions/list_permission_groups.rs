use anyhow::{anyhow, Result};
use diesel::{
    dsl::sql,
    sql_types::{Array, BigInt, Text, Uuid as DieselUuid},
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::IdentityType,
        lib::get_pg_pool,
        models::User,
        schema::{
            datasets_to_permission_groups, permission_groups, permission_groups_to_identities,
            teams, teams_to_users, users,
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
pub struct ListPermissionGroupsFilter {
    pub user_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub belongs_to: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListPermissionGroupsRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(flatten)]
    pub filters: Option<ListPermissionGroupsFilter>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PermissionGroupTeam {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct PermissionGroupInfo {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams: Option<Vec<PermissionGroupTeam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belongs_to: Option<bool>,
}

pub async fn list_permission_groups(user: &User, req: ListPermissionGroupsRequest) -> Result<()> {
    let permission_groups =
        match list_permission_groups_handler(user, req.page, req.page_size, req.filters).await {
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
        WsRoutes::Permissions(PermissionRoute::ListPermissionGroups),
        WsEvent::Permissions(PermissionEvent::ListPermissionGroups),
        permission_groups,
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

async fn list_permission_groups_handler(
    user: &User,
    page: Option<i64>,
    page_size: Option<i64>,
    filters: Option<ListPermissionGroupsFilter>,
) -> Result<Vec<PermissionGroupInfo>> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(25);

    let organization_id = match get_user_organization_id(&user.id).await {
        Ok(organization_id) => organization_id,
        Err(e) => return Err(e),
    };

    if let Some(filters) = filters {
        if let Some(user_id) = filters.user_id {
            return list_user_permission_groups(
                organization_id,
                user_id,
                filters.belongs_to,
                page,
                page_size,
            )
            .await;
        } else if let Some(team_id) = filters.team_id {
            return list_team_permission_groups(team_id, filters.belongs_to, page, page_size).await;
        } else {
            return list_all_permission_groups(organization_id, page, page_size).await;
        }
    } else {
        return list_all_permission_groups(organization_id, page, page_size).await;
    };
}

async fn list_user_permission_groups(
    organization_id: Uuid,
    user_id: Uuid,
    belongs_to: Option<bool>,
    page: i64,
    page_size: i64,
) -> Result<Vec<PermissionGroupInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let users_from_permission_group_res = if let Some(_) = belongs_to {
        permission_groups::table
        .inner_join(
            permission_groups_to_identities::table.on(permission_groups::id
                .eq(permission_groups_to_identities::permission_group_id)
                .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .inner_join(
            datasets_to_permission_groups::table.on(permission_groups::id
                .eq(datasets_to_permission_groups::permission_group_id)
                .and(datasets_to_permission_groups::deleted_at.is_null())),
        )
        .inner_join(
            users::table.on(permission_groups_to_identities::identity_id
                .eq(users::id)
                .and(permission_groups_to_identities::identity_type.eq(IdentityType::User))
                .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .left_join(
            teams_to_users::table.on(permission_groups_to_identities::identity_id
                .eq(teams_to_users::team_id)
                .and(teams_to_users::deleted_at.is_null())
                .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))
                .and(teams_to_users::user_id.eq(user_id))),
        )
        .left_join(
            teams::table.on(teams_to_users::team_id
                .eq(teams::id)
                .and(teams::deleted_at.is_null())),
        )
        .filter(permission_groups::organization_id.eq(organization_id))
        .filter(permission_groups::deleted_at.is_null())
        .select((
            permission_groups::id,
            permission_groups::name,
            sql::<Array<DieselUuid>>("COALESCE(ARRAY_AGG(DISTINCT COALESCE(users.id, teams_to_users.user_id)) FILTER (WHERE COALESCE(users.id, teams_to_users.user_id) IS NOT NULL), '{}')"),
            sql::<Array<DieselUuid>>("COALESCE(ARRAY_AGG(DISTINCT teams.id) FILTER (WHERE teams.id IS NOT NULL), '{}')"),
            sql::<Array<Text>>("COALESCE(ARRAY_AGG(DISTINCT teams.name) FILTER (WHERE teams.name IS NOT NULL), '{}')"),
            sql::<BigInt>("count(distinct datasets_to_permission_groups.dataset_id)"),
        ))
        .group_by((
            permission_groups::id,
            permission_groups::name,
        ))
        .limit(page_size)
        .offset(page * page_size)
        .load::<(Uuid, String, Vec<Uuid>, Vec<Uuid>, Vec<String>, i64)>(&mut conn)
        .await
    } else {
        permission_groups::table
        .inner_join(
            datasets_to_permission_groups::table.on(permission_groups::id
                .eq(datasets_to_permission_groups::permission_group_id)
                .and(datasets_to_permission_groups::deleted_at.is_null())),
        )
        .left_join(
            permission_groups_to_identities::table.on(permission_groups::id
                .eq(permission_groups_to_identities::permission_group_id)
                .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .left_join(
            users::table.on(permission_groups_to_identities::identity_id
                .eq(users::id)
                .and(permission_groups_to_identities::identity_type.eq(IdentityType::User))
                .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .left_join(
            teams_to_users::table.on(permission_groups_to_identities::identity_id
                .eq(teams_to_users::team_id)
                .and(teams_to_users::deleted_at.is_null())
                .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))
                .and(teams_to_users::user_id.eq(user_id))),
        )
        .left_join(
            teams::table.on(teams_to_users::team_id
                .eq(teams::id)
                .and(teams::deleted_at.is_null())),
        )
        .filter(permission_groups::organization_id.eq(organization_id))
        .filter(permission_groups::deleted_at.is_null())
        .select((
            permission_groups::id,
            permission_groups::name,
            sql::<Array<DieselUuid>>("COALESCE(ARRAY_AGG(DISTINCT COALESCE(users.id, teams_to_users.user_id)) FILTER (WHERE COALESCE(users.id, teams_to_users.user_id) IS NOT NULL), '{}')"),
            sql::<Array<DieselUuid>>("COALESCE(ARRAY_AGG(DISTINCT teams.id) FILTER (WHERE teams.id IS NOT NULL), '{}')"),
            sql::<Array<Text>>("COALESCE(ARRAY_AGG(DISTINCT teams.name) FILTER (WHERE teams.name IS NOT NULL), '{}')"),
            sql::<BigInt>("count(distinct datasets_to_permission_groups.dataset_id)"),
        ))
        .group_by((
            permission_groups::id,
            permission_groups::name,
        ))
        .limit(page_size)
        .offset(page * page_size)
        .load::<(Uuid, String, Vec<Uuid>, Vec<Uuid>, Vec<String>, i64)>(&mut conn)
        .await
    };

    let users_from_permission_group = match users_from_permission_group_res {
        Ok(users_from_permission_group) => users_from_permission_group,
        Err(e) => return Err(anyhow!("Error loading users from permission group: {}", e)),
    };

    let user_permission_records = users_from_permission_group
        .into_iter()
        .map(
            |(id, name, identity_ids, team_ids, team_names, dataset_count)| PermissionGroupInfo {
                id,
                name,
                member_count: None,
                dataset_count: Some(dataset_count),
                team_count: None,
                teams: Some(
                    team_ids
                        .into_iter()
                        .zip(team_names.into_iter())
                        .map(|(id, name)| PermissionGroupTeam { id, name })
                        .collect(),
                ),
                belongs_to: Some(!identity_ids.is_empty()),
            },
        )
        .collect();

    Ok(user_permission_records)
}

async fn list_team_permission_groups(
    team_id: Uuid,
    belongs_to: Option<bool>,
    page: i64,
    page_size: i64,
) -> Result<Vec<PermissionGroupInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let permission_groups_res = if let Some(_) = belongs_to {
        permission_groups::table
            .inner_join(
                permission_groups_to_identities::table.on(permission_groups::id
                    .eq(permission_groups_to_identities::permission_group_id)
                    .and(permission_groups_to_identities::identity_id.eq(team_id))
                    .and(permission_groups_to_identities::deleted_at.is_null())
                    .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))),
            )
            .inner_join(
                datasets_to_permission_groups::table.on(permission_groups::id
                    .eq(datasets_to_permission_groups::permission_group_id)
                    .and(datasets_to_permission_groups::deleted_at.is_null())),
            )
            .select((
                permission_groups::id,
                permission_groups::name,
                permission_groups_to_identities::identity_id.nullable(),
                sql::<BigInt>("count(distinct datasets_to_permission_groups.dataset_id)"),
            ))
            .group_by((
                permission_groups::id,
                permission_groups::name,
                permission_groups_to_identities::identity_id,
            ))
            .filter(permission_groups::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>, i64)>(&mut conn)
            .await
    } else {
        permission_groups::table
            .left_join(
                permission_groups_to_identities::table.on(permission_groups::id
                    .eq(permission_groups_to_identities::permission_group_id)
                    .and(permission_groups_to_identities::identity_id.eq(team_id))
                    .and(permission_groups_to_identities::deleted_at.is_null())
                    .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))),
            )
            .inner_join(
                datasets_to_permission_groups::table.on(permission_groups::id
                    .eq(datasets_to_permission_groups::permission_group_id)
                    .and(datasets_to_permission_groups::deleted_at.is_null())),
            )
            .select((
                permission_groups::id,
                permission_groups::name,
                permission_groups_to_identities::identity_id.nullable(),
                sql::<BigInt>("count(distinct datasets_to_permission_groups.dataset_id)"),
            ))
            .group_by((
                permission_groups::id,
                permission_groups::name,
                permission_groups_to_identities::identity_id,
            ))
            .filter(permission_groups::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>, i64)>(&mut conn)
            .await
    };

    let permission_group_records = match permission_groups_res {
        Ok(permission_groups) => permission_groups,
        Err(e) => return Err(anyhow!("Error loading permission groups: {}", e)),
    };

    let permission_groups = permission_group_records
        .into_iter()
        .map(
            |(id, name, identity_id, dataset_count)| PermissionGroupInfo {
                id,
                name,
                dataset_count: Some(dataset_count),
                member_count: None,
                team_count: None,
                teams: None,
                belongs_to: Some(identity_id.is_some()),
            },
        )
        .collect();

    Ok(permission_groups)
}

async fn list_all_permission_groups(
    organization_id: Uuid,
    page: i64,
    page_size: i64,
) -> Result<Vec<PermissionGroupInfo>> {
    let mut conn = get_pg_pool().get().await?;

    let permission_groups: Vec<(Uuid, String, i64, i64, i64)> = match permission_groups::table
        .left_join(
            permission_groups_to_identities::table.on(permission_groups::id
                .eq(permission_groups_to_identities::permission_group_id)
                .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .left_join(
            teams_to_users::table.on(permission_groups_to_identities::identity_id
                .eq(teams_to_users::team_id)
                .and(teams_to_users::deleted_at.is_null())),
        )
        .left_join(
            users::table.on(permission_groups_to_identities::identity_id
                .eq(users::id)
                .and(users::id.ne(teams_to_users::user_id))),
        )
        .left_join(
            datasets_to_permission_groups::table.on(permission_groups::id
                .eq(datasets_to_permission_groups::permission_group_id)
                .and(datasets_to_permission_groups::deleted_at.is_null())),
        )
        .select((
            permission_groups::id,
            permission_groups::name,
            sql::<BigInt>("count(distinct coalesce(users.id, teams_to_users.user_id))"),
            sql::<BigInt>("count(distinct datasets_to_permission_groups.dataset_id)"),
            sql::<BigInt>("count(distinct teams_to_users.team_id)"),
        ))
        .group_by((permission_groups::id, permission_groups::name))
        .filter(permission_groups::organization_id.eq(&organization_id))
        .filter(permission_groups::deleted_at.is_null())
        .order(permission_groups::name.asc())
        .limit(page_size)
        .offset(page * page_size)
        .load::<(Uuid, String, i64, i64, i64)>(&mut conn)
        .await
    {
        Ok(permission_groups) => permission_groups,
        Err(e) => return Err(anyhow!("Error loading permission groups: {}", e)),
    };

    let permission_groups = permission_groups
        .into_iter()
        .map(
            |(id, name, member_count, dataset_count, team_count)| PermissionGroupInfo {
                id,
                name,
                member_count: Some(member_count),
                dataset_count: Some(dataset_count),
                team_count: Some(team_count),
                teams: None,
                belongs_to: None,
            },
        )
        .collect();

    Ok(permission_groups)
}
