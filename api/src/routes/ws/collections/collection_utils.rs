use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::{AssetPermissionRole, AssetType},
        lib::get_pg_pool,
        models::Collection,
        schema::{
            asset_permissions, collections, collections_to_assets, dashboards, messages,
            teams_to_users, threads, users,
        },
    },
    utils::{
        clients::sentry_utils::send_sentry_error,
        sharing::asset_sharing::{get_asset_sharing_info, IndividualPermission, TeamPermissions},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetUser {
    pub name: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionAsset {
    pub id: Uuid,
    pub name: String,
    pub created_by: AssetUser,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub asset_type: AssetType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionState {
    #[serde(flatten)]
    pub collection: Collection,
    pub permission: AssetPermissionRole,
    pub individual_permissions: Option<Vec<IndividualPermission>>,
    pub team_permissions: Option<Vec<TeamPermissions>>,
    pub organization_permissions: bool,
    pub assets: Option<Vec<CollectionAsset>>,
}

pub async fn get_collection_by_id(user_id: &Uuid, collection_id: &Uuid) -> Result<CollectionState> {
    let collection_id = Arc::new(collection_id.clone());
    let user_id = Arc::new(user_id.clone());

    let collection_and_permission = {
        let collection_id = Arc::clone(&collection_id);
        let user_id = Arc::clone(&user_id);
        tokio::spawn(
            async move { get_collection_and_check_permissions(user_id, collection_id).await },
        )
    };

    let collection_sharing_info = {
        let collection_id = Arc::clone(&collection_id);
        tokio::spawn(
            async move { get_asset_sharing_info(collection_id, AssetType::Collection).await },
        )
    };

    let collection_assets = {
        let collection_id = Arc::clone(&collection_id);
        tokio::spawn(async move { get_collection_assets(collection_id).await })
    };

    let (
        collection_and_permission_result,
        collection_sharing_info_result,
        collection_assets_result,
    ) = match tokio::try_join!(
        collection_and_permission,
        collection_sharing_info,
        collection_assets
    ) {
        Ok((collection_and_permission, collection_sharing_info, collection_assets)) => (
            collection_and_permission,
            collection_sharing_info,
            collection_assets,
        ),
        Err(e) => {
            tracing::error!("Error getting collection by ID: {}", e);
            send_sentry_error(&format!("Error getting collection by ID: {}", e), None);
            return Err(anyhow!("Error getting collection by ID: {}", e));
        }
    };

    let (collection, permission) = match collection_and_permission_result {
        Ok((collection, permission)) => (collection, permission),
        Err(e) => {
            tracing::error!("Error getting collection and permission: {}", e);
            send_sentry_error(
                &format!("Error getting collection and permission: {}", e),
                None,
            );
            return Err(anyhow!("Error getting collection and permission: {}", e));
        }
    };

    let collection_sharing_info = match collection_sharing_info_result {
        Ok(mut collection_sharing_info) => {
            // Filter out the current user from individual permissions
            if let Some(ref mut individual_permissions) =
                collection_sharing_info.individual_permissions
            {
                individual_permissions.retain(|permission| permission.id != *user_id);
                if individual_permissions.is_empty() {
                    collection_sharing_info.individual_permissions = None;
                }
            }

            // Filter out the current user from team permissions
            if let Some(ref mut team_permissions) = collection_sharing_info.team_permissions {
                for team_permission in team_permissions.iter_mut() {
                    team_permission
                        .user_permissions
                        .retain(|user_permission| user_permission.id != *user_id);
                }
                // Remove teams with no remaining user permissions
                // team_permissions
                //     .retain(|team_permission| !team_permission.user_permissions.is_empty());
                // if team_permissions.is_empty() {
                //     collection_sharing_info.team_permissions = None;
                // }
            }

            // Create a map of team permissions for quick lookup
            let team_permissions_map: HashMap<Uuid, AssetPermissionRole> = collection_sharing_info
                .team_permissions
                .as_ref()
                .map(|perms| {
                    perms
                        .iter()
                        .flat_map(|p| p.user_permissions.iter().map(|up| (p.id, up.role.clone())))
                        .collect()
                })
                .unwrap_or_default();

            // Update individual permissions
            if let Some(ref mut individual_permissions) =
                collection_sharing_info.individual_permissions
            {
                for permission in individual_permissions.iter_mut() {
                    if let Some(team_role) = team_permissions_map.get(&permission.id) {
                        permission.role =
                            AssetPermissionRole::max(permission.role.clone(), team_role.clone());
                    }
                }
            }

            // Update team permissions
            if let Some(ref mut team_permissions) = collection_sharing_info.team_permissions {
                for team_permission in team_permissions.iter_mut() {
                    if let Some(individual_permissions) =
                        &collection_sharing_info.individual_permissions
                    {
                        if let Some(individual_role) = individual_permissions
                            .iter()
                            .find(|p| p.id == team_permission.id)
                            .map(|p| &p.role)
                        {
                            for user_permission in &mut team_permission.user_permissions {
                                user_permission.role = AssetPermissionRole::max(
                                    user_permission.role.clone(),
                                    individual_role.clone(),
                                );
                            }
                        }
                    }
                }
            }

            collection_sharing_info
        }
        Err(e) => {
            tracing::error!("Error getting collection sharing info: {}", e);
            send_sentry_error(
                &format!("Error getting collection sharing info: {}", e),
                None,
            );
            return Err(anyhow!("Error getting collection sharing info: {}", e));
        }
    };

    let collection_assets = match collection_assets_result {
        Ok(collection_assets) => collection_assets,
        Err(e) => {
            tracing::error!("Error getting collection assets: {}", e);
            send_sentry_error(&format!("Error getting collection assets: {}", e), None);
            return Err(anyhow!("Error getting collection assets: {}", e));
        }
    };

    Ok(CollectionState {
        collection,
        permission,
        individual_permissions: collection_sharing_info.individual_permissions,
        team_permissions: collection_sharing_info.team_permissions,
        organization_permissions: collection_sharing_info.organization_permissions,
        assets: collection_assets,
    })
}

pub async fn get_user_collection_permission(
    user_id: &Uuid,
    collection_id: &Uuid,
) -> Result<AssetPermissionRole> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let permissions = match asset_permissions::table
        .left_join(
            teams_to_users::table.on(asset_permissions::identity_id.eq(teams_to_users::team_id)),
        )
        .select(asset_permissions::role)
        .filter(
            asset_permissions::identity_id
                .eq(&user_id)
                .or(teams_to_users::user_id.eq(&user_id)),
        )
        .filter(asset_permissions::asset_id.eq(&collection_id))
        .filter(asset_permissions::deleted_at.is_null())
        .load::<AssetPermissionRole>(&mut conn)
        .await
    {
        Ok(permissions) => permissions,
        Err(diesel::result::Error::NotFound) => {
            tracing::error!("Collection not found");
            return Err(anyhow!("Collection not found"));
        }
        Err(e) => {
            tracing::error!("Error querying collection by ID: {}", e);
            return Err(anyhow!("Error querying collection by ID: {}", e));
        }
    };

    let permission = permissions
        .into_iter()
        .max_by_key(|role| match role {
            AssetPermissionRole::Owner => 3,
            AssetPermissionRole::Editor => 2,
            AssetPermissionRole::Viewer => 1,
        })
        .ok_or_else(|| anyhow!("No collection found with permissions"))?;

    Ok(permission)
}

pub async fn get_bulk_user_collection_permission(
    user_id: &Uuid,
    collection_ids: &Vec<Uuid>,
) -> Result<HashMap<Uuid, AssetPermissionRole>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let permissions = match asset_permissions::table
        .left_join(
            teams_to_users::table.on(asset_permissions::identity_id.eq(teams_to_users::team_id)),
        )
        .select((asset_permissions::asset_id, asset_permissions::role))
        .filter(
            asset_permissions::identity_id
                .eq(&user_id)
                .or(teams_to_users::user_id.eq(&user_id)),
        )
        .filter(asset_permissions::asset_id.eq_any(collection_ids))
        .filter(asset_permissions::deleted_at.is_null())
        .load::<(Uuid, AssetPermissionRole)>(&mut conn)
        .await
    {
        Ok(permissions) => permissions,
        Err(diesel::result::Error::NotFound) => {
            tracing::error!("Collection not found");
            return Err(anyhow!("Collection not found"));
        }
        Err(e) => {
            tracing::error!("Error querying collection by ID: {}", e);
            return Err(anyhow!("Error querying collection by ID: {}", e));
        }
    };

    let mut permission_map: HashMap<Uuid, AssetPermissionRole> = HashMap::new();

    for (asset_id, role) in permissions {
        permission_map
            .entry(asset_id)
            .and_modify(|e| *e = AssetPermissionRole::max(e.clone(), role.clone()))
            .or_insert(role);
    }

    Ok(permission_map)
}

async fn get_collection_and_check_permissions(
    user_id: Arc<Uuid>,
    collection_id: Arc<Uuid>,
) -> Result<(Collection, AssetPermissionRole)> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let collection_with_permissions = match collections::table
        .inner_join(
            asset_permissions::table.on(collections::id
                .eq(asset_permissions::asset_id)
                .and(asset_permissions::asset_type.eq(AssetType::Collection))),
        )
        .left_join(
            teams_to_users::table.on(asset_permissions::identity_id.eq(teams_to_users::team_id)),
        )
        .select((
            (
                collections::id,
                collections::name,
                collections::description,
                collections::created_by,
                collections::updated_by,
                collections::created_at,
                collections::updated_at,
                collections::deleted_at,
                collections::organization_id,
            ),
            asset_permissions::role,
        ))
        .filter(
            asset_permissions::identity_id
                .eq(user_id.as_ref())
                .or(teams_to_users::user_id.eq(user_id.as_ref())),
        )
        .filter(collections::id.eq(collection_id.as_ref()))
        .filter(collections::deleted_at.is_null())
        .filter(asset_permissions::deleted_at.is_null())
        .distinct()
        .load::<(Collection, AssetPermissionRole)>(&mut conn)
        .await
    {
        Ok(collection_with_permissions) => collection_with_permissions,
        Err(diesel::result::Error::NotFound) => {
            tracing::error!("Collection not found");
            return Err(anyhow!("Collection not found"));
        }
        Err(e) => {
            tracing::error!("Error querying collection by ID: {}", e);
            return Err(anyhow!("Error querying collection by ID: {}", e));
        }
    };

    let (collection, permission) = collection_with_permissions
        .into_iter()
        .max_by_key(|(_, role)| match role {
            AssetPermissionRole::Owner => 3,
            AssetPermissionRole::Editor => 2,
            AssetPermissionRole::Viewer => 1,
        })
        .ok_or_else(|| anyhow!("No collection found with permissions"))?;

    Ok((collection, permission))
}

