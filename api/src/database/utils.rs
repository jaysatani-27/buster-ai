use anyhow::{anyhow, Result};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::{
    enums::{AssetPermissionRole, AssetType},
    lib::{get_pg_pool, PgPool},
    models::{Dashboard, DataSource, Term, User},
    schema::{
        api_keys, asset_permissions, dashboards, data_sources, datasets, teams_to_users, terms,
        users,
    },
};

impl User {
    pub async fn find_by_id(user_id: &Uuid, pool: &PgPool) -> Result<Option<User>> {
        let mut conn = match pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Error getting connection from pool: {}", e);
                return Err(anyhow!("Error getting connection: {}", e));
            }
        };

        let user: Option<User> = match users::table
            .filter(users::id.eq(user_id))
            .first::<User>(&mut conn)
            .await
        {
            Ok(user) => Some(user),
            Err(diesel::result::Error::NotFound) => None,
            Err(e) => {
                tracing::error!("Error querying user by ID: {}", e);
                return Err(anyhow!("Error querying user by ID: {}", e));
            }
        };

        Ok(user)
    }

    pub async fn find_by_api_key(api_key: &str, pool: &PgPool) -> Result<Option<User>> {
        let mut conn = match pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Error getting connection from pool: {}", e);
                return Err(anyhow!("Error getting connection: {}", e));
            }
        };

        let user: Option<User> = match users::table
            .inner_join(api_keys::table.on(api_keys::owner_id.eq(users::id)))
            .filter(api_keys::key.eq(api_key))
            .select(users::all_columns)
            .first::<User>(&mut conn)
            .await
        {
            Ok(user) => Some(user),
            Err(diesel::result::Error::NotFound) => None,
            Err(e) => {
                tracing::error!("Error querying user by API key: {}", e);
                return Err(anyhow!("Error querying user by API key: {}", e));
            }
        };

        Ok(user)
    }
}

impl DataSource {
    pub async fn find_by_id(data_source_id: &Uuid) -> Result<Option<DataSource>> {
        let mut conn = match get_pg_pool().get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Error getting connection from pool: {}", e);
                return Err(anyhow!("Error getting connection: {}", e));
            }
        };

        let data_source: Option<DataSource> = match data_sources::table
            .filter(data_sources::id.eq(data_source_id))
            .select((
                data_sources::id,
                data_sources::name,
                data_sources::type_,
                data_sources::secret_id,
                data_sources::onboarding_status,
                data_sources::onboarding_error,
                data_sources::organization_id,
                data_sources::created_by,
                data_sources::updated_by,
                data_sources::created_at,
                data_sources::updated_at,
                data_sources::deleted_at,
                data_sources::env,
            ))
            .first::<DataSource>(&mut conn)
            .await
        {
            Ok(data_source) => Some(data_source),
            Err(diesel::result::Error::NotFound) => None,
            Err(e) => {
                tracing::error!("Error querying data source by ID: {}", e);
                return Err(anyhow!("Error querying data source by ID: {}", e));
            }
        };

        Ok(data_source)
    }

    pub async fn find_by_dataset_id(dataset_id: &Uuid) -> Result<Option<DataSource>> {
        let mut conn = match get_pg_pool().get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Error getting connection from pool: {}", e);
                return Err(anyhow!("Error getting connection: {}", e));
            }
        };

        let data_source: Option<DataSource> = match data_sources::table
            .inner_join(datasets::table.on(datasets::data_source_id.eq(data_sources::id)))
            .filter(datasets::id.eq(dataset_id))
            .select((
                data_sources::id,
                data_sources::name,
                data_sources::type_,
                data_sources::secret_id,
                data_sources::onboarding_status,
                data_sources::onboarding_error,
                data_sources::organization_id,
                data_sources::created_by,
                data_sources::updated_by,
                data_sources::created_at,
                data_sources::updated_at,
                data_sources::deleted_at,
                data_sources::env,
            ))
            .first::<DataSource>(&mut conn)
            .await
        {
            Ok(data_source) => Some(data_source),
            Err(diesel::result::Error::NotFound) => None,
            Err(e) => {
                tracing::error!("Error querying data source by ID: {}", e);
                return Err(anyhow!("Error querying data source by ID: {}", e));
            }
        };

        Ok(data_source)
    }
}

impl Term {
    pub async fn find_by_id(pool: &PgPool, term_id: &Uuid) -> Result<Option<Term>> {
        let mut conn = match pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Error getting connection from pool: {}", e);
                return Err(anyhow!("Error getting connection: {}", e));
            }
        };

        let term: Option<Term> = match terms::table
            .filter(terms::id.eq(term_id))
            .select((
                terms::id,
                terms::name,
                terms::definition,
                terms::sql_snippet,
                terms::organization_id,
                terms::created_by,
                terms::updated_by,
                terms::created_at,
                terms::updated_at,
                terms::deleted_at,
            ))
            .first::<Term>(&mut conn)
            .await
        {
            Ok(term) => Some(term),
            Err(diesel::result::Error::NotFound) => None,
            Err(e) => {
                tracing::error!("Error querying term by ID: {}", e);
                return Err(anyhow!("Error querying term by ID: {}", e));
            }
        };

        Ok(term)
    }
}

impl Dashboard {
    pub async fn find_by_id_with_permissions(
        pool: &PgPool,
        dashboard_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(Dashboard, AssetPermissionRole)> {
        let mut conn = match pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Error getting connection from pool: {}", e);
                return Err(anyhow!("Error getting connection: {}", e));
            }
        };

        let dashboard_with_permissions = match dashboards::table
            .inner_join(
                asset_permissions::table.on(dashboards::id
                    .eq(asset_permissions::asset_id)
                    .and(asset_permissions::asset_type.eq(AssetType::Dashboard))),
            )
            .inner_join(
                teams_to_users::table
                    .on(asset_permissions::identity_id.eq(teams_to_users::team_id)),
            )
            .select((
                (
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
                ),
                asset_permissions::role,
            ))
            .filter(
                asset_permissions::identity_id
                    .eq(user_id)
                    .or(teams_to_users::user_id.eq(user_id)),
            )
            .filter(dashboards::id.eq(dashboard_id))
            .filter(dashboards::deleted_at.is_null())
            .distinct()
            .load::<(Dashboard, AssetPermissionRole)>(&mut conn)
            .await
        {
            Ok(dashboard_with_permissions) => dashboard_with_permissions,
            Err(diesel::result::Error::NotFound) => {
                tracing::error!("Dashboard not found");
                return Err(anyhow!("Dashboard not found"));
            }
            Err(e) => {
                tracing::error!("Error querying dashboard by ID: {}", e);
                return Err(anyhow!("Error querying dashboard by ID: {}", e));
            }
        };

        let dashboard_with_highest_permission = dashboard_with_permissions
            .into_iter()
            .max_by_key(|(_, role)| match role {
                AssetPermissionRole::Owner => 3,
                AssetPermissionRole::Editor => 2,
                AssetPermissionRole::Viewer => 1,
            })
            .ok_or_else(|| anyhow!("No dashboard found with permissions"))?;

        Ok(dashboard_with_highest_permission)
    }
}
