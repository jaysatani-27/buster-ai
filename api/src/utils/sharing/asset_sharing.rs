use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{
    dsl::{count, sql},
    insert_into,
    sql_types::Bool,
    update,
    upsert::excluded,
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde_json::json;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::{AssetPermissionRole, AssetType, IdentityType},
        lib::get_pg_pool,
        models::{AssetPermission, CollectionToAsset, User},
        schema::{
            asset_permissions, collections, collections_to_assets, dashboards, messages,
            organizations, teams, teams_to_users, threads, user_favorites, users,
        },
    },
    utils::clients::{
        email::resend::{send_email, CollectionInvite, DashboardInvite, EmailType, ThreadInvite},
        sentry_utils::send_sentry_error,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareWithTeamsReqObject {
    team_id: Uuid,
    role: AssetPermissionRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ShareWithUsersReqObject {
    user_email: String,
    role: AssetPermissionRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPermissions {
    pub id: Uuid,
    pub name: Option<String>,
    pub role: AssetPermissionRole,
    pub user_permissions: Vec<IndividualPermission>,
}

#[derive(Debug)]
pub struct AssetSharingInfo {
    pub individual_permissions: Option<Vec<IndividualPermission>>,
    pub team_permissions: Option<Vec<TeamPermissions>>,
    pub organization_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualPermission {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: String,
    pub role: AssetPermissionRole,
}

pub async fn update_asset_permissions(
    user: Arc<User>,
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    team_permissions: Option<Vec<ShareWithTeamsReqObject>>,
    user_permissions: Option<Vec<ShareWithUsersReqObject>>,
    remove_teams: Option<Vec<Uuid>>,
    remove_users: Option<Vec<Uuid>>,
) -> Result<()> {
    let user_id = Arc::new(user.id);
    let team_permissions_handle = if let Some(team_permissions) = team_permissions {
        let asset_id = Arc::clone(&asset_id);
        let user_id = Arc::clone(&user_id);
        Some(tokio::spawn(async move {
            grant_team_access_to_asset(&team_permissions, asset_id, asset_type, user_id).await
        }))
    } else {
        None
    };

    let user_permissions_handle = if let Some(user_permissions) = user_permissions {
        let asset_id = Arc::clone(&asset_id);
        let user = Arc::clone(&user);
        Some(tokio::spawn(async move {
            grant_user_access_to_asset(&user_permissions, asset_id, asset_type, user).await
        }))
    } else {
        None
    };

    let remove_teams_handle = if let Some(remove_teams) = remove_teams {
        let asset_id = Arc::clone(&asset_id);
        let user_id = Arc::clone(&user_id);
        Some(tokio::spawn(async move {
            remove_team_access_to_asset(&remove_teams, asset_id, asset_type, user_id).await
        }))
    } else {
        None
    };

    let remove_users_handle = if let Some(remove_users) = remove_users {
        let asset_id = Arc::clone(&asset_id);
        let user_id = Arc::clone(&user_id);
        Some(tokio::spawn(async move {
            remove_user_access_to_asset(&remove_users, asset_id, asset_type, user_id).await
        }))
    } else {
        None
    };

    if let Some(team_permissions_handle) = team_permissions_handle {
        match team_permissions_handle.await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Error granting team access to collection: {}", e));
            }
        }
    };

    if let Some(user_permissions_handle) = user_permissions_handle {
        match user_permissions_handle.await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Error granting user access to collection: {}", e));
            }
        }
    };

    if let Some(remove_teams_handle) = remove_teams_handle {
        match remove_teams_handle.await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Error removing team access from collection: {}", e));
            }
        }
    };

    if let Some(remove_users_handle) = remove_users_handle {
        match remove_users_handle.await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Error removing user access from collection: {}", e));
            }
        }
    };

    Ok(())
}

