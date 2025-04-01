use anyhow::Result;
use tracing;

use crate::{
    database::models::DataSource,
    utils::{
        query_engine::{
            credentials::get_data_source_credentials,
            import_dataset_columns::retrieve_dataset_columns_batch,
        },
        validation::{
            types::{ValidationError, ValidationResult},
            type_mapping::{normalize_type, types_compatible},
        },
    },
};

pub async fn validate_model(
    model_name: &str,
    model_database_name: &str,
    schema: &str,
    database: Option<String>,
    data_source: &DataSource,
    columns: &[(&str, &str)], // (name, type) - type is now ignored for validation
    expressions: Option<&[(&str, &str)]>, // (column_name, expr)
    relationships: Option<&[(&str, &str, &str)]>, // (from_model, to_model, type)
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new(
        model_name.to_string(),
        data_source.name.clone(),
        schema.to_string(),
    );

    // Get credentials
    let credentials = match get_data_source_credentials(
        &data_source.secret_id,
        &data_source.type_,
        false,
    )
    .await
    {
        Ok(creds) => creds,
        Err(e) => {
            tracing::error!("Failed to get data source credentials: {}", e);
            result.add_error(ValidationError::data_source_error(format!(
                "Failed to get data source credentials: {}",
                e
            )));
            return Ok(result);
        }
    };

    // Collect all tables that need validation (including those referenced in relationships)
    let mut tables_to_validate = vec![(model_database_name.to_string(), schema.to_string())];
    
    // Add tables from relationships if any
    if let Some(rels) = relationships {
        for (from_model, to_model, _) in rels {
            // Add both from and to models if they're not already in the list
            let from_pair = (from_model.to_string(), schema.to_string());
            let to_pair = (to_model.to_string(), schema.to_string());
            
            if !tables_to_validate.contains(&from_pair) {
                tables_to_validate.push(from_pair);
            }
            if !tables_to_validate.contains(&to_pair) {
                tables_to_validate.push(to_pair);
            }
        }
    }

    // Get data source columns using batched retrieval for all tables at once
    let ds_columns_result = match retrieve_dataset_columns_batch(&tables_to_validate, &credentials, database).await {
        Ok(cols) => cols,
        Err(e) => {
            tracing::error!("Failed to get columns from data source: {}", e);
            result.add_error(ValidationError::data_source_error(format!(
                "Failed to get columns from data source: {}",
                e
            )));
            return Ok(result);
        }
    };

    if ds_columns_result.is_empty() {
        result.add_error(ValidationError::table_not_found(model_database_name));
        return Ok(result);
    }

    // Filter columns for the current model
    let ds_columns: Vec<_> = ds_columns_result
        .iter()
        .filter(|col| col.dataset_name == model_database_name && col.schema_name == schema)
        .collect();

    if ds_columns.is_empty() {
        result.add_error(ValidationError::table_not_found(model_database_name));
        return Ok(result);
    }

    // Validate each column exists (no type validation)
    for (col_name, _) in columns {
        if !ds_columns.iter().any(|c| c.name == *col_name) {
            result.add_error(ValidationError::column_not_found(col_name));
        }
    }

    // Validate expressions if provided
    if let Some(exprs) = expressions {
        for (col_name, expr) in exprs {
            // Check if expression references valid columns
            let expr_cols: Vec<&str> = expr
                .split_whitespace()
                .filter(|word| !word.chars().all(|c| c.is_ascii_punctuation()))
                .collect();

            for expr_col in expr_cols {
                if !columns.iter().any(|(name, _)| *name == expr_col) {
                    result.add_error(ValidationError::expression_error(
                        col_name,
                        expr,
                        &format!("Referenced column '{}' not found in model definition", expr_col),
                    ));
                }
            }
        }
    }

    // Validate relationships if provided
    if let Some(rels) = relationships {
        for (from_model, to_model, _) in rels {
            // For now, just validate that both models exist
            if !ds_columns.iter().any(|c| c.name == *from_model) {
                result.add_error(ValidationError::invalid_relationship(
                    from_model,
                    to_model,
                    "Source model not found",
                ));
            }
            if !ds_columns.iter().any(|c| c.name == *to_model) {
                result.add_error(ValidationError::invalid_relationship(
                    from_model,
                    to_model,
                    "Target model not found",
                ));
            }
        }
    }

    Ok(result)
}