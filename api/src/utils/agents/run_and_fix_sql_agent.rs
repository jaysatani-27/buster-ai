use indexmap::IndexMap;
use rayon::prelude::*;
use regex::Regex;
use serde_json::{json, Value};
use std::{fmt, time::Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

use std::collections::HashSet;
const MAX_UNIQUE_VALUES: usize = 100;

use crate::{
    database::lib::{ColumnMetadata, DataMetadataJsonBody, MinMaxValue},
    utils::{
        agent_builder::nodes::{
            error_node::ErrorNode,
            prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
        },
        query_engine::{data_types::DataType, query_engine::query_engine},
    },
};

use super::data_analyst_agent::{Thought, Thoughts};

pub struct RunAndFixSqlAgentOptions {
    pub sql_input: String,
    pub dataset_id: Uuid,
    pub dataset: String,
    pub thoughts: Thoughts,
    pub start_time: Instant,
    pub output_sender: mpsc::Sender<Value>,
}

pub enum RunAndFixSqlAgentError {
    NoSqlFound,
    SqlExecutionError,
    MaxRetriesExceeded,
    PromptNodeError,
}

impl fmt::Display for RunAndFixSqlAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSqlFound => write!(f, "no_sql_found"),
            Self::SqlExecutionError => write!(f, "sql_execution_error"),
            Self::MaxRetriesExceeded => write!(f, "max_retries_exceeded"),
            Self::PromptNodeError => write!(f, "prompt_node_error"),
        }
    }
}

