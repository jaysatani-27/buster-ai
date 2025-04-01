use crate::{database::enums::DataSourceType, utils::query_engine::data_types::DataType};
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Standard types we use across all data sources
#[derive(Debug, Clone, PartialEq)]
pub enum StandardType {
    Text,
    Integer,
    Float,
    Boolean,
    Date,
    Timestamp,
    Json,
    Unknown,
}

impl StandardType {
    pub fn from_data_type(data_type: &DataType) -> Self {
        match data_type.simple_type() {
            Some(simple_type) => match simple_type.as_str() {
                "string" => StandardType::Text,
                "number" => StandardType::Float,
                "boolean" => StandardType::Boolean,
                "date" => StandardType::Date,
                "null" => StandardType::Unknown,
                _ => StandardType::Unknown,
            },
            None => StandardType::Unknown,
        }
    }

    pub fn from_str(type_str: &str) -> Self {
        match type_str.to_lowercase().as_str() {
            "text" | "string" | "varchar" | "char" => StandardType::Text,
            "int" | "integer" | "bigint" | "smallint" => StandardType::Integer,
            "float" | "double" | "decimal" | "numeric" => StandardType::Float,
            "bool" | "boolean" => StandardType::Boolean,
            "date" => StandardType::Date,
            "timestamp" | "datetime" | "timestamptz" => StandardType::Timestamp,
            "json" | "jsonb" => StandardType::Json,
            _ => StandardType::Unknown,
        }
    }
}

// Type mappings for each data source type to DataType
static TYPE_MAPPINGS: Lazy<HashMap<DataSourceType, HashMap<&'static str, DataType>>> = Lazy::new(|| {
    let mut mappings = HashMap::new();
    
    // Postgres mappings
    let mut postgres = HashMap::new();
    postgres.insert("text", DataType::Text(None));
    postgres.insert("varchar", DataType::Text(None));
    postgres.insert("char", DataType::Text(None));
    postgres.insert("int", DataType::Int4(None));
    postgres.insert("integer", DataType::Int4(None));
    postgres.insert("bigint", DataType::Int8(None));
    postgres.insert("smallint", DataType::Int2(None));
    postgres.insert("float", DataType::Float4(None));
    postgres.insert("double precision", DataType::Float8(None));
    postgres.insert("numeric", DataType::Decimal(None));
    postgres.insert("decimal", DataType::Decimal(None));
    postgres.insert("boolean", DataType::Bool(None));
    postgres.insert("date", DataType::Date(None));
    postgres.insert("timestamp", DataType::Timestamp(None));
    postgres.insert("timestamptz", DataType::Timestamptz(None));
    postgres.insert("json", DataType::Json(None));
    postgres.insert("jsonb", DataType::Json(None));
    mappings.insert(DataSourceType::Postgres, postgres.clone());
    mappings.insert(DataSourceType::Supabase, postgres);

    // MySQL mappings
    let mut mysql = HashMap::new();
    mysql.insert("varchar", DataType::Text(None));
    mysql.insert("text", DataType::Text(None));
    mysql.insert("char", DataType::Text(None));
    mysql.insert("int", DataType::Int4(None));
    mysql.insert("bigint", DataType::Int8(None));
    mysql.insert("tinyint", DataType::Int2(None));
    mysql.insert("float", DataType::Float4(None));
    mysql.insert("double", DataType::Float8(None));
    mysql.insert("decimal", DataType::Decimal(None));
    mysql.insert("boolean", DataType::Bool(None));
    mysql.insert("date", DataType::Date(None));
    mysql.insert("datetime", DataType::Timestamp(None));
    mysql.insert("timestamp", DataType::Timestamptz(None));
    mysql.insert("json", DataType::Json(None));
    mappings.insert(DataSourceType::MySql, mysql.clone());
    mappings.insert(DataSourceType::Mariadb, mysql);

    // BigQuery mappings
    let mut bigquery = HashMap::new();
    bigquery.insert("STRING", DataType::Text(None));
    bigquery.insert("INT64", DataType::Int8(None));
    bigquery.insert("INTEGER", DataType::Int4(None));
    bigquery.insert("FLOAT64", DataType::Float8(None));
    bigquery.insert("NUMERIC", DataType::Decimal(None));
    bigquery.insert("BOOL", DataType::Bool(None));
    bigquery.insert("DATE", DataType::Date(None));
    bigquery.insert("TIMESTAMP", DataType::Timestamptz(None));
    bigquery.insert("JSON", DataType::Json(None));
    mappings.insert(DataSourceType::BigQuery, bigquery);

    // Snowflake mappings
    let mut snowflake = HashMap::new();
    snowflake.insert("TEXT", DataType::Text(None));
    snowflake.insert("VARCHAR", DataType::Text(None));
    snowflake.insert("CHAR", DataType::Text(None));
    snowflake.insert("NUMBER", DataType::Decimal(None));
    snowflake.insert("DECIMAL", DataType::Decimal(None));
    snowflake.insert("INTEGER", DataType::Int4(None));
    snowflake.insert("BIGINT", DataType::Int8(None));
    snowflake.insert("BOOLEAN", DataType::Bool(None));
    snowflake.insert("DATE", DataType::Date(None));
    snowflake.insert("TIMESTAMP", DataType::Timestamptz(None));
    snowflake.insert("VARIANT", DataType::Json(None));
    mappings.insert(DataSourceType::Snowflake, snowflake);

    mappings
});

