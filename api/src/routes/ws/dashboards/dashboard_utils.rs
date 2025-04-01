use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    database::{
        enums::{AssetPermissionRole, AssetType},
        lib::get_pg_pool,
        models::{Dashboard, Message},
        schema::{
            asset_permissions, dashboards, messages, teams_to_users, threads_to_dashboards,
            users_to_organizations,
        },
    },
    utils::{
        clients::{sentry_utils::send_sentry_error, supabase_vault::read_secret},
        query_engine::data_types::DataType,
        sharing::asset_sharing::{
            get_asset_collections, get_asset_sharing_info, CollectionNameAndId,
            IndividualPermission, TeamPermissions,
        },
        user::user_info::get_user_organization_id,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metric {
    pub id: Uuid, // This is the thread id
    #[serde(skip_serializing)]
    pub message_id: Uuid,
    pub name: String,
    pub time_frame: String,
    #[serde(skip_serializing)]
    pub sql: String,
    #[serde(skip_serializing)]
    pub dataset_id: Uuid,
    pub chart_config: Value,
    pub data: Option<Vec<IndexMap<String, DataType>>>,
    pub data_metadata: Option<Value>,
}

#[derive(Serialize)]
pub struct DashboardState {
    pub dashboard: Dashboard,
    pub metrics: Vec<Metric>,
    pub collections: Vec<CollectionNameAndId>,
    pub permission: Option<AssetPermissionRole>,
    pub individual_permissions: Option<Vec<IndividualPermission>>,
    pub team_permissions: Option<Vec<TeamPermissions>>,
    pub organization_permissions: bool,
    pub public_password: Option<String>,
}

pub async fn get_dashboard_state_by_id(
    user_id: &Uuid,
    dashboard_id: &Uuid,
) -> Result<DashboardState> {
    let dashboard_id = Arc::new(dashboard_id.clone());
    let user_id = Arc::new(user_id.clone());

    let dashboard_and_permission = {
        let dashboard_id = Arc::clone(&dashboard_id);
        let user_id = Arc::clone(&user_id);
        tokio::spawn(
            async move { get_dashboard_and_check_permissions(user_id, dashboard_id).await },
        )
    };

    let dashboard_sharing_info = {
        let dashboard_id = Arc::clone(&dashboard_id);
        tokio::spawn(
            async move { get_asset_sharing_info(dashboard_id, AssetType::Dashboard).await },
        )
    };

    let dashboard_metrics = {
        let dashboard_id = Arc::clone(&dashboard_id);
        tokio::spawn(async move { get_dashboard_metrics(dashboard_id).await })
    };

    let dashboard_collections = {
        let dashboard_id = Arc::clone(&dashboard_id);
        tokio::spawn(async move { get_asset_collections(dashboard_id, AssetType::Dashboard).await })
    };

    let (
        dashboard_and_permission_result,
        dashboard_sharing_info_result,
        dashboard_metrics_result,
        dashboard_collections_result,
    ) = match tokio::try_join!(
        dashboard_and_permission,
        dashboard_sharing_info,
        dashboard_metrics,
        dashboard_collections
    ) {
        Ok((
            dashboard_and_permission,
            dashboard_sharing_info,
            dashboard_metrics,
            dashboard_collections,
        )) => (
            dashboard_and_permission,
            dashboard_sharing_info,
            dashboard_metrics,
            dashboard_collections,
        ),
        Err(e) => {
            tracing::error!("Error getting dashboard by ID: {}", e);
            send_sentry_error(&format!("Error getting dashboard by ID: {}", e), None);
            return Err(anyhow!("Error getting dashboard by ID: {}", e));
        }
    };

    let (dashboard, permission) = match dashboard_and_permission_result {
        Ok((dashboard, permission)) => (dashboard, permission),
        Err(e) => {
            tracing::error!("Error getting dashboard and permission: {}", e);
            return Err(anyhow!("Error getting dashboard and permission: {}", e));
        }
    };

    let dashboard_sharing_info = match dashboard_sharing_info_result {
        Ok(info) => info,
        Err(e) => {
            tracing::error!("Error getting dashboard sharing info: {}", e);
            return Err(anyhow!("Error getting dashboard sharing info: {}", e));
        }
    };

    let dashboard_metrics = match dashboard_metrics_result {
        Ok(metrics) => metrics,
        Err(e) => {
            tracing::error!("Error getting dashboard metrics: {}", e);
            return Err(anyhow!("Error getting dashboard metrics: {}", e));
        }
    };

    let dashboard_collections = match dashboard_collections_result {
        Ok(collections) => collections,
        Err(e) => {
            tracing::error!("Error getting dashboard collections: {}", e);
            return Err(anyhow!("Error getting dashboard collections: {}", e));
        }
    };

    // Get user's organization
    let user_org = match get_user_organization_id(&user_id).await {
        Ok(org_id) => org_id,
        Err(e) => {
            tracing::error!("Error getting user organization: {}", e);
            return Err(anyhow!("Error getting user organization: {}", e));
        }
    };

    // If user has no explicit permission and is not in the same organization
    if permission.is_none() && user_org != dashboard.organization_id {
        // Check if the dashboard is directly shared with the user
        let has_direct_access = dashboard_sharing_info
            .individual_permissions
            .as_ref()
            .map(|perms| perms.iter().any(|p| p.id == *user_id))
            .unwrap_or(false);

        // Check if the dashboard is shared with any of user's teams
        let has_team_access = dashboard_sharing_info
            .team_permissions
            .as_ref()
            .map(|perms| {
                perms.iter().any(|team_perm| {
                    team_perm
                        .user_permissions
                        .iter()
                        .any(|team_user| team_user.id == *user_id)
                })
            })
            .unwrap_or(false);

        // If no direct or team access, return unauthorized
        if !has_direct_access && !has_team_access && !dashboard.publicly_accessible {
            return Err(anyhow!("Unauthorized access to dashboard"));
        }
    }

    let public_password = if dashboard.publicly_accessible {
        if let Some(secret_id) = dashboard.password_secret_id {
            match read_secret(&secret_id).await {
                Ok(password) => Some(password),
                Err(e) => {
                tracing::error!("Error getting dashboard password: {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(DashboardState {
        dashboard,
        permission,
        collections: dashboard_collections,
        individual_permissions: dashboard_sharing_info.individual_permissions,
        team_permissions: dashboard_sharing_info.team_permissions,
        organization_permissions: dashboard_sharing_info.organization_permissions,
        metrics: dashboard_metrics,
        public_password,
    })
}

pub async fn get_user_dashboard_permission(
    user_id: &Uuid,
    dashboard_id: &Uuid,
) -> Result<Option<AssetPermissionRole>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    // First get the dashboard's organization
    let dashboard_org = match dashboards::table
        .select(dashboards::organization_id)
        .filter(dashboards::id.eq(dashboard_id))
        .filter(dashboards::deleted_at.is_null())
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(org_id) => org_id,
        Err(diesel::result::Error::NotFound) => return Ok(None),
        Err(e) => {
            tracing::error!("Error querying dashboard organization: {}", e);
            return Err(anyhow!("Error querying dashboard organization: {}", e));
        }
    };

    // Then get the user's organization
    let user_org = match users_to_organizations::table
        .select(users_to_organizations::organization_id)
        .filter(users_to_organizations::user_id.eq(user_id))
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(org_id) => org_id,
        Err(diesel::result::Error::NotFound) => return Ok(None),
        Err(e) => {
            tracing::error!("Error querying user organization: {}", e);
            return Err(anyhow!("Error querying user organization: {}", e));
        }
    };

    // If organizations don't match, deny access
    if dashboard_org != user_org {
        return Ok(None);
    }

    let permissions = match asset_permissions::table
        .select(asset_permissions::role)
        .filter(asset_permissions::identity_id.eq(&user_id))
        .filter(asset_permissions::asset_id.eq(&dashboard_id))
        .filter(asset_permissions::deleted_at.is_null())
        .load::<AssetPermissionRole>(&mut conn)
        .await
    {
        Ok(permissions) => permissions,
        Err(diesel::result::Error::NotFound) => return Ok(None),
        Err(e) => {
            tracing::error!("Error querying dashboard by ID: {}", e);
            return Err(anyhow!("Error querying dashboard by ID: {}", e));
        }
    };

    if permissions.is_empty() {
        return Ok(None);
    }

    let permission = permissions
        .into_iter()
        .max_by_key(|role| match role {
            AssetPermissionRole::Owner => 3,
            AssetPermissionRole::Editor => 2,
            AssetPermissionRole::Viewer => 1,
        })
        .ok_or_else(|| anyhow!("No dashboard found with permissions"))?;

    Ok(Some(permission))
}

pub async fn get_bulk_user_dashboard_permission(
    user_id: &Uuid,
    dashboard_ids: &Vec<Uuid>,
) -> Result<HashMap<Uuid, AssetPermissionRole>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    // Get user's organization
    let user_org = match users_to_organizations::table
        .select(users_to_organizations::organization_id)
        .filter(users_to_organizations::user_id.eq(user_id))
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(org_id) => org_id,
        Err(diesel::result::Error::NotFound) => return Ok(HashMap::new()),
        Err(e) => {
            tracing::error!("Error querying user organization: {}", e);
            return Err(anyhow!("Error querying user organization: {}", e));
        }
    };

    let permissions = match asset_permissions::table
        .inner_join(dashboards::table.on(asset_permissions::asset_id.eq(dashboards::id)))
        .select((asset_permissions::asset_id, asset_permissions::role))
        .filter(asset_permissions::identity_id.eq(&user_id))
        .filter(asset_permissions::asset_id.eq_any(dashboard_ids))
        .filter(asset_permissions::deleted_at.is_null())
        .filter(dashboards::organization_id.eq(user_org))
        .load::<(Uuid, AssetPermissionRole)>(&mut conn)
        .await
    {
        Ok(permissions) => permissions,
        Err(diesel::result::Error::NotFound) => {
            tracing::error!("dashboard not found");
            return Err(anyhow!("dashboard not found"));
        }
        Err(e) => {
            tracing::error!("Error querying dashboard by ID: {}", e);
            return Err(anyhow!("Error querying dashboard by ID: {}", e));
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

async fn get_dashboard_and_check_permissions(
    user_id: Arc<Uuid>,
    dashboard_id: Arc<Uuid>,
) -> Result<(Dashboard, Option<AssetPermissionRole>)> {
    let dashboard_handler = {
        let id = Arc::clone(&dashboard_id);
        tokio::spawn(async move { get_dashboard_by_id(id).await })
    };

    let permission_handler =
        tokio::spawn(async move { get_user_dashboard_permission(&user_id, &dashboard_id).await });

    let (dashboard_result, permission_result) =
        match tokio::try_join!(dashboard_handler, permission_handler) {
            Ok((dashboard, permission)) => (dashboard, permission),
            Err(e) => return Err(anyhow!("Error getting dashboard or permission: {}", e)),
        };

    let dashboard = match dashboard_result {
        Ok(dashboard) => dashboard,
        Err(e) => return Err(anyhow!("Error getting dashboard: {}", e)),
    };

    let permission = match permission_result {
        Ok(permission) => permission,
        Err(e) => return Err(anyhow!("Error getting permission: {}", e)),
    };

    Ok((dashboard, permission))
}

async fn get_dashboard_by_id(dashboard_id: Arc<Uuid>) -> Result<Dashboard> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let dashboard = match dashboards::table
        .filter(dashboards::id.eq(dashboard_id.as_ref()))
        .filter(dashboards::deleted_at.is_null())
        .select((
            dashboards::id,
            dashboards::name,
            dashboards::description,
            dashboards::config,
            dashboards::publicly_accessible,
            dashboards::publicly_enabled_by,
            dashboards::public_expiry_date,
            dashboards::password_secret_id,
            dashboards::created_by,
            dashboards::updated_by,
            dashboards::created_at,
            dashboards::updated_at,
            dashboards::deleted_at,
            dashboards::organization_id,
        ))
        .first::<Dashboard>(&mut conn)
        .await
    {
        Ok(dashboards) => dashboards,
        Err(diesel::result::Error::NotFound) => {
            return Err(anyhow!("dashboard not found"));
        }
        Err(e) => {
            return Err(anyhow!("Error querying dashboard by ID: {}", e));
        }
    };

    Ok(dashboard)
}

async fn get_dashboard_metrics(dashboard_id: Arc<Uuid>) -> Result<Vec<Metric>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let metric_records = match messages::table
        .inner_join(
            threads_to_dashboards::table
                .on(messages::thread_id.eq(threads_to_dashboards::thread_id)),
        )
        .select((threads_to_dashboards::thread_id, messages::all_columns))
        .filter(threads_to_dashboards::dashboard_id.eq(dashboard_id.as_ref()))
        .filter(messages::deleted_at.is_null())
        .filter(messages::draft_session_id.is_null())
        .filter(threads_to_dashboards::deleted_at.is_null())
        .load::<(Uuid, Message)>(&mut conn)
        .await
    {
        Ok(metric_records) => metric_records,
        Err(e) => {
            return Err(anyhow!("Error querying metric records: {}", e));
        }
    };

    let mut metrics = Vec::new();
    let mut thread_messages: HashMap<Uuid, Message> = HashMap::new();

    // Group messages by thread and keep the most recent one
    for (thread_id, message) in metric_records {
        thread_messages
            .entry(thread_id)
            .and_modify(|existing_message| {
                if message.created_at > existing_message.created_at {
                    *existing_message = message.clone();
                }
            })
            .or_insert(message);
    }

    // Create Metric objects from the most recent messages
    for (thread_id, message) in thread_messages {
        let metric = Metric {
            id: thread_id,
            message_id: message.id,
            name: message.title.unwrap_or_default(),
            time_frame: message.time_frame.unwrap_or_default(),
            sql: message.code.unwrap_or_default(),
            dataset_id: message.dataset_id.unwrap_or_default(),
            chart_config: message.chart_config.unwrap_or_default(),
            data: None,
            data_metadata: message.data_metadata,
        };
        metrics.push(metric);
    }

    Ok(metrics)
}
