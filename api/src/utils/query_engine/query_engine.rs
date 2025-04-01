use anyhow::Result;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::database::enums::UserOrganizationRole;
use crate::database::lib::get_pg_pool;
use crate::database::models::DataSource;
use crate::database::schema::{data_sources, users_to_organizations};

use super::data_source_query_routes::query_router::query_router;
use super::data_types::DataType;

pub async fn query_engine(
    dataset_id: &Uuid,
    sql: &String,
) -> Result<Vec<IndexMap<String, DataType>>> {
    let data_source = match DataSource::find_by_dataset_id(dataset_id).await? {
        Some(data_source) => data_source,
        None => return Err(anyhow::anyhow!("Data source not found")),
    };

    let results = match query_router(&data_source, sql, None, false).await {
        Ok(results) => results,
        Err(e) => return Err(e),
    };

    Ok(results)
}

pub async fn modeling_query_engine(
    data_source_id: &Uuid,
    sql: &String,
    user_id: &Uuid,
) -> Result<Vec<IndexMap<String, DataType>>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Error getting connection from pool: {:?}",
                e
            ))
        }
    };

    let is_org_admin_or_owner = users_to_organizations::table
        .inner_join(
            data_sources::table
                .on(data_sources::organization_id.eq(users_to_organizations::organization_id)),
        )
        .filter(users_to_organizations::user_id.eq(user_id))
        .filter(data_sources::id.eq(data_source_id))
        .filter(
            users_to_organizations::role
                .eq(UserOrganizationRole::WorkspaceAdmin)
                .or(users_to_organizations::role.eq(UserOrganizationRole::DataAdmin)),
        )
        .select(users_to_organizations::user_id)
        .first::<Uuid>(&mut conn)
        .await
        .is_ok();

    if !is_org_admin_or_owner {
        return Err(anyhow::anyhow!(
            "User does not have access to this data source"
        ));
    }

    let data_source = match DataSource::find_by_id(data_source_id).await? {
        Some(data_source) => data_source,
        None => return Err(anyhow::anyhow!("Data source not found")),
    };

    let results = match query_router(&data_source, sql, Some(25), false).await {
        Ok(results) => results,
        Err(e) => return Err(e),
    };

    Ok(results)
}
