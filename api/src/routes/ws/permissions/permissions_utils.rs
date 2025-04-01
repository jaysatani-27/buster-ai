use chrono::Utc;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use diesel::{
    alias,
    deserialize::QueryableByName,
    dsl::{count, not, sql},
    insert_into,
    pg::sql_types::Array,
    sql_types::{BigInt, Text, Uuid as SqlUuid},
    update, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    QueryDsl,
};

use crate::database::{
    enums::SharingSetting,
    lib::get_pg_pool,
    models::{PermissionGroupToIdentity, UserToOrganization},
    schema::{datasets, sql_types::IdentityTypeEnum},
};

use serde::Serialize;
use uuid::Uuid;

use crate::database::{
    enums::{TeamToUserRole, UserOrganizationRole},
    models::{PermissionGroup, Team, User},
    schema::{
        datasets_to_permission_groups, messages, permission_groups,
        permission_groups_to_identities, teams, teams_to_users, users, users_to_organizations,
    },
};

use crate::database::enums::IdentityType;

#[derive(Serialize, QueryableByName)]
pub struct PermissionGroupItem {
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Array<IdentityTypeEnum>)]
    pub identities: Vec<IdentityType>,
    #[diesel(sql_type = BigInt)]
    pub dataset_count: i64,
}

#[derive(Serialize)]
pub struct TeamPermissionGroupItem {
    pub id: Uuid,
    pub name: String,
    pub dataset_count: i64,
}

#[derive(Serialize)]
pub struct DatasetItem {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct UserPermissionGroupItem {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct UserTeamItem {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: TeamToUserRole,
}

#[derive(Serialize)]
pub struct TeamItem {
    pub id: Uuid,
    pub name: String,
    pub member_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_role: Option<TeamToUserRole>,
}

#[derive(Serialize)]
pub struct UserPermissionGroupState {
    #[serde(flatten)]
    pub user: User,
    pub role: UserOrganizationRole,
    pub sharing_setting: SharingSetting,
    pub edit_sql: bool,
    pub upload_csv: bool,
    pub export_assets: bool,
    pub email_slack_enabled: bool,
    pub dataset_count: i64,
    pub permission_group_count: i64,
    pub team_count: i64,
    pub permission_groups: Vec<PermissionGroupItem>,
    pub teams: Vec<TeamItem>,
    pub queries_last_30_days: i64,
}

#[derive(Serialize)]
pub struct CreatedByUser {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct TeamPermissionGroupState {
    #[serde(flatten)]
    pub team: Team,
    pub permission_group_count: i64,
    pub member_count: i64,
    pub permission_groups: Vec<TeamPermissionGroupItem>,
    pub users: Vec<UserTeamItem>,
    pub created_by: CreatedByUser,
}

#[derive(Serialize)]
pub struct PermissionGroupState {
    #[serde(flatten)]
    pub permission_group: PermissionGroup,
    pub dataset_count: i64,
    pub datasets: Vec<DatasetItem>,
    pub team_count: i64,
    pub teams: Vec<TeamItem>,
    pub member_count: i64,
    pub users: Vec<UserPermissionGroupItem>,
    pub created_by: CreatedByUser,
}

pub async fn get_permission_group_state(perm_group_id: &Uuid) -> Result<PermissionGroupState> {
    let perm_group_id = Arc::new(perm_group_id.clone());

    let permission_group_handle = {
        let perm_group_id = perm_group_id.clone();
        tokio::spawn(async move { get_permission_group_and_creator(perm_group_id).await })
    };

    let datasets_handle = {
        let perm_group_id = perm_group_id.clone();
        tokio::spawn(async move { get_permission_group_datasets(perm_group_id).await })
    };

    let teams_handle = {
        let perm_group_id = perm_group_id.clone();
        tokio::spawn(async move { get_permission_group_teams(perm_group_id).await })
    };

    let users_handle = {
        let perm_group_id = perm_group_id.clone();
        tokio::spawn(async move { get_permission_group_users(perm_group_id).await })
    };

    let (permission_group_res, datasets_res, teams_res, users_res) = match tokio::try_join!(
        permission_group_handle,
        datasets_handle,
        teams_handle,
        users_handle,
    ) {
        Ok((permission_group_res, datasets_res, teams_res, users_res)) => {
            (permission_group_res, datasets_res, teams_res, users_res)
        }
        Err(e) => return Err(anyhow!(e)),
    };

    let (permission_group, created_by) = match permission_group_res {
        Ok(permission_group) => permission_group,
        Err(e) => return Err(anyhow!(e)),
    };

    let datasets = match datasets_res {
        Ok(datasets) => datasets,
        Err(e) => return Err(anyhow!(e)),
    };

    let teams = match teams_res {
        Ok(teams) => teams,
        Err(e) => return Err(anyhow!(e)),
    };

    let users = match users_res {
        Ok(users) => users,
        Err(e) => return Err(anyhow!(e)),
    };

    Ok(PermissionGroupState {
        permission_group,
        dataset_count: datasets.len() as i64,
        datasets,
        team_count: teams.len() as i64,
        teams,
        member_count: users.len() as i64,
        users,
        created_by,
    })
}

async fn get_permission_group_and_creator(
    perm_group_id: Arc<Uuid>,
) -> Result<(PermissionGroup, CreatedByUser)> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let permission_group = match permission_groups::table
        .inner_join(users::table.on(permission_groups::created_by.eq(users::id)))
        .select((
            (
                permission_groups::id,
                permission_groups::name,
                permission_groups::organization_id,
                permission_groups::created_by,
                permission_groups::updated_by,
                permission_groups::created_at,
                permission_groups::updated_at,
                permission_groups::deleted_at,
            ),
            users::id,
            users::name.nullable(),
            users::email,
        ))
        .filter(permission_groups::id.eq(perm_group_id.as_ref()))
        .filter(permission_groups::deleted_at.is_null())
        .first::<(PermissionGroup, Uuid, Option<String>, String)>(&mut conn)
        .await
    {
        Ok((permission_group, created_by_id, created_by_name, created_by_email)) => (
            permission_group,
            CreatedByUser {
                id: created_by_id,
                name: created_by_name.unwrap_or(created_by_email),
            },
        ),
        Err(e) => return Err(anyhow!(e)),
    };

