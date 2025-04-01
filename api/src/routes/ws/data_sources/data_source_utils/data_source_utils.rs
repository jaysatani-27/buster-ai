use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{
    deserialize::Queryable, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::{DataSourceType, UserOrganizationRole},
        lib::get_pg_pool,
        models::Dataset,
        schema::{data_sources, datasets, organizations, users, users_to_organizations},
    },
    utils::query_engine::credentials::{get_data_source_credentials, Credential},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreatedByUser {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSourceState {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "db_type")]
    pub type_: DataSourceType,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub created_by: CreatedByUser,
    pub credentials: Credential,
    pub data_sets: Vec<Dataset>,
}

#[derive(Queryable)]
pub struct DataSourceRecord {
    pub id: Uuid,
    pub name: String,
    pub type_: DataSourceType,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub secret_id: Uuid,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub user_email: String,
}

pub async fn get_data_source_state(user_id: &Uuid, id: Uuid) -> Result<DataSourceState> {
    let data_sets_handle = {
        let id = id.clone();
        tokio::spawn(async move { get_data_source_datasets(id).await })
    };

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let data_source = match data_sources::table
        .inner_join(organizations::table.on(data_sources::organization_id.eq(organizations::id)))
        .inner_join(
            users_to_organizations::table
                .on(organizations::id.eq(users_to_organizations::organization_id)),
        )
        .inner_join(users::table.on(data_sources::created_by.eq(users::id)))
        .select((
            data_sources::id,
            data_sources::name,
            data_sources::type_,
            data_sources::updated_at,
            data_sources::created_at,
            data_sources::created_by,
            data_sources::secret_id,
            users::id,
            users::name,
            users::email,
        ))
        .filter(data_sources::id.eq(id))
        .filter(users_to_organizations::user_id.eq(user_id))
        .filter(
            users_to_organizations::role
                .eq(UserOrganizationRole::WorkspaceAdmin)
                .or(users_to_organizations::role.eq(UserOrganizationRole::DataAdmin)),
        )
        .filter(users_to_organizations::deleted_at.is_null())
        .filter(data_sources::deleted_at.is_null())
        .filter(organizations::deleted_at.is_null())
        .first::<DataSourceRecord>(&mut conn)
        .await
    {
        Ok(data_sources) => data_sources,
        Err(diesel::NotFound) => return Err(anyhow!("Data source not found")),
        Err(e) => return Err(anyhow!("Error loading data sources: {:?}", e)),
    };

    let created_by = CreatedByUser {
        id: data_source.user_id,
        name: data_source.user_name,
        email: data_source.user_email,
    };

    let credentials =
        match get_data_source_credentials(&data_source.secret_id, &data_source.type_, true).await {
            Ok(credential) => credential,
            Err(e) => return Err(anyhow!("Error getting data source credentials: {:}", e)),
        };

    let datasets = match data_sets_handle.await.unwrap() {
        Ok(datasets) => datasets,
        Err(e) => return Err(anyhow!("Error getting data sets: {:?}", e)),
    };

    let response = DataSourceState {
        id: data_source.id,
        name: data_source.name,
        type_: data_source.type_,
        updated_at: data_source.updated_at,
        created_at: data_source.created_at,
        created_by: created_by,
        credentials: credentials,
        data_sets: datasets,
    };

    Ok(response)
}

async fn get_data_source_datasets(data_source_id: Uuid) -> Result<Vec<Dataset>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let datasets = match datasets::table
        .select(datasets::all_columns)
        .filter(datasets::data_source_id.eq(&data_source_id))
        .filter(datasets::deleted_at.is_null())
        .load::<Dataset>(&mut conn)
        .await
    {
        Ok(datasets) => datasets,
        Err(diesel::NotFound) => return Ok(vec![]),
        Err(e) => return Err(anyhow!("Error getting datasets: {:?}", e)),
    };

    Ok(datasets)
}