async fn grant_team_access_to_asset(
    team_permissions: &Vec<ShareWithTeamsReqObject>,
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    user_id: Arc<Uuid>,
) -> Result<()> {
    let mut asset_permissions = Vec::new();

    for team_permission in team_permissions {
        let asset_permission = AssetPermission {
            identity_id: team_permission.team_id,
            identity_type: IdentityType::Team,
            asset_id: *asset_id,
            asset_type,
            role: team_permission.role,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            created_by: *user_id,
            updated_by: *user_id,
        };

        asset_permissions.push(asset_permission);
    }

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    match insert_into(asset_permissions::table)
        .values(asset_permissions)
        .on_conflict((
            asset_permissions::asset_id,
            asset_permissions::identity_id,
            asset_permissions::identity_type,
            asset_permissions::asset_type,
        ))
        .do_update()
        .set((
            asset_permissions::role.eq(excluded(asset_permissions::role)),
            asset_permissions::updated_at.eq(chrono::Utc::now()),
            asset_permissions::updated_by.eq(user_id.as_ref()),
            asset_permissions::deleted_at.eq(None::<chrono::DateTime<chrono::Utc>>),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!(
                "Unable to insert or update asset permissions: {}",
                e
            ))
        }
    };

    Ok(())
}

async fn remove_team_access_to_asset(
    team_ids: &Vec<Uuid>,
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    user_id: Arc<Uuid>,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let team_ids = match update(asset_permissions::table)
        .filter(asset_permissions::identity_id.eq_any(team_ids))
        .filter(asset_permissions::asset_id.eq(asset_id.as_ref()))
        .filter(asset_permissions::identity_type.eq(IdentityType::Team))
        .filter(asset_permissions::asset_type.eq(asset_type))
        .set((
            asset_permissions::deleted_at.eq(chrono::Utc::now()),
            asset_permissions::updated_by.eq(user_id.as_ref()),
        ))
        .returning(asset_permissions::identity_id)
        .get_results(&mut conn)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            return Err(anyhow!(
                "Unable to insert or update asset permissions: {}",
                e
            ))
        }
    };

    tokio::spawn(async move {
        match remove_from_users_on_teams_favorites(asset_id, asset_type, team_ids).await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Error removing asset from user favorites: {}", e));
            }
        };

        Ok(())
    });

    Ok(())
}

async fn remove_from_users_on_teams_favorites(
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    team_ids: Vec<Uuid>,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let user_ids = match teams_to_users::table
        .filter(teams_to_users::team_id.eq_any(team_ids))
        .select(teams_to_users::user_id)
        .load::<Uuid>(&mut conn)
        .await
    {
        Ok(ids) => ids,
        Err(e) => return Err(anyhow!("Error getting user IDs: {}", e)),
    };

    match update(user_favorites::table)
        .filter(user_favorites::asset_id.eq(asset_id.as_ref()))
        .filter(user_favorites::asset_type.eq(asset_type))
        .filter(user_favorites::user_id.eq_any(user_ids))
        .set(user_favorites::deleted_at.eq(chrono::Utc::now()))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error removing asset from user favorites: {}", e));
        }
    };

    Ok(())
}

async fn remove_user_access_to_asset(
    user_ids: &Vec<Uuid>,
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    user_id: Arc<Uuid>,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let updated_ids: Vec<Uuid> = match update(asset_permissions::table)
        .filter(asset_permissions::identity_id.eq_any(user_ids))
        .filter(asset_permissions::asset_id.eq(asset_id.as_ref()))
        .filter(asset_permissions::identity_type.eq(IdentityType::User))
        .filter(asset_permissions::asset_type.eq(asset_type))
        .set((
            asset_permissions::deleted_at.eq(chrono::Utc::now()),
            asset_permissions::updated_by.eq(user_id.as_ref()),
        ))
        .returning(asset_permissions::identity_id)
        .get_results(&mut conn)
        .await
    {
        Ok(ids) => ids,
        Err(e) => {
            return Err(anyhow!(
                "Unable to insert or update asset permissions: {}",
                e
            ))
        }
    };

    tokio::spawn(async move {
        match remove_from_user_favorites(asset_id, asset_type, updated_ids).await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Error removing asset from user favorites: {}", e));
            }
        }

        Ok(())
    });

    Ok(())
}

