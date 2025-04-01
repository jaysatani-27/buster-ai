use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::database::enums::DataSourceType;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TargetDialect {
    Athena,
    BigQuery,
    Databricks,
    MySql,
    Postgres,
    Redshift,
    Snowflake,
    #[serde(rename = "tsql")]
    SqlServer,
    #[serde(rename = "mysql")]
    MariaDb,
    #[serde(rename = "postgres")]
    Supabase,
}

impl From<DataSourceType> for TargetDialect {
    fn from(data_source_type: DataSourceType) -> Self {
        match data_source_type {
            DataSourceType::BigQuery => TargetDialect::BigQuery,
            DataSourceType::Databricks => TargetDialect::Databricks,
            DataSourceType::MySql => TargetDialect::MySql,
            DataSourceType::Postgres => TargetDialect::Postgres,
            DataSourceType::Redshift => TargetDialect::Redshift,
            DataSourceType::Snowflake => TargetDialect::Snowflake,
            DataSourceType::SqlServer => TargetDialect::SqlServer,
            DataSourceType::Mariadb => TargetDialect::MariaDb,
            DataSourceType::Supabase => TargetDialect::Supabase,
        }
    }
}

pub async fn transpile_sql(sql: &String, target_dialect: TargetDialect) -> Result<String> {
    let serialized_dialect = serde_json::to_string(&target_dialect).unwrap();

    let transpiled_sql = match Command::new("./python/sqlglot_transpiler")
        .arg(sql)
        .arg(serialized_dialect.replace("\"", ""))
        .output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                tracing::error!("Command failed with exit code: {}", output.status);
                return Ok(sql.to_string());
            }

            let stdout = match String::from_utf8(output.stdout) {
                Ok(stdout) => stdout,
                Err(e) => {
                    tracing::error!("Error: {}", e);
                    return Ok(sql.to_string());
                }
            };

            stdout
        }
        Err(e) => {
            tracing::error!("Error: {}", e);
            sql.to_string()
        }
    };

    Ok(transpiled_sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transpiler() {
        let sql = "WITH customer_sales AS (
            SELECT DISTINCT
                customer_id,
                customer_name,
                SUM(total_sales_amount) AS total_sales
            FROM sales_summary
            GROUP BY customer_id, customer_name
        )
        SELECT
            customer_name,
            total_sales
        FROM customer_sales
        ORDER BY total_sales DESC
        LIMIT 1;";
        let target_dialect = TargetDialect::Postgres;
        let transpiled_sql = transpile_sql(&sql.to_string(), target_dialect)
            .await
            .unwrap();
        println!("transpiled_sql: {:?}", transpiled_sql);
    }
}