    Ok(permission_group)
}

async fn get_permission_group_teams(perm_group_id: Arc<Uuid>) -> Result<Vec<TeamItem>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let teams = match teams::table
        .inner_join(
            permission_groups_to_identities::table.on(teams::id
                .eq(permission_groups_to_identities::identity_id)
                .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))),
        )
        .left_join(teams_to_users::table.on(teams::id.eq(teams_to_users::team_id)))
        .select((
            teams::id,
            teams::name,
            sql::<BigInt>("count(teams_to_users.user_id)"),
        ))
        .group_by((teams::id, teams::name))
        .filter(permission_groups_to_identities::permission_group_id.eq(perm_group_id.as_ref()))
        .filter(permission_groups_to_identities::deleted_at.is_null())
        .filter(teams_to_users::deleted_at.is_null())
        .filter(teams::deleted_at.is_null())
        .load::<(Uuid, String, i64)>(&mut conn)
        .await
    {
        Ok(teams) => teams,
        Err(e) => return Err(anyhow!(e)),
    };

    let teams = teams
        .into_iter()
        .map(|(id, name, member_count)| TeamItem {
            id,
            name,
            member_count,
            team_role: None,
        })
        .collect();

    Ok(teams)
}

async fn get_permission_group_users(
    perm_group_id: Arc<Uuid>,
) -> Result<Vec<UserPermissionGroupItem>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let direct_permissions = alias!(permission_groups_to_identities as direct_permissions);

    let users =
        match users::table
            .left_join(
                direct_permissions.on(users::id
                    .eq(direct_permissions.field(permission_groups_to_identities::identity_id))
                    .and(
                        direct_permissions
                            .field(permission_groups_to_identities::identity_type)
                            .eq(IdentityType::User),
                    )
                    .and(
                        direct_permissions
                            .field(permission_groups_to_identities::permission_group_id)
                            .eq(*perm_group_id),
                    )
                    .and(
                        direct_permissions
                            .field(permission_groups_to_identities::deleted_at)
                            .is_null(),
                    )),
            )
            .left_join(teams_to_users::table.on(users::id.eq(teams_to_users::user_id)))
            .left_join(
                permission_groups_to_identities::table.on(teams_to_users::team_id
                    .eq(permission_groups_to_identities::identity_id)
                    .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))
                    .and(permission_groups_to_identities::permission_group_id.eq(*perm_group_id))
                    .and(permission_groups_to_identities::deleted_at.is_null())),
            )
            .select((users::id, users::email, users::name))
            .distinct()
            .filter(
                direct_permissions
                    .field(permission_groups_to_identities::permission_group_id)
                    .eq(perm_group_id.as_ref())
                    .or(permission_groups_to_identities::permission_group_id
                        .eq(perm_group_id.as_ref())),
            )
            .load::<(Uuid, String, Option<String>)>(&mut conn)
            .await
        {
            Ok(users) => users,
            Err(e) => return Err(anyhow!(e)),
        };
    let user_items = users
        .into_iter()
        .map(|(id, email, name)| UserPermissionGroupItem {
            id,
            email: email.clone(),
            name: name.unwrap_or(email),
        })
        .collect();

    Ok(user_items)
}