async fn remove_from_user_favorites(
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    user_ids: Vec<Uuid>,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    match update(user_favorites::table)
        .filter(user_favorites::asset_id.eq(asset_id.as_ref()))
        .filter(user_favorites::asset_type.eq(asset_type))
        .filter(user_favorites::user_id.eq_any(user_ids))
        .set(user_favorites::deleted_at.eq(chrono::Utc::now()))
        .execute(&mut conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Error removing asset from user favorites: {}", e)),
    }
}

async fn grant_user_access_to_asset(
    user_permissions: &Vec<ShareWithUsersReqObject>,
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    user: Arc<User>,
) -> Result<()> {
    let mut asset_permissions = Vec::new();

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let user_records: Vec<(Uuid, String, AssetPermissionRole)> = match users::table
        .filter(
            users::email.eq_any(
                user_permissions
                    .iter()
                    .map(|u| &u.user_email)
                    .collect::<Vec<&String>>(),
            ),
        )
        .select((users::id, users::email))
        .load::<(Uuid, String)>(&mut conn)
        .await
    {
        Ok(user_ids) => user_ids
            .into_iter()
            .map(|(id, email)| {
                let role = user_permissions
                    .iter()
                    .find(|u| u.user_email == email)
                    .map(|u| u.role)
                    .unwrap_or(AssetPermissionRole::Viewer);
                (id, email, role)
            })
            .collect(),
        Err(e) => {
            tracing::error!("Error getting user IDs: {}", e);
            return Err(anyhow!("Error getting user IDs: {}", e));
        }
    };

    let existing_users: HashSet<String> = user_records
        .iter()
        .map(|(_, email, _)| email.clone())
        .collect();
    let non_existing_users: HashSet<ShareWithUsersReqObject> = user_permissions
        .iter()
        .filter(|u| !existing_users.contains(&u.user_email))
        .cloned()
        .collect();

    for (user_id, _, role) in &user_records {
        let asset_permission = AssetPermission {
            identity_id: *user_id,
            identity_type: IdentityType::User,
            asset_id: *asset_id,
            asset_type,
            role: *role,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            created_by: user.id.clone(),
            updated_by: user.id.clone(),
        };

        asset_permissions.push(asset_permission);
    }

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let updated_records = match insert_into(asset_permissions::table)
        .values(&asset_permissions)
        .on_conflict((
            asset_permissions::asset_id,
            asset_permissions::identity_id,
            asset_permissions::identity_type,
            asset_permissions::asset_type,
        ))
        .do_update()
        .set((
            asset_permissions::role.eq(excluded(asset_permissions::role)),
            asset_permissions::updated_at.eq(chrono::Utc::now()),
            asset_permissions::updated_by.eq(user.id.clone()),
            asset_permissions::deleted_at.eq(None::<chrono::DateTime<chrono::Utc>>),
        ))
        .returning((
            asset_permissions::identity_id,
            asset_permissions::role,
            sql::<diesel::sql_types::Bool>("(xmax = 0) as inserted"),
        ))
        .get_results::<(Uuid, AssetPermissionRole, bool)>(&mut conn)
        .await
    {
        Ok(records) => records,
        Err(e) => {
            tracing::error!("Unable to insert or update asset permissions: {:?}", e);
            let err = anyhow!("Unable to insert or update asset permissions: {}", e);
            send_sentry_error(&e.to_string(), None);
            return Err(err);
        }
    };

    let updated_records: Vec<(Uuid, AssetPermissionRole)> = updated_records
        .into_iter()
        .filter(|(_, _, inserted)| !inserted)
        .map(|(id, role, _)| (id, role))
        .collect();

    // Create a HashSet of updated user IDs
    let updated_user_ids: HashSet<Uuid> = updated_records.iter().map(|(id, _)| *id).collect();

    // Filter out existing_users whose IDs are in updated_records
    let existing_users: HashSet<String> = existing_users
        .into_iter()
        .filter(|email| {
            user_records
                .iter()
                .find(|(id, user_email, _)| user_email == email && !updated_user_ids.contains(id))
                .is_some()
        })
        .collect();

    // Now `updated_records` contains only the records that were updated
    asset_notification_and_invites(
        non_existing_users,
        existing_users,
        asset_id,
        asset_type,
        user,
    )
    .await;

    Ok(())
}