async fn get_collection_assets(collection_id: Arc<Uuid>) -> Result<Option<Vec<CollectionAsset>>> {
    let dashboard_assets = {
        let id = Arc::clone(&collection_id);
        tokio::spawn(async move { get_dashboard_assets(id).await })
    };

    let thread_assets = {
        let id = Arc::clone(&collection_id);
        tokio::spawn(async move { get_thread_assets(id).await })
    };

    let (dashboard_assets_res, thread_assets_res) =
        match tokio::try_join!(dashboard_assets, thread_assets) {
            Ok((dashboard_assets, thread_assets)) => (dashboard_assets, thread_assets),
            Err(e) => {
                return Err(anyhow!("Error getting collection assets: {}", e));
            }
        };

    let dashboard_assets = match dashboard_assets_res {
        Ok(dashboard_assets) => dashboard_assets,
        Err(e) => {
            return Err(anyhow!("Error getting dashboard assets: {}", e));
        }
    };

    let threads_assets = match thread_assets_res {
        Ok(threads_assets) => threads_assets,
        Err(e) => {
            return Err(anyhow!("Error getting threads assets: {}", e));
        }
    };

    let mut combined_assets = Vec::new();

    if let Some(mut dashboards) = dashboard_assets {
        combined_assets.append(&mut dashboards);
    }

    if let Some(mut threads) = threads_assets {
        combined_assets.append(&mut threads);
    }

    if !combined_assets.is_empty() {
        return Ok(Some(combined_assets));
    } else {
        return Ok(None);
    }
}

