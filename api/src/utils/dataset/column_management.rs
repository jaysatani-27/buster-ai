use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::ExpressionMethods;
use diesel::{upsert::excluded, SelectableHelper};
use diesel_async::RunQueryDsl;
use std::collections::HashSet;
use uuid::Uuid;

use crate::{
    database::{
        lib::get_pg_pool,
        models::{DataSource, Dataset, DatasetColumn},
        schema::dataset_columns,
    },
    utils::query_engine::{
        credentials::get_data_source_credentials, import_dataset_columns::retrieve_dataset_columns,
    },
};

pub struct ColumnUpdate {
    pub name: String,
    pub description: Option<String>,
    pub semantic_type: Option<String>,
    pub type_: String,
    pub nullable: bool,
    pub dim_type: Option<String>,
    pub expr: Option<String>,
    pub searchable: bool,
}

/// Retrieves column types from the data source
pub async fn get_column_types(
    dataset: &Dataset,
    database: Option<String>,
    data_source: &DataSource,
) -> Result<Vec<ColumnUpdate>> {
    let credentials =
        get_data_source_credentials(&data_source.secret_id, &data_source.type_, false)
            .await
            .map_err(|e| anyhow!("Error getting data source credentials: {}", e))?;

    let cols = retrieve_dataset_columns(
        &dataset.database_name,
        &dataset.schema,
        &credentials,
        database,
    )
    .await
    .map_err(|e| anyhow!("Error retrieving dataset columns: {}", e))?;

    Ok(cols
        .into_iter()
        .map(|col| ColumnUpdate {
            name: col.name,
            description: col.comment,
            semantic_type: None,
            type_: col.type_,
            nullable: col.nullable,
            dim_type: None,
            expr: None,
            searchable: false,
        })
        .collect())
}

/// Updates dataset columns with new information and handles soft deletes
pub async fn update_dataset_columns(
    dataset: &Dataset,
    columns: Vec<ColumnUpdate>,
) -> Result<Vec<DatasetColumn>> {
    let mut conn = get_pg_pool().get().await?;

    // Convert columns to DatasetColumn structs
    let columns_to_upsert: Vec<DatasetColumn> = columns
        .into_iter()
        .map(|col| DatasetColumn {
            id: Uuid::new_v4(),
            dataset_id: dataset.id,
            name: col.name,
            description: col.description,
            semantic_type: col.semantic_type,
            type_: col.type_,
            nullable: col.nullable,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            stored_values: Some(col.searchable),
            stored_values_status: None,
            stored_values_error: None,
            stored_values_count: None,
            stored_values_last_synced: None,
            dim_type: col.dim_type,
            expr: col.expr,
        })
        .collect();

    // Perform upsert
    let inserted_columns = diesel::insert_into(dataset_columns::table)
        .values(&columns_to_upsert)
        .on_conflict((dataset_columns::dataset_id, dataset_columns::name))
        .do_update()
        .set((
            dataset_columns::description.eq(excluded(dataset_columns::description)),
            dataset_columns::semantic_type.eq(excluded(dataset_columns::semantic_type)),
            dataset_columns::type_.eq(excluded(dataset_columns::type_)),
            dataset_columns::dim_type.eq(excluded(dataset_columns::dim_type)),
            dataset_columns::expr.eq(excluded(dataset_columns::expr)),
            dataset_columns::nullable.eq(excluded(dataset_columns::nullable)),
            dataset_columns::stored_values.eq(excluded(dataset_columns::stored_values)),
            dataset_columns::updated_at.eq(Utc::now()),
            dataset_columns::deleted_at.eq(None::<chrono::DateTime<Utc>>),
        ))
        .returning(DatasetColumn::as_select())
        .get_results(&mut conn)
        .await?;

    // Soft delete columns that weren't in the update
    let updated_column_names: HashSet<String> =
        inserted_columns.iter().map(|c| c.name.clone()).collect();

    diesel::update(dataset_columns::table)
        .filter(dataset_columns::dataset_id.eq(dataset.id))
        .filter(dataset_columns::deleted_at.is_null())
        .filter(dataset_columns::name.ne_all(updated_column_names))
        .set(dataset_columns::deleted_at.eq(Some(Utc::now())))
        .execute(&mut conn)
        .await?;

    Ok(inserted_columns)
}