async fn asset_notification_and_invites(
    non_existing_users: HashSet<ShareWithUsersReqObject>,
    existing_users: HashSet<String>,
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    user: Arc<User>,
) -> () {
    let asset_name = match get_asset_name(asset_id.clone(), asset_type).await {
        Ok(name) => name,
        Err(e) => {
            tracing::error!("Error getting asset name: {}", e);
            return;
        }
    };

    if !non_existing_users.is_empty() {
        let asset_name = asset_name.clone();
        let inviter_name = user.name.clone().unwrap_or(user.email.clone());
        let user_id = user.id.clone();
        let asset_id = asset_id.clone();
        let new_users = match create_new_users_and_add_permissions(
            non_existing_users,
            &asset_id,
            asset_type,
            user_id,
        )
        .await
        {
            Ok(new_users) => new_users,
            Err(e) => {
                tracing::error!("Error creating new users: {}", e);
                return;
            }
        };

        tokio::spawn(async move {
            let email_type =
                create_invitation_email_type(inviter_name, asset_name, *asset_id, asset_type, true);

            match send_email(new_users, email_type).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Error sending email: {}", e);
                    send_sentry_error(&e.to_string(), None);
                }
            }
        });
    };

    if !existing_users.is_empty() {
        let email_type = create_invitation_email_type(
            user.name.clone().unwrap_or(user.email.clone()),
            asset_name,
            *asset_id,
            asset_type,
            false,
        );
        tokio::spawn(async move {
            match send_email(existing_users, email_type).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Error sending email: {}", e);
                    send_sentry_error(&e.to_string(), None);
                }
            }
        });
    };
}

fn create_invitation_email_type(
    inviter_name: String,
    asset_name: String,
    asset_id: Uuid,
    asset_type: AssetType,
    new_user: bool,
) -> EmailType {
    let email_type = match asset_type {
        AssetType::Collection => EmailType::CollectionInvite(CollectionInvite {
            inviter_name,
            new_user,
            collection_name: asset_name,
            collection_id: asset_id,
        }),
        AssetType::Dashboard => EmailType::DashboardInvite(DashboardInvite {
            inviter_name,
            new_user,
            dashboard_name: asset_name,
            dashboard_id: asset_id,
        }),
        AssetType::Thread => EmailType::ThreadInvite(ThreadInvite {
            inviter_name,
            new_user,
            thread_name: asset_name,
            thread_id: asset_id,
        }),
    };

    email_type
}

async fn get_asset_name(asset_id: Arc<Uuid>, asset_type: AssetType) -> Result<String> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let name = match asset_type {
        AssetType::Collection => {
            match collections::table
                .filter(collections::id.eq(asset_id.as_ref()))
                .select(collections::name)
                .first::<String>(&mut conn)
                .await
            {
                Ok(name) => name,
                Err(e) => {
                    tracing::error!("Error getting asset name: {}", e);
                    return Err(anyhow!("Error getting asset name: {}", e));
                }
            }
        }
        AssetType::Dashboard => {
            match dashboards::table
                .filter(dashboards::id.eq(asset_id.as_ref()))
                .select(dashboards::name)
                .first::<String>(&mut conn)
                .await
            {
                Ok(name) => name,
                Err(e) => {
                    tracing::error!("Error getting asset name: {}", e);
                    return Err(anyhow!("Error getting asset name: {}", e));
                }
            }
        }
        AssetType::Thread => {
            match messages::table
                .inner_join(threads::table.on(messages::thread_id.eq(threads::id)))
                .filter(threads::id.eq(asset_id.as_ref()))
                .select(messages::title.nullable())
                .order(messages::created_at.desc())
                .first::<Option<String>>(&mut conn)
                .await
            {
                Ok(Some(name)) => name,
                Ok(None) => "a thread".to_string(),
                Err(e) => {
                    tracing::error!("Error getting asset name: {}", e);
                    return Err(anyhow!("Error getting asset name: {}", e));
                }
            }
        }
    };

    Ok(name)
}

