use chrono::Utc;
use indexmap::IndexMap;

use anyhow::Error;
use futures::{future::join_all, TryStreamExt};
use sqlx::{Column, MySql, Pool, Row};
use tokio::task;

use crate::utils::query_engine::data_types::DataType;

pub async fn mysql_query(
    pg_pool: Pool<MySql>,
    query: String,
) -> Result<Vec<IndexMap<std::string::String, DataType>>, Error> {
    let mut stream = sqlx::query(&query).fetch(&pg_pool);

    let mut result: Vec<IndexMap<String, DataType>>= Vec::new();

    let mut count = 0;

    while let Some(row) = stream.try_next().await? {
        let mut row_map: IndexMap<String, DataType> = IndexMap::new();

        let mut row_value_handlers = Vec::new();
        row_value_handlers.push(task::spawn(async move {
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();
                let type_info = column.type_info().clone().to_string();

                let column_value = match type_info.as_str() {
                    "BOOL" | "BOOLEAN" => DataType::Bool(row.try_get::<bool, _>(i).ok()),
                    "BIT" => DataType::Bytea(row.try_get::<Vec<u8>, _>(i).ok()),
                    "CHAR" => DataType::Char(row.try_get::<String, _>(i).ok()),
                    "BIGINT" => DataType::Int8(row.try_get::<i64, _>(i).ok()),
                    "MEDIUMINT" | "INT" | "INTEGER" => DataType::Int4(row.try_get::<i32, _>(i).ok()),
                    "TINYINT" | "SMALLINT" => DataType::Int2(row.try_get::<i16, _>(i).ok()),
                    "TEXT" | "VARCHAR" => DataType::Text(row.try_get::<String, _>(i).ok()),
                    "FLOAT" => DataType::Float4(row.try_get::<f32, _>(i).ok()),
                    "DOUBLE" => DataType::Float8(row.try_get::<f64, _>(i).ok()),
                    "DECIMAL" | "DEC" => DataType::Float8(row.try_get::<f64, _>(i).ok()),
                    "UUID" => DataType::Uuid(row.try_get::<uuid::Uuid, _>(i).ok()),
                    "TIMESTAMP" | "DATETIME" => DataType::Timestamp(row.try_get::<chrono::NaiveDateTime, _>(i).ok()),
                    "DATE" => DataType::Date(row.try_get::<chrono::NaiveDate, _>(i).ok()),
                    "TIME" => DataType::Time(row.try_get::<chrono::NaiveTime, _>(i).ok()),
                    "TIMESTAMPTZ" => DataType::Timestamptz(row.try_get::<chrono::DateTime<Utc>, _>(i).ok()),
                    "JSON" | "JSONB" => DataType::Json(row.try_get::<serde_json::Value, _>(i).ok()),
                    _ => DataType::Unknown(row.try_get::<String, _>(i).ok()),
                };

                row_map.insert(column_name.to_string(), column_value);
            }

            row_map
        }));

        let row_value_handlers_results = join_all(row_value_handlers).await;

        for row_value_handler_result in row_value_handlers_results {
            let row = row_value_handler_result.unwrap();
            result.push(row)
        }

        count += 1;
        if count >= 5000 {
            break;
        }
    }
    Ok(result)
}
