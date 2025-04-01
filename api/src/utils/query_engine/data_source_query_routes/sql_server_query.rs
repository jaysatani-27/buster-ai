use crate::utils::query_engine::data_types::DataType;
use anyhow::{anyhow, Error, Result};
use chrono::NaiveDateTime;
use futures::future::join_all;
use indexmap::IndexMap;
use tiberius::{numeric::Decimal, Client, ColumnType};
use tokio::{net::TcpStream, task};
use tokio_util::compat::Compat;

pub async fn sql_server_query(
    mut client: Client<Compat<TcpStream>>,
    query: String,
) -> Result<Vec<IndexMap<std::string::String, DataType>>, Error> {
    let rows = match client.query(query, &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("Unable to execute query: {:?}", e);
            let err = anyhow!("Unable to execute query: {}", e);
            return Err(err);
        }
    };

    let mut result: Vec<IndexMap<String, DataType>> = Vec::new();
    let query_result = match rows.into_first_result().await {
        Ok(query_result) => query_result.into_iter().take(1000),
        Err(e) => {
            tracing::error!("Unable to fetch query result: {:?}", e);
            let err = anyhow!("Unable to fetch query result: {}", e);
            return Err(err);
        }
    };

    for row in query_result {
        let mut row_value_handlers = Vec::new();

        row_value_handlers.push(task::spawn(async move {
            let mut row_map = IndexMap::new();
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();
                let type_info = column.column_type().clone();
                let column_value = match type_info {
                    ColumnType::Text
                    | ColumnType::NVarchar
                    | ColumnType::NChar
                    | ColumnType::BigChar
                    | ColumnType::NText
                    | ColumnType::BigVarChar => {
                        DataType::Text(row.get::<&str, _>(i).map(|v| v.to_string()))
                    }
                    ColumnType::Int8 => DataType::Bool(row.get::<bool, _>(i).map(|v| v)),
                    ColumnType::Int4 => DataType::Int4(row.get::<i32, _>(i).map(|v| v)),
                    ColumnType::Int2 | ColumnType::Int1 => DataType::Int2(row.get::<i16, _>(i).map(|v| v)),
                    ColumnType::Float4 => DataType::Float4(row.get::<f32, _>(i).map(|v| v)),
                    ColumnType::Float8 => DataType::Float8(row.get::<f64, _>(i).map(|v| v)),
                    ColumnType::Bit => DataType::Bool(row.get::<bool, _>(i).map(|v| v)),
                    ColumnType::Null => DataType::Null,
                    ColumnType::Datetime4 => {
                        DataType::Timestamp(row.get::<NaiveDateTime, _>(i).map(|v| v))
                    }
                    ColumnType::Money => DataType::Int8(row.get::<i64, _>(i).map(|v| v)),
                    ColumnType::Datetime => DataType::Timestamp(row.get::<NaiveDateTime, _>(i).map(|v| v)),
                    ColumnType::Money4 => DataType::Int8(row.get::<i64, _>(i).map(|v| v)),
                    ColumnType::Guid => DataType::Uuid(row.get::<uuid::Uuid, _>(i).map(|v| v)),
                    ColumnType::Intn => DataType::Int4(row.get::<i32, _>(i).map(|v| v)),
                    ColumnType::Decimaln => DataType::Decimal(row.get::<Decimal, _>(i).map(|v| v)),
                    ColumnType::Numericn => DataType::Decimal(row.get::<Decimal, _>(i).map(|v| v)),
                    ColumnType::Floatn => DataType::Float8(row.get::<f64, _>(i).map(|v| v)),
                    ColumnType::Datetimen => {
                        DataType::Timestamp(row.get::<NaiveDateTime, _>(i).map(|v| v))
                    }
                    ColumnType::Daten => DataType::Date(row.get::<NaiveDateTime, _>(i).map(|v| v.date())),
                    ColumnType::Timen => DataType::Time(row.get::<NaiveDateTime, _>(i).map(|v| v.time())),
                    ColumnType::Datetime2 => DataType::Timestamp(row.get::<NaiveDateTime, _>(i).map(|v| v)),
                    ColumnType::DatetimeOffsetn => DataType::Timestamp(row.get::<NaiveDateTime, _>(i).map(|v| v)),
                    _ => {
                        tracing::debug!("No match found");
                        DataType::Null
                    }
                };
                tracing::debug!("column_value: {:?}", column_value);
                row_map.insert(column_name.to_string(), column_value);
            }
            row_map
        }));
        let row_value_handlers_results = join_all(row_value_handlers).await;
        for row_value_handler_result in row_value_handlers_results {
            let row = row_value_handler_result.unwrap();
            result.push(row);
        }
    }
    Ok(result)
}