async fn create_new_users_and_add_permissions(
    new_users: HashSet<ShareWithUsersReqObject>,
    collection_id: &Uuid,
    asset_type: AssetType,
    user_id: Uuid,
) -> Result<HashSet<String>> {
    let mut new_users_records = Vec::new();
    let mut new_permissions_records = Vec::new();

    for new_user in new_users {
        let new_user_id = Uuid::new_v4();

        let user = User {
            id: new_user_id,
            email: new_user.user_email.clone(),
            name: None,
            attributes: json!({"user_id": new_user_id.to_string(), "user_email": new_user.user_email}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            config: json!({}),
        };

        let permission = AssetPermission {
            identity_id: user.id,
            identity_type: IdentityType::User,
            asset_id: *collection_id,
            asset_type,
            role: new_user.role,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            created_by: user_id,
            updated_by: user_id,
        };

        new_users_records.push(user);
        new_permissions_records.push(permission);
    }

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    match insert_into(users::table)
        .values(&new_users_records)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error inserting new users: {}", e));
        }
    };

    match insert_into(asset_permissions::table)
        .values(new_permissions_records)
        .on_conflict((
            asset_permissions::identity_id,
            asset_permissions::identity_type,
            asset_permissions::asset_id,
            asset_permissions::asset_type,
        ))
        .do_update()
        .set((
            asset_permissions::role.eq(excluded(asset_permissions::role)),
            asset_permissions::updated_at.eq(chrono::Utc::now()),
            asset_permissions::updated_by.eq(user_id),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error inserting new permissions: {}", e));
        }
    };

    Ok(new_users_records.iter().map(|u| u.email.clone()).collect())
}

pub async fn get_asset_sharing_info(
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
) -> Result<AssetSharingInfo> {
    let individual_permissions_handle = {
        let id = Arc::clone(&asset_id);
        tokio::spawn(async move { get_individual_permissions(id, asset_type).await })
    };

    let team_permissions_handle = {
        let id = Arc::clone(&asset_id);
        tokio::spawn(async move { get_team_permissions(id, asset_type).await })
    };

    let org_permissions_handle = {
        let id = Arc::clone(&asset_id);
        tokio::spawn(async move { get_organization_permissions(id, asset_type).await })
    };

    let (individual_permissions_result, team_permissions_result, org_permissions_result) = match tokio::try_join!(
        individual_permissions_handle,
        team_permissions_handle,
        org_permissions_handle
    ) {
        Ok((individual_permissions, team_permissions, org_permissions)) => {
            (individual_permissions, team_permissions, org_permissions)
        }
        Err(e) => {
            return Err(anyhow!("Error getting thread sharing info: {}", e));
        }
    };

    let individual_permissions = match individual_permissions_result {
        Ok(individual_permissions) => individual_permissions,
        Err(e) => {
            return Err(anyhow!("Error getting individual permissions: {}", e));
        }
    };

    let team_permissions = match team_permissions_result {
        Ok(team_permissions) => team_permissions,
        Err(e) => {
            return Err(anyhow!("Error getting team permissions: {}", e));
        }
    };

    let org_permissions = match org_permissions_result {
        Ok(org_permissions) => org_permissions,
        Err(e) => {
            return Err(anyhow!("Error getting organization permissions: {}", e));
        }
    };

    Ok(AssetSharingInfo {
        individual_permissions,
        team_permissions,
        organization_permissions: org_permissions,
    })
}

