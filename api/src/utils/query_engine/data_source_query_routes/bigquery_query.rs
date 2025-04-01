use indexmap::IndexMap;

use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveTime};
use gcp_bigquery_client::{model::query_request::QueryRequest, Client};
use serde_json::{Number, Value};

use crate::utils::query_engine::data_types::DataType;

pub async fn bigquery_query(
    client: Client,
    project_id: String,
    query: String,
) -> Result<Vec<IndexMap<String, DataType>>> {
    let query_request = QueryRequest {
        connection_properties: None,
        default_dataset: None,
        dry_run: None,
        kind: None,
        labels: None,
        location: None,
        max_results: Some(500),
        maximum_bytes_billed: None,
        parameter_mode: None,
        preserve_nulls: None,
        query: query,
        query_parameters: None,
        request_id: None,
        timeout_ms: Some(120000),
        use_legacy_sql: false,
        use_query_cache: None,
        format_options: None,
    };

    let result = match client.job().query(project_id.as_str(), query_request).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("There was an issue while fetching the column values: {}", e);
            return Err(anyhow!(e));
        }
    };

    let fields = result.schema
        .as_ref()
        .and_then(|schema| schema.fields.as_ref())
        .ok_or_else(|| anyhow!("No schema found in response"))?;

    let typed_rows = result.rows
        .as_ref()
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    let mut map = IndexMap::new();
                    if let Some(cols) = &row.columns {
                        for (i, value) in cols.iter().enumerate() {
                            if i < fields.len() {
                                let field_name = &fields[i].name;
                                let data_type = match &value.value {
                                    Some(Value::String(s)) => parse_string_to_datatype(s),
                                    Some(Value::Number(n)) => parse_number_to_datatype(n),
                                    Some(Value::Bool(b)) => DataType::Bool(Some(*b)),
                                    Some(Value::Object(_)) | Some(Value::Array(_)) => 
                                        DataType::Json(value.value.clone()),
                                    Some(Value::Null) | None => 
                                        DataType::Unknown(Some("NULL".to_string())),
                                };
                                map.insert(field_name.clone(), data_type);
                            }
                        }
                    }
                    map
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(typed_rows)
}

fn parse_string_to_datatype(s: &str) -> DataType {
    if let Ok(value) = s.parse::<i32>() {
        DataType::Int4(Some(value))
    } else if let Ok(value) = s.parse::<i64>() {
        DataType::Int8(Some(value))
    } else if let Ok(value) = s.parse::<f32>() {
        DataType::Float4(Some(value))
    } else if let Ok(value) = s.parse::<f64>() {
        DataType::Float8(Some(value))
    } else if let Ok(value) = s.parse::<bool>() {
        DataType::Bool(Some(value))
    } else if let Ok(value) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        DataType::Date(Some(value))
    } else if let Ok(value) = NaiveTime::parse_from_str(s, "%H:%M:%S%.f") {
        DataType::Time(Some(value))
    } else if let Ok(value) = serde_json::from_str::<Value>(s) {
        DataType::Json(Some(value))
    } else {
        DataType::Text(Some(s.to_string()))
    }
}

fn parse_number_to_datatype(n: &Number) -> DataType {
    if let Some(i) = n.as_i64() {
        if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
            DataType::Int4(Some(i as i32))
        } else {
            DataType::Int8(Some(i))
        }
    } else if let Some(f) = n.as_f64() {
        if f >= f32::MIN as f64 && f <= f32::MAX as f64 {
            DataType::Float4(Some(f as f32))
        } else {
            DataType::Float8(Some(f))
        }
    } else {
        DataType::Unknown(Some("Invalid number".to_string()))
    }
}