async fn get_dashboard_assets(collection_id: Arc<Uuid>) -> Result<Option<Vec<CollectionAsset>>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let dashboard_assets = match collections_to_assets::table
        .inner_join(dashboards::table.on(collections_to_assets::asset_id.eq(dashboards::id)))
        .inner_join(users::table.on(dashboards::created_by.eq(users::id)))
        .select((
            dashboards::id,
            dashboards::name,
            dashboards::created_at,
            dashboards::updated_at,
            users::name.nullable(),
            users::email,
            collections_to_assets::asset_type,
        ))
        .filter(collections_to_assets::collection_id.eq(collection_id.as_ref()))
        .filter(collections_to_assets::deleted_at.is_null())
        .filter(collections_to_assets::asset_type.eq(AssetType::Dashboard))
        .distinct()
        .load::<(
            Uuid,
            String,
            DateTime<Utc>,
            DateTime<Utc>,
            Option<String>,
            String,
            AssetType,
        )>(&mut conn)
        .await
    {
        Ok(dashboard_assets) => dashboard_assets,
        Err(e) => return Err(anyhow!("Error getting dashboard assets: {}", e)),
    };

    let dashboard_assets = dashboard_assets
        .into_iter()
        .map(
            |(id, dashboard_name, created_at, updated_at, user_name, email, asset_type)| {
                CollectionAsset {
                    id,
                    name: dashboard_name,
                    created_at,
                    updated_at,
                    asset_type,
                    created_by: AssetUser {
                        name: user_name,
                        email,
                    },
                }
            },
        )
        .collect();

    Ok(Some(dashboard_assets))
}