async fn get_individual_permissions(
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
) -> Result<Option<Vec<IndividualPermission>>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let individual_records = match asset_permissions::table
        .inner_join(users::table.on(asset_permissions::identity_id.eq(users::id)))
        .select((
            users::id,
            users::name.nullable(),
            users::email,
            asset_permissions::role,
        ))
        .filter(asset_permissions::identity_type.eq(IdentityType::User))
        .filter(asset_permissions::asset_id.eq(asset_id.as_ref()))
        .filter(asset_permissions::asset_type.eq(asset_type))
        .filter(asset_permissions::deleted_at.is_null())
        .load::<(Uuid, Option<String>, String, AssetPermissionRole)>(&mut conn)
        .await
    {
        Ok(individual_records) => individual_records,
        Err(diesel::result::Error::NotFound) => return Ok(None),
        Err(e) => {
            return Err(anyhow!("Error querying individual permissions: {}", e));
        }
    };

    if individual_records.is_empty() {
        return Ok(None);
    }

    let individual_permissions = individual_records
        .into_iter()
        .map(|(id, name, email, role)| IndividualPermission {
            id,
            name,
            email,
            role,
        })
        .collect();

    Ok(Some(individual_permissions))
}

async fn get_team_permissions(
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
) -> Result<Option<Vec<TeamPermissions>>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let team_records = match asset_permissions::table
        .inner_join(
            teams_to_users::table.on(asset_permissions::identity_id.eq(teams_to_users::team_id)),
        )
        .inner_join(teams::table.on(teams_to_users::team_id.eq(teams::id)))
        .inner_join(users::table.on(teams_to_users::user_id.eq(users::id)))
        .select((
            users::id,
            users::name.nullable(),
            users::email,
            asset_permissions::role,
            teams::name,
            teams::id,
        ))
        .filter(asset_permissions::identity_type.eq(IdentityType::Team))
        .filter(asset_permissions::asset_id.eq(asset_id.as_ref()))
        .filter(asset_permissions::asset_type.eq(asset_type))
        .filter(asset_permissions::deleted_at.is_null())
        .load::<(
            Uuid,
            Option<String>,
            String,
            AssetPermissionRole,
            String,
            Uuid,
        )>(&mut conn)
        .await
    {
        Ok(team_records) => team_records,
        Err(diesel::result::Error::NotFound) => return Ok(None),
        Err(e) => {
            return Err(anyhow!("Error querying team permissions: {}", e));
        }
    };

    let team_permissions: HashMap<Uuid, TeamPermissions> = team_records.into_iter().fold(
        HashMap::new(),
        |mut acc, (user_id, user_name, email, role, team_name, team_id)| {
            acc.entry(team_id)
                .or_insert_with(|| TeamPermissions {
                    id: team_id,
                    name: Some(team_name.clone()),
                    role,
                    user_permissions: Vec::new(),
                })
                .user_permissions
                .push(IndividualPermission {
                    id: user_id,
                    name: user_name,
                    email,
                    role,
                });
            acc
        },
    );

    let team_permissions = team_permissions.into_values().collect();

    Ok(Some(team_permissions))
}

async fn get_organization_permissions(asset_id: Arc<Uuid>, asset_type: AssetType) -> Result<bool> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let org_has_permission = match asset_permissions::table
        .inner_join(organizations::table.on(asset_permissions::identity_id.eq(organizations::id)))
        .select(count(asset_permissions::identity_id))
        .filter(asset_permissions::identity_type.eq(IdentityType::Organization))
        .filter(asset_permissions::asset_id.eq(asset_id.as_ref()))
        .filter(asset_permissions::asset_type.eq(asset_type))
        .filter(asset_permissions::deleted_at.is_null())
        .first::<i64>(&mut conn)
        .await
    {
        Ok(count) => count > 0,
        Err(diesel::result::Error::NotFound) => false,
        Err(e) => {
            return Err(anyhow!("Error querying organization permissions: {}", e));
        }
    };

    Ok(org_has_permission)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CollectionNameAndId {
    pub id: Uuid,
    pub name: String,
}