async fn get_permission_group_datasets(perm_group_id: Arc<Uuid>) -> Result<Vec<DatasetItem>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let datasets = match datasets::table
        .inner_join(
            datasets_to_permission_groups::table
                .on(datasets::id.eq(datasets_to_permission_groups::dataset_id)),
        )
        .select((datasets::id, datasets::name))
        .filter(datasets_to_permission_groups::permission_group_id.eq(perm_group_id.as_ref()))
        .filter(datasets::deleted_at.is_null())
        .filter(datasets_to_permission_groups::deleted_at.is_null())
        .load::<(Uuid, String)>(&mut conn)
        .await
    {
        Ok(datasets) => datasets,
        Err(e) => return Err(anyhow!(e)),
    };

    let datasets = datasets
        .into_iter()
        .map(|(id, name)| DatasetItem { id, name })
        .collect();

    Ok(datasets)
}

pub async fn get_user_permission_group_state(user_id: &Uuid) -> Result<UserPermissionGroupState> {
    let user_id = Arc::new(user_id.clone());

    let user_role_and_queries_handle = {
        let user_id = user_id.clone();
        tokio::spawn(async move { get_user_role_and_queries(user_id).await })
    };

    let user_permission_groups_handle = {
        let user_id = user_id.clone();
        tokio::spawn(async move { get_user_permission_groups(user_id).await })
    };

    let user_teams_handle = {
        let user_id = user_id.clone();
        tokio::spawn(async move { get_user_teams(user_id).await })
    };

    let (user_role_and_queries_res, user_permission_groups_res, user_teams_res) = match tokio::try_join!(
        user_role_and_queries_handle,
        user_permission_groups_handle,
        user_teams_handle
    ) {
        Ok((user_role_and_queries_res, user_permission_groups_res, user_teams_res)) => (
            user_role_and_queries_res,
            user_permission_groups_res,
            user_teams_res,
        ),
        Err(e) => return Err(anyhow!(e)),
    };

    let (user, user_to_organization, queries_last_30_days) = match user_role_and_queries_res {
        Ok(user_role_and_queries) => user_role_and_queries,
        Err(e) => return Err(anyhow!(e)),
    };

    let permission_groups = match user_permission_groups_res {
        Ok(user_permission_groups) => user_permission_groups,
        Err(e) => return Err(anyhow!(e)),
    };

    let teams = match user_teams_res {
        Ok(user_teams) => user_teams,
        Err(e) => return Err(anyhow!(e)),
    };

    let permission_group_count = permission_groups.len() as i64;

    let team_count = teams.len() as i64;

    let dataset_count = permission_groups
        .iter()
        .map(|pg| pg.dataset_count)
        .sum::<i64>();

    Ok(UserPermissionGroupState {
        user,
        role: user_to_organization.role,
        sharing_setting: user_to_organization.sharing_setting,
        edit_sql: user_to_organization.edit_sql,
        upload_csv: user_to_organization.upload_csv,
        export_assets: user_to_organization.export_assets,
        email_slack_enabled: user_to_organization.email_slack_enabled,
        queries_last_30_days,
        permission_groups,
        permission_group_count,
        team_count,
        dataset_count,
        teams,
    })
}