async fn get_thread_assets(collection_id: Arc<Uuid>) -> Result<Option<Vec<CollectionAsset>>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let thread_assets = match collections_to_assets::table
        .inner_join(threads::table.on(collections_to_assets::asset_id.eq(threads::id)))
        .inner_join(users::table.on(threads::created_by.eq(users::id)))
        .inner_join(messages::table.on(threads::state_message_id.eq(messages::id.nullable())))
        .select((
            threads::id,
            messages::title.nullable(),
            messages::message,
            threads::created_at,
            threads::updated_at,
            users::name.nullable(),
            users::email,
            collections_to_assets::asset_type,
        ))
        .filter(collections_to_assets::collection_id.eq(collection_id.as_ref()))
        .filter(collections_to_assets::deleted_at.is_null())
        .filter(collections_to_assets::asset_type.eq(AssetType::Thread))
        .distinct()
        .load::<(
            Uuid,
            Option<String>,
            String,
            DateTime<Utc>,
            DateTime<Utc>,
            Option<String>,
            String,
            AssetType,
        )>(&mut conn)
        .await
    {
        Ok(thread_assets) => thread_assets,
        Err(e) => return Err(anyhow!("Error getting thread assets: {}", e)),
    };

    let thread_assets = thread_assets
        .into_iter()
        .map(
            |(id, title, message, created_at, updated_at, user_name, email, asset_type)| {
                CollectionAsset {
                    id,
                    name: title.unwrap_or(message),
                    created_at,
                    updated_at,
                    asset_type,
                    created_by: AssetUser {
                        name: user_name,
                        email,
                    },
                }
            },
        )
        .collect();

    Ok(Some(thread_assets))
}