pub async fn run_and_fix_sql_agent(options: RunAndFixSqlAgentOptions) -> Result<Value, ErrorNode> {
    let mut thoughts = options.thoughts;

    // Extract SQL from markdown
    let re = Regex::new(r"```sql\s*([\s\S]*?)\s*```").unwrap();
    let sql = match re.captures(&options.sql_input) {
        Some(caps) => caps.get(1).unwrap().as_str().to_string(),
        None => {
            return Err(ErrorNode::new(
                RunAndFixSqlAgentError::NoSqlFound.to_string(),
                "No SQL found between ```sql tags".to_string(),
            ));
        }
    };

    let mut current_sql = sql;
    let mut current_error = String::new();
    let max_retries = 3;
    let mut final_result = None;

    for attempt in 0..max_retries {
        let start_time = Instant::now();
        let attempt_uuid = Uuid::new_v4();

        send_message(
            "running_sql_started".to_string(),
            Value::String(format!(
                "\n\n<buster-timestamp status='inProgress' id='{}' title='Running the SQL...' milliseconds=''></buster-timestamp>\n\n",
                attempt_uuid.to_string()
            )),
            options.output_sender.clone(),
        )
        .await?;

        match fetch_data(&current_sql, &options.dataset_id).await {
            Ok(result) => {
                final_result = Some(result);

                let end_time = Instant::now();
                let duration = end_time.duration_since(start_time);

                send_message(
                    "running_sql_completed".to_string(),
                    Value::String(  format!(
                        "\n\n<buster-timestamp status='completed' id='{}' title='SQL ran successfully' milliseconds='{}'></buster-timestamp>\n\n",
                        attempt_uuid.to_string(),
                        duration.as_millis()
                    ))  ,
                    options.output_sender.clone(),
                )
                .await?;

                if attempt > 0 {
                    let duration = Instant::now().duration_since(options.start_time);

                    let main_title = format!("Thought for {} seconds", duration.as_secs());

                    thoughts.title = main_title.to_string();

                    send_message(
                        "thought_finished".to_string(),
                        serde_json::to_value(&thoughts).unwrap(),
                        options.output_sender.clone(),
                    )
                    .await?;

                    send_message(
                        "custom_response".to_string(),
                        Value::String(format!("```sql\n{}\n```", current_sql)),
                        options.output_sender.clone(),
                    )
                    .await?;
                }

                break;
            }
            Err(error) => {
                current_error = error.error_message.to_string();

                let duration = Instant::now().duration_since(start_time);

                send_message(
                    "running_sql_completed".to_string(),
                    Value::String(format!(
                        "\n\n<buster-timestamp status='completed' id='{}' title='Error returned' milliseconds='{}'></buster-timestamp>\n\n",
                        attempt_uuid.to_string(),
                        duration.as_millis()
                    )),
                    options.output_sender.clone(),
                )
                .await?;

                if attempt == max_retries - 1 {
                    let duration = Instant::now().duration_since(options.start_time);

                    let main_title = format!("Thought for {} seconds", duration.as_secs());

                    let title = format!("Attempt #{}: Fixing SQL...", attempt + 1);

                    thoughts.title = main_title.to_string();

                    thoughts.thoughts.push(Thought {
                        type_: "codeBlock".to_string(),
                        title: title,
                        content: None,
                        code: Some(current_sql.clone()),
                        error: Some(error.error_message.to_string()),
                    });

                    send_message(
                        "thought_finished".to_string(),
                        serde_json::to_value(&thoughts).unwrap(),
                        options.output_sender.clone(),
                    )
                    .await?;

                    break;
                }

                let main_title = format!("Attempt #{}: Fixing SQL...", attempt + 1);

                thoughts.title = main_title.to_string();

                thoughts.thoughts.push(Thought {
                    type_: "codeBlock".to_string(),
                    title: main_title,
                    content: None,
                    code: Some(current_sql.clone()),
                    error: Some(error.error_message.to_string()),
                });

                send_message(
                    "thought".to_string(),
                    serde_json::to_value(&thoughts).unwrap(),
                    options.output_sender.clone(),
                )
                .await?;

                let fix_sql_attempt_uuid = Uuid::new_v4();
                let fix_sql_start_time = Instant::now();

                send_message(
                    "generating_sql".to_string(),
                    Value::String(format!("\n\n<buster-timestamp status='inProgress' id='{}' title='Generating new SQL...' milliseconds=''></buster-timestamp>\n\n", fix_sql_attempt_uuid.to_string())),
                    options.output_sender.clone(),
                )
                .await?;

                // Use LLM to fix the SQL
                let fix_sql_prompt_settings = PromptNodeSettings {
                    messages: create_fix_sql_messages(
                        &current_sql,
                        &current_error,
                        &options.dataset,
                    ),
                    prompt_name: "fix_sql".to_string(),
                    model: "gpt-4o".to_string(),
                    ..Default::default()
                };

                current_sql = match prompt_node(fix_sql_prompt_settings).await {
                    Ok(Value::String(fixed_sql)) => {
                        // Extract SQL from markdown
                        let re = Regex::new(r"```sql\s*([\s\S]*?)\s*```").unwrap();
                        let sql = match re.captures(&fixed_sql) {
                            Some(caps) => caps.get(1).unwrap().as_str().to_string(),
                            None => {
                                return Err(ErrorNode::new(
                                    RunAndFixSqlAgentError::NoSqlFound.to_string(),
                                    "No SQL found between ```sql tags".to_string(),
                                ));
                            }
                        };

                        sql
                    }
                    Ok(_) => {
                        return Err(ErrorNode::new(
                            RunAndFixSqlAgentError::PromptNodeError.to_string(),
                            "SQL fix response is not a string".to_string(),
                        ));
                    }
                    Err(e) => return Err(e),
                };

                let duration = Instant::now().duration_since(fix_sql_start_time);

                send_message(
                    "generating_sql".to_string(),
                    Value::String(format!("\n\n<buster-timestamp status='completed' id='{}' title='Generated new SQL' milliseconds='{}'></buster-timestamp>\n\n", fix_sql_attempt_uuid.to_string(), duration.as_millis())),
                    options.output_sender.clone(),
                )
                .await?;
            }
        }
    }

    let (results, data_metadata, error) = match final_result {
        Some(result) => (Some(result.data), Some(result.data_metadata), None),
        None => (None, None, Some(current_error)),
    };

    Ok(json!({
        "name": "run_and_fix_sql",
        "sql": current_sql,
        "results": results,
        "data_metadata": data_metadata,
        "thoughts": thoughts,
        "error": error,
    }))
}