async fn get_user_role_and_queries(user_id: Arc<Uuid>) -> Result<(User, UserToOrganization, i64)> {
    let user_and_org_role_handle = {
        let user_id = user_id.clone();
        tokio::spawn(async move { get_user_and_org_role(user_id).await })
    };

    let queries_last_30_days_handle = {
        let user_id = user_id.clone();
        tokio::spawn(async move { get_user_queries_last_30_days(user_id).await })
    };

    let (user_and_org_role_res, queries_last_30_days_res) =
        match tokio::try_join!(user_and_org_role_handle, queries_last_30_days_handle) {
            Ok((user_and_org_role_res, queries_last_30_days_res)) => {
                (user_and_org_role_res, queries_last_30_days_res)
            }
            Err(e) => return Err(anyhow!(e)),
        };

    let (user, user_to_organization) = match user_and_org_role_res {
        Ok(user_and_org_role) => user_and_org_role,
        Err(e) => return Err(anyhow!(e)),
    };
    let queries_last_30_days = match queries_last_30_days_res {
        Ok(queries_last_30_days) => queries_last_30_days,
        Err(e) => return Err(anyhow!(e)),
    };

    Ok((user, user_to_organization, queries_last_30_days))
}

async fn get_user_and_org_role(user_id: Arc<Uuid>) -> Result<(User, UserToOrganization)> {
    let mut conn = get_pg_pool().get().await?;

    let user_with_org_role = match users::table
        .inner_join(users_to_organizations::table.on(users::id.eq(users_to_organizations::user_id)))
        .select((users::all_columns, users_to_organizations::all_columns))
        .filter(users::id.eq(user_id.as_ref()))
        .first::<(User, UserToOrganization)>(&mut conn)
        .await
    {
        Ok(user_with_org_role) => user_with_org_role,
        Err(diesel::NotFound) => return Err(anyhow!("User not found")),
        Err(e) => return Err(anyhow!(e)),
    };

    Ok(user_with_org_role)
}

async fn get_user_queries_last_30_days(user_id: Arc<Uuid>) -> Result<i64> {
    let mut conn = get_pg_pool().get().await?;

    let query_count = match messages::table
        .select(count(messages::id))
        .filter(messages::sent_by.eq(user_id.as_ref()))
        .filter(messages::created_at.ge(Utc::now() - chrono::Duration::days(30)))
        .first::<i64>(&mut conn)
        .await
    {
        Ok(query_count) => query_count,
        Err(diesel::NotFound) => 0,
        Err(e) => return Err(anyhow!(e)),
    };

    Ok(query_count)
}