pub fn normalize_type(source_type: DataSourceType, type_str: &str) -> DataType {
    TYPE_MAPPINGS
        .get(&source_type)
        .and_then(|mappings| mappings.get(type_str))
        .cloned()
        .unwrap_or(DataType::Unknown(Some(type_str.to_string())))
}

pub fn types_compatible(source_type: DataSourceType, ds_type: &str, model_type: &str) -> bool {
    let ds_data_type = normalize_type(source_type, ds_type);
    let model_data_type = normalize_type(source_type, model_type);
    
    match (&ds_data_type, &model_data_type) {
        // Allow integer -> float/decimal conversions
        (DataType::Int2(_), DataType::Float4(_)) => true,
        (DataType::Int2(_), DataType::Float8(_)) => true,
        (DataType::Int2(_), DataType::Decimal(_)) => true,
        (DataType::Int4(_), DataType::Float4(_)) => true,
        (DataType::Int4(_), DataType::Float8(_)) => true,
        (DataType::Int4(_), DataType::Decimal(_)) => true,
        (DataType::Int8(_), DataType::Float4(_)) => true,
        (DataType::Int8(_), DataType::Float8(_)) => true,
        (DataType::Int8(_), DataType::Decimal(_)) => true,
        
        // Allow text for any type (common in views/computed columns)
        (DataType::Text(_), _) => true,
        
        // Allow timestamp/timestamptz compatibility
        (DataType::Timestamp(_), DataType::Timestamptz(_)) => true,
        (DataType::Timestamptz(_), DataType::Timestamp(_)) => true,
        
        // Exact matches (using to_string to compare types, not values)
        (a, b) if a.to_string() == b.to_string() => true,
        
        // Everything else is incompatible
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_type_normalization() {
        assert!(matches!(
            normalize_type(DataSourceType::Postgres, "text"),
            DataType::Text(_)
        ));
        assert!(matches!(
            normalize_type(DataSourceType::Postgres, "integer"),
            DataType::Int4(_)
        ));
        assert!(matches!(
            normalize_type(DataSourceType::Postgres, "numeric"),
            DataType::Decimal(_)
        ));
    }

    #[test]
    fn test_bigquery_type_normalization() {
        assert!(matches!(
            normalize_type(DataSourceType::BigQuery, "STRING"),
            DataType::Text(_)
        ));
        assert!(matches!(
            normalize_type(DataSourceType::BigQuery, "INT64"),
            DataType::Int8(_)
        ));
        assert!(matches!(
            normalize_type(DataSourceType::BigQuery, "FLOAT64"),
            DataType::Float8(_)
        ));
    }

    #[test]
    fn test_type_compatibility() {
        // Same types are compatible
        assert!(types_compatible(
            DataSourceType::Postgres,
            "text",
            "text"
        ));

        // Integer can be used as float
        assert!(types_compatible(
            DataSourceType::Postgres,
            "integer",
            "float"
        ));

        // Text can be used for any type
        assert!(types_compatible(
            DataSourceType::Postgres,
            "text",
            "integer"
        ));

        // Different types are incompatible
        assert!(!types_compatible(
            DataSourceType::Postgres,
            "integer",
            "text"
        ));

        // Timestamp compatibility
        assert!(types_compatible(
            DataSourceType::Postgres,
            "timestamp",
            "timestamptz"
        ));
    }
} 