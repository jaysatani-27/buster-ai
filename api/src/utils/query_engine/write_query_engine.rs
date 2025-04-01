use anyhow::Result;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::database::models::DataSource;

use super::data_source_query_routes::query_router::query_router;
use super::data_types::DataType;

pub async fn write_query_engine(
    dataset_id: &Uuid,
    sql: &String,
) -> Result<Vec<IndexMap<String, DataType>>> {
    let data_source = match DataSource::find_by_dataset_id(dataset_id).await? {
        Some(data_source) => data_source,
        None => return Err(anyhow::anyhow!("Data source not found")),
    };

    let results = match query_router(&data_source, sql, None, true).await {
        Ok(results) => results,
        Err(e) => return Err(e),
    };

    Ok(results)
}