async fn get_user_permission_groups(user_id: Arc<Uuid>) -> Result<Vec<PermissionGroupItem>> {
    let mut conn = get_pg_pool().get().await?;

    let permission_groups = match permission_groups::table
        .inner_join(
            permission_groups_to_identities::table
                .on(permission_groups::id
                    .eq(permission_groups_to_identities::permission_group_id)
                    .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .left_join(users::table.on(permission_groups_to_identities::identity_id.eq(users::id).and(permission_groups_to_identities::identity_type.eq(IdentityType::User))))
        .left_join(
            teams_to_users::table
                .on(permission_groups_to_identities::identity_id
                    .eq(teams_to_users::team_id)
                    .and(teams_to_users::deleted_at.is_null())),
        )
        .left_join(
            datasets_to_permission_groups::table
                .on(permission_groups::id.eq(datasets_to_permission_groups::permission_group_id)),
        )
        .group_by((
            permission_groups::id,
            permission_groups::name,
        ))
        .select((
            permission_groups::id,
            permission_groups::name,
            sql::<Array<IdentityTypeEnum>>("array_agg(DISTINCT permission_groups_to_identities.identity_type) as identity_types"),
            sql::<BigInt>("count(DISTINCT datasets_to_permission_groups.dataset_id) as dataset_count"),
        ))
        .filter(
            users::id
                .eq(user_id.as_ref())
                .or(teams_to_users::user_id.eq(user_id.as_ref())),
        )
        .filter(permission_groups::deleted_at.is_null())
        .load::<(Uuid, String, Vec<IdentityType>, i64)>(&mut conn)
        .await
    {
        Ok(permission_groups) => permission_groups,
        Err(diesel::NotFound) => vec![],
        Err(e) => return Err(anyhow!(e)),
    };

    let permission_groups = permission_groups
        .into_iter()
        .map(
            |(id, name, identities, dataset_count)| PermissionGroupItem {
                id,
                name,
                identities,
                dataset_count,
            },
        )
        .collect();

    Ok(permission_groups)
}

async fn get_user_teams(user_id: Arc<Uuid>) -> Result<Vec<TeamItem>> {
    let mut conn = get_pg_pool().get().await?;

    let teams = match teams::table
        .inner_join(
            teams_to_users::table.on(teams::id
                .eq(teams_to_users::team_id)
                .and(teams_to_users::deleted_at.is_null())),
        )
        .select((
            teams::id,
            teams::name,
            sql::<BigInt>("count(teams_to_users.user_id)"),
            teams_to_users::role,
        ))
        .group_by((teams::id, teams::name, teams_to_users::role))
        .filter(teams_to_users::user_id.eq(user_id.as_ref()))
        .filter(teams::deleted_at.is_null())
        .load::<(Uuid, String, i64, TeamToUserRole)>(&mut conn)
        .await
    {
        Ok(teams) => teams,
        Err(diesel::NotFound) => vec![],
        Err(e) => return Err(anyhow!(e)),
    };

    let teams = teams
        .into_iter()
        .map(|(id, name, member_count, team_role)| TeamItem {
            id,
            name,
            member_count,
            team_role: Some(team_role),
        })
        .collect();

    Ok(teams)
}

pub async fn get_team_permission_group_state(team_id: &Uuid) -> Result<TeamPermissionGroupState> {
    let team_id = Arc::new(team_id.clone());

    let team_and_creator_handle = {
        let team_id = team_id.clone();
        tokio::spawn(async move { get_team_and_creator(team_id).await })
    };

    let team_permission_groups_handle = {
        let team_id = team_id.clone();
        tokio::spawn(async move { get_team_permission_groups(team_id).await })
    };

    let team_users_handle = {
        let team_id = team_id.clone();
        tokio::spawn(async move { get_team_users(team_id).await })
    };

    let (team_and_creator_res, team_permission_groups_res, team_users_res) = match tokio::try_join!(
        team_and_creator_handle,
        team_permission_groups_handle,
        team_users_handle
    ) {
        Ok((team_and_creator_res, team_permission_groups_res, team_users_res)) => (
            team_and_creator_res,
            team_permission_groups_res,
            team_users_res,
        ),
        Err(e) => return Err(anyhow!(e)),
    };

    let (team, created_by) = match team_and_creator_res {
        Ok(team_and_creator) => team_and_creator,
        Err(e) => return Err(anyhow!(e)),
    };

    let permission_groups = match team_permission_groups_res {
        Ok(team_permission_groups) => team_permission_groups,
        Err(e) => return Err(anyhow!(e)),
    };

    let users = match team_users_res {
        Ok(team_users) => team_users,
        Err(e) => return Err(anyhow!(e)),
    };

    Ok(TeamPermissionGroupState {
        permission_group_count: permission_groups.len() as i64,
        member_count: users.len() as i64,
        team,
        created_by,
        permission_groups,
        users,
    })
}

async fn get_team_and_creator(team_id: Arc<Uuid>) -> Result<(Team, CreatedByUser)> {
    let mut conn = get_pg_pool().get().await?;

    let team = match teams::table
        .inner_join(users::table.on(teams::created_by.eq(users::id)))
        .select((
            (
                teams::id,
                teams::name,
                teams::organization_id,
                teams::sharing_setting,
                teams::edit_sql,
                teams::upload_csv,
                teams::export_assets,
                teams::email_slack_enabled,
                teams::created_by,
                teams::created_at,
                teams::updated_at,
                teams::deleted_at,
            ),
            users::id,
            users::name.nullable(),
            users::email,
        ))
        .filter(teams::id.eq(team_id.as_ref()))
        .filter(teams::deleted_at.is_null())
        .first::<(Team, Uuid, Option<String>, String)>(&mut conn)
        .await
    {
        Ok((team, created_by_id, created_by_name, created_by_email)) => (
            team,
            CreatedByUser {
                id: created_by_id,
                name: created_by_name.unwrap_or(created_by_email),
            },
        ),
        Err(e) => return Err(anyhow!(e)),
    };

    Ok(team)
}

async fn get_team_permission_groups(team_id: Arc<Uuid>) -> Result<Vec<TeamPermissionGroupItem>> {
    let mut conn = get_pg_pool().get().await?;

    let permission_groups = match permission_groups::table
        .inner_join(
            permission_groups_to_identities::table.on(permission_groups::id
                .eq(permission_groups_to_identities::permission_group_id)
                .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))
                .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .left_join(
            datasets_to_permission_groups::table
                .on(permission_groups::id.eq(datasets_to_permission_groups::permission_group_id)),
        )
        .select((
            permission_groups::id,
            permission_groups::name,
            sql::<BigInt>("count(datasets_to_permission_groups.dataset_id) as dataset_count"),
        ))
        .group_by((permission_groups::id, permission_groups::name))
        .filter(permission_groups_to_identities::identity_id.eq(team_id.as_ref()))
        .filter(permission_groups::deleted_at.is_null())
        .load::<(Uuid, String, i64)>(&mut conn)
        .await
    {
        Ok(permission_groups) => permission_groups,
        Err(e) => return Err(anyhow!(e)),
    };

    let permission_groups = permission_groups
        .into_iter()
        .map(|(id, name, dataset_count)| TeamPermissionGroupItem {
            id,
            name,
            dataset_count,
        })
        .collect();

    Ok(permission_groups)
}

async fn get_team_users(team_id: Arc<Uuid>) -> Result<Vec<UserTeamItem>> {
    let mut conn = get_pg_pool().get().await?;

    let users = match users::table
        .inner_join(
            teams_to_users::table.on(users::id
                .eq(teams_to_users::user_id)
                .and(teams_to_users::deleted_at.is_null())),
        )
        .select((
            users::id,
            users::email,
            users::name.nullable(),
            teams_to_users::role,
        ))
        .filter(teams_to_users::team_id.eq(team_id.as_ref()))
        .load::<(Uuid, String, Option<String>, TeamToUserRole)>(&mut conn)
        .await
    {
        Ok(users) => users,
        Err(e) => return Err(anyhow!(e)),
    };

    let users = users
        .into_iter()
        .map(|(id, email, name, role)| UserTeamItem {
            id,
            email: email.clone(),
            name: name.unwrap_or(email),
            role,
        })
        .collect();

    Ok(users)
}

pub async fn remove_identities_from_permission_group(
    permission_group_id: &Uuid,
    ids: Arc<Vec<Uuid>>,
    identity_type: IdentityType,
) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    match update(permission_groups_to_identities::table)
        .set((permission_groups_to_identities::deleted_at.eq(Some(Utc::now())),))
        .filter(not(
            permission_groups_to_identities::identity_id.eq_any(ids.as_ref())
        ))
        .filter(permission_groups_to_identities::permission_group_id.eq(permission_group_id))
        .filter(permission_groups_to_identities::identity_type.eq(identity_type))
        .execute(&mut conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(e)),
    }
}

pub async fn add_identities_to_permission_group(
    user_id: &Uuid,
    permission_group_id: &Uuid,
    identities: Vec<(IdentityType, Uuid)>,
) -> Result<()> {
    let permission_groups_to_identities: Vec<PermissionGroupToIdentity> = identities
        .iter()
        .map(|(identity_type, identity_id)| PermissionGroupToIdentity {
            permission_group_id: permission_group_id.clone(),
            identity_id: identity_id.clone(),
            identity_type: *identity_type,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            created_by: user_id.clone(),
            updated_by: user_id.clone(),
        })
        .collect();

    let mut conn = get_pg_pool().get().await?;

    match insert_into(permission_groups_to_identities::table)
        .values(permission_groups_to_identities)
        .on_conflict((
            permission_groups_to_identities::identity_id,
            permission_groups_to_identities::identity_type,
            permission_groups_to_identities::permission_group_id,
        ))
        .do_update()
        .set(permission_groups_to_identities::deleted_at.eq(None::<chrono::DateTime<Utc>>))
        .execute(&mut conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(e)),
    }
}