pub async fn get_asset_collections(
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
) -> Result<Vec<CollectionNameAndId>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let collection_records = match collections::table
        .inner_join(
            collections_to_assets::table
                .on(collections::id.eq(collections_to_assets::collection_id)),
        )
        .select((collections::id, collections::name))
        .filter(collections_to_assets::asset_id.eq(asset_id.as_ref()))
        .filter(collections_to_assets::asset_type.eq(asset_type))
        .filter(collections_to_assets::deleted_at.is_null())
        .filter(collections::deleted_at.is_null())
        .load::<(Uuid, String)>(&mut conn)
        .await
    {
        Ok(collections) => collections,
        Err(e) => {
            return Err(anyhow!("Error querying collections: {}", e));
        }
    };

    let collections = collection_records
        .into_iter()
        .map(|(id, name)| CollectionNameAndId { id, name })
        .collect();

    Ok(collections)
}

pub async fn create_asset_collection_association(
    collection_ids: Vec<Uuid>,
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    user_id: Arc<Uuid>,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let mut collections_to_assets = Vec::new();

    for collection_id in collection_ids {
        let collections_to_assets_record = CollectionToAsset {
            collection_id,
            asset_id: *asset_id,
            asset_type,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            created_by: *user_id,
            updated_by: *user_id,
        };

        collections_to_assets.push(collections_to_assets_record);
    }

    match insert_into(collections_to_assets::table)
        .values(&collections_to_assets)
        .on_conflict((
            collections_to_assets::collection_id,
            collections_to_assets::asset_id,
            collections_to_assets::asset_type,
        ))
        .do_update()
        .set((
            collections_to_assets::updated_at.eq(chrono::Utc::now()),
            collections_to_assets::updated_by.eq(*user_id),
            collections_to_assets::deleted_at.eq(None::<DateTime<Utc>>),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error inserting collection: {}", e));
        }
    };

    Ok(())
}

pub async fn delete_asset_collection_association(
    collection_ids: Vec<Uuid>,
    asset_id: Arc<Uuid>,
    asset_type: AssetType,
    user_id: Arc<Uuid>,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    match update(collections_to_assets::table)
        .filter(collections_to_assets::collection_id.eq_any(collection_ids))
        .filter(collections_to_assets::asset_id.eq(asset_id.as_ref()))
        .filter(collections_to_assets::asset_type.eq(asset_type))
        .set((
            collections_to_assets::deleted_at.eq(chrono::Utc::now()),
            collections_to_assets::updated_by.eq(*user_id),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error deleting collection: {}", e));
        }
    };

    Ok(())
}

pub async fn check_if_assets_are_shared(
    asset_ids: &Vec<Uuid>,
    asset_type: AssetType,
) -> Result<Vec<(Uuid, bool)>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let asset_records = match asset_permissions::table
        .select((
            asset_permissions::asset_id,
            sql::<Bool>("COUNT(DISTINCT identity_id) > 1 AS is_shared"),
        ))
        .filter(asset_permissions::asset_id.eq_any(asset_ids))
        .filter(asset_permissions::asset_type.eq(asset_type))
        .filter(asset_permissions::deleted_at.is_null())
        .group_by(asset_permissions::asset_id)
        .load::<(Uuid, bool)>(&mut conn)
        .await
    {
        Ok(asset_records) => asset_records,
        Err(e) => {
            tracing::error!("Unable to query asset permissions: {:?}", e);
            let err = anyhow!("Error querying asset permissions: {}", e);
            send_sentry_error(&e.to_string(), None);
            return Err(err);
        }
    };

    // For assets not in the result, add them with false (not shared)
    let result: Vec<(Uuid, bool)> = asset_ids
        .iter()
        .map(|&id| {
            asset_records
                .iter()
                .find(|&(aid, _)| *aid == id)
                .map_or((id, false), |&(aid, shared)| (aid, shared))
        })
        .collect();

    Ok(result)
}
