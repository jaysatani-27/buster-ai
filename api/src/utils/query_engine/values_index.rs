use crate::{
    database::{
        enums::StoredValuesStatus,
        lib::get_pg_pool,
        schema::{dataset_columns, datasets},
    },
    utils::clients::typesense,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{update, AsChangeset, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;

use serde::Serialize;
use uuid::Uuid;

use super::{data_types::DataType, query_engine::query_engine};

#[derive(Debug, AsChangeset)]
#[diesel(table_name = dataset_columns)]
pub struct DatasetColumnChangeset {
    pub stored_values: Option<bool>,
    pub stored_values_status: Option<StoredValuesStatus>,
    pub stored_values_error: Option<String>,
    pub stored_values_count: Option<i64>,
    pub stored_values_last_synced: Option<chrono::DateTime<Utc>>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StoredValue {
    pub id: Uuid,
    pub value: String,
    pub dataset_id: Uuid,
    pub dataset_column_id: Uuid,
}

// TODO: This whole function and process needs to be more robust.  We just need something to move fast.
pub async fn start_stored_values_sync(dataset_column_id: &Uuid) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let (dataset_id, schema_name, database_name, dataset_column_name) = match dataset_columns::table
        .inner_join(datasets::table.on(dataset_columns::dataset_id.eq(datasets::id)))
        .select((
            dataset_columns::dataset_id,
            datasets::schema,
            datasets::database_name,
            dataset_columns::name,
        ))
        .filter(dataset_columns::id.eq(dataset_column_id))
        .first::<(Uuid, String, String, String)>(&mut conn)
        .await
    {
        Ok(dataset_record) => dataset_record,
        Err(e) => return Err(anyhow!("Error getting dataset id: {}", e)),
    };

    let dataset_columns_values_query = format!(
        "SELECT DISTINCT {} FROM {}.{} WHERE {} IS NOT NULL",
        dataset_column_name, schema_name, database_name, dataset_column_name
    );

    let mut dataset_column_changeset = DatasetColumnChangeset {
        stored_values: Some(true),
        stored_values_status: Some(StoredValuesStatus::Syncing),
        stored_values_error: None,
        stored_values_count: None,
        stored_values_last_synced: Some(Utc::now()),
        updated_at: Utc::now(),
    };

    let results = match query_engine(&dataset_id, &dataset_columns_values_query).await {
        Ok(results) => results,
        Err(e) => {
            dataset_column_changeset.stored_values_error = Some(e.to_string());
            dataset_column_changeset.stored_values_status = Some(StoredValuesStatus::Failed);
            update_collection_record(dataset_column_id, dataset_column_changeset).await?;
            return Err(anyhow!("Error querying engine: {}", e));
        }
    };

    dataset_column_changeset.stored_values_count = Some(results.len() as i64);
    dataset_column_changeset.stored_values_last_synced = Some(Utc::now());

    let mut documents = Vec::new();

    for result in results {
        let value = match result.get::<String>(&dataset_column_name) {
            Some(value) => match value {
                DataType::Text(value) => match value {
                    Some(value) => value.clone(),
                    None => {
                        dataset_column_changeset.stored_values_error =
                            Some("Value is not a string".to_string());
                        dataset_column_changeset.stored_values_status =
                            Some(StoredValuesStatus::Failed);
                        update_collection_record(dataset_column_id, dataset_column_changeset)
                            .await?;
                        return Err(anyhow!("Value is not a string"));
                    }
                },
                _ => {
                    dataset_column_changeset.stored_values_error =
                        Some("Value is not a string".to_string());
                    dataset_column_changeset.stored_values_status =
                        Some(StoredValuesStatus::Failed);
                    update_collection_record(dataset_column_id, dataset_column_changeset).await?;
                    return Err(anyhow!("Value is not a string"));
                }
            },
            None => {
                dataset_column_changeset.stored_values_error = Some("Value not found".to_string());
                dataset_column_changeset.stored_values_status = Some(StoredValuesStatus::Failed);
                update_collection_record(dataset_column_id, dataset_column_changeset).await?;
                return Err(anyhow!("Value not found"));
            }
        };

        documents.push(StoredValue {
            id: Uuid::new_v4(),
            value,
            dataset_id,
            dataset_column_id: dataset_column_id.clone(),
        });
    }

    let collection_name = format!("dataset_index_{}", dataset_id);

    match typesense::bulk_insert_documents(&collection_name, &documents).await {
        Ok(_) => {
            dataset_column_changeset.stored_values_status = Some(StoredValuesStatus::Success);
        }
        Err(e) => {
            dataset_column_changeset.stored_values_error =
                Some(format!("Error inserting documents: {}", e));
            dataset_column_changeset.stored_values_status = Some(StoredValuesStatus::Failed);
            update_collection_record(dataset_column_id, dataset_column_changeset).await?;
            return Err(anyhow!("Error inserting documents: {}", e));
        }
    };

    match update_collection_record(dataset_column_id, dataset_column_changeset).await {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error updating collection record: {}", e)),
    };

    Ok(())
}

async fn update_collection_record(
    dataset_column_id: &Uuid,
    changeset: DatasetColumnChangeset,
) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    match update(dataset_columns::table)
        .filter(dataset_columns::id.eq(*dataset_column_id))
        .set(changeset)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error updating collection record: {}", e)),
    }

    Ok(())
}