#[derive(Debug)]
pub struct DataObject {
    pub data: Vec<IndexMap<String, DataType>>,
    pub data_metadata: DataMetadataJsonBody,
}

pub async fn fetch_data(sql: &String, dataset_id: &Uuid) -> Result<DataObject, ErrorNode> {
    let data = match query_engine(&dataset_id, &sql).await {
        Ok(data) => data,
        Err(e) => {
            return Err(ErrorNode::new(
                RunAndFixSqlAgentError::SqlExecutionError.to_string(),
                format!("An error occured while executing the SQL: {}", e),
            ));
        }
    };

    let data_metadata = match process_data_metadata(&data).await {
        Ok(data_metadata) => data_metadata,
        Err(e) => {
            return Err(e);
        }
    };

    Ok(DataObject {
        data,
        data_metadata,
    })
}

async fn process_data_metadata(
    data: &Vec<IndexMap<String, DataType>>,
) -> Result<DataMetadataJsonBody, ErrorNode> {
    if data.is_empty() {
        return Ok(DataMetadataJsonBody {
            column_count: 0,
            row_count: 0,
            column_metadata: vec![],
        });
    }

    let first_row = &data[0];
    let columns: Vec<_> = first_row.keys().cloned().collect();

    let column_metadata: Vec<_> = columns
        .par_iter() // Parallel iterator
        .map(|column_name| {
            let mut unique_values = Vec::with_capacity(MAX_UNIQUE_VALUES);
            let mut min_value = None;
            let mut max_value = None;
            let mut unique_values_exceeded = false;
            let mut is_date_type = false;
            let mut min_value_str: Option<String> = None;
            let mut max_value_str: Option<String> = None;

            for row in data {
                if let Some(value) = row.get(column_name) {
                    if !unique_values_exceeded && unique_values.len() < MAX_UNIQUE_VALUES {
                        if !unique_values.iter().any(|x| x == value) {
                            unique_values.push(value.clone());
                        }
                    } else {
                        unique_values_exceeded = true;
                    }

                    // Update min/max for numeric types
                    match value {
                        DataType::Int8(Some(n)) => {
                            let n = *n as f64;
                            min_value = Some(min_value.map_or(n, |min: f64| min.min(n)));
                            max_value = Some(max_value.map_or(n, |max: f64| max.max(n)));
                        }
                        DataType::Int4(Some(n)) => {
                            let n = *n as f64;
                            min_value = Some(min_value.map_or(n, |min: f64| min.min(n)));
                            max_value = Some(max_value.map_or(n, |max: f64| max.max(n)));
                        }
                        DataType::Int2(Some(n)) => {
                            let n = *n as f64;
                            min_value = Some(min_value.map_or(n, |min: f64| min.min(n)));
                            max_value = Some(max_value.map_or(n, |max: f64| max.max(n)));
                        }
                        DataType::Float4(Some(n)) => {
                            let n = *n as f64;
                            min_value = Some(min_value.map_or(n, |min: f64| min.min(n)));
                            max_value = Some(max_value.map_or(n, |max: f64| max.max(n)));
                        }
                        DataType::Float8(Some(n)) => {
                            let n = *n as f64;
                            min_value = Some(min_value.map_or(n, |min: f64| min.min(n)));
                            max_value = Some(max_value.map_or(n, |max: f64| max.max(n)));
                        }
                        DataType::Date(Some(date)) => {
                            is_date_type = true;
                            let date_str = date.to_string();
                            min_value = match min_value {
                                None => Some(date_str.parse::<f64>().unwrap_or(0.0)),
                                Some(_) => None, // Clear numeric min/max since we'll use strings
                            };
                            max_value = None;
                            if let Some(current_min) = &min_value_str {
                                if date_str < *current_min {
                                    min_value_str = Some(date_str.clone());
                                }
                            } else {
                                min_value_str = Some(date_str.clone());
                            }
                            if let Some(current_max) = &max_value_str {
                                if date_str > *current_max {
                                    max_value_str = Some(date_str);
                                }
                            } else {
                                max_value_str = Some(date_str);
                            }
                        }
                        DataType::Timestamp(Some(ts)) => {
                            is_date_type = true;
                            let ts_str = ts.to_string();
                            min_value = match min_value {
                                None => Some(ts_str.parse::<f64>().unwrap_or(0.0)),
                                Some(_) => None,
                            };
                            max_value = None;
                            if let Some(current_min) = &min_value_str {
                                if ts_str < *current_min {
                                    min_value_str = Some(ts_str.clone());
                                }
                            } else {
                                min_value_str = Some(ts_str.clone());
                            }
                            if let Some(current_max) = &max_value_str {
                                if ts_str > *current_max {
                                    max_value_str = Some(ts_str);
                                }
                            } else {
                                max_value_str = Some(ts_str);
                            }
                        }
                        DataType::Timestamptz(Some(ts)) => {
                            is_date_type = true;
                            let ts_str = ts.naive_utc().to_string();
                            min_value = match min_value {
                                None => Some(ts_str.parse::<f64>().unwrap_or(0.0)),
                                Some(_) => None,
                            };
                            max_value = None;
                            if let Some(current_min) = &min_value_str {
                                if ts_str < *current_min {
                                    min_value_str = Some(ts_str.clone());
                                }
                            } else {
                                min_value_str = Some(ts_str.clone());
                            }
                            if let Some(current_max) = &max_value_str {
                                if ts_str > *current_max {
                                    max_value_str = Some(ts_str);
                                }
                            } else {
                                max_value_str = Some(ts_str);
                            }
                        }
                        _ => {}
                    }
                }
            }

            let column_type = first_row.get(column_name).unwrap();
            ColumnMetadata {
                name: column_name.clone(),
                type_: column_type.to_string(),
                simple_type: column_type.simple_type(),
                unique_values: if !unique_values_exceeded {
                    unique_values.len() as i32
                } else {
                    MAX_UNIQUE_VALUES as i32
                },
                min_value: if is_date_type {
                    min_value_str.map(MinMaxValue::String)
                } else {
                    min_value.map(MinMaxValue::Number)
                },
                max_value: if is_date_type {
                    max_value_str.map(MinMaxValue::String)
                } else {
                    max_value.map(MinMaxValue::Number)
                },
            }
        })
        .collect();

    Ok(DataMetadataJsonBody {
        column_count: first_row.len() as i32,
        row_count: data.len() as i32,
        column_metadata,
    })
}

fn create_fix_sql_messages(
    sql: &String,
    error: &String,
    dataset: &String,
) -> Vec<PromptNodeMessage> {
    vec![
        PromptNodeMessage {
            role: "system".to_string(),
            content: format!(
                "You are a SQL expert. Fix the SQL query based on the error message. Dataset schema: {}",
                dataset
            ),
        },
        PromptNodeMessage {
            role: "user".to_string(),
            content: format!(
                "SQL Query:\n{}\n\nError Message:\n{}\n\nPlease provide the corrected SQL query.",
                sql, error
            ),
        },
    ]
}

async fn send_message(
    name: String,
    value: Value,
    output_sender: mpsc::Sender<Value>,
) -> Result<(), ErrorNode> {
    match output_sender
        .send(json!({
            "name": name,
            "value": value
        }))
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(ErrorNode::new(
                e.to_string(),
                "Failed to send message".to_string(),
            ));
        }
    }
}
