use anyhow::{anyhow, Result};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use uuid::Uuid;

use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::UserOrganizationRole,
        lib::{get_pg_pool, ColumnMetadata, DataMetadataJsonBody, MinMaxValue},
        models::User,
        schema::{data_sources, datasets, users_to_organizations},
    },
    routes::ws::{
        sql::sql_router::{SqlEvent, SqlRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::sentry_utils::send_sentry_error,
        query_engine::{
            data_types::DataType,
            query_engine::{modeling_query_engine, query_engine},
        },
        security::dataset_security::has_dataset_access,
    },
};

const MAX_UNIQUE_VALUES: usize = 100;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunSqlRequest {
    pub dataset_id: Option<Uuid>,
    pub data_source_id: Option<Uuid>,
    pub sql: String,
}

pub async fn run_sql(user: &User, req: RunSqlRequest) -> Result<()> {
    let run_sql_res =
        match run_sql_handler(&req.sql, &req.data_source_id, &req.dataset_id, &user.id).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error running SQL: {}", e);
                let err = anyhow!("Error running SQL: {}", e);
                send_sentry_error(&e.to_string(), Some(&user.id));
                send_error_message(
                    &user.id.to_string(),
                    WsRoutes::Sql(SqlRoute::Run),
                    WsEvent::Sql(SqlEvent::RunSql),
                    WsErrorCode::InternalServerError,
                    e.to_string(),
                    user,
                )
                .await?;
                return Err(err);
            }
        };

    let run_sql_message = WsResponseMessage::new(
        WsRoutes::Sql(SqlRoute::Run),
        WsEvent::Sql(SqlEvent::RunSql),
        run_sql_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &run_sql_message).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error sending ws message: {}", e);
            let err = anyhow!("Error sending ws message: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(err);
        }
    }

    Ok(())
}

async fn run_sql_handler(
    sql: &String,
    data_source_id: &Option<Uuid>,
    dataset_id: &Option<Uuid>,
    user_id: &Uuid,
) -> Result<DataObject> {
    if let Some(data_source_id) = data_source_id {
        return run_data_source_sql_handler(sql, &data_source_id, user_id).await;
    } else if let Some(dataset_id) = dataset_id {
        return run_dataset_sql_handler(sql, &dataset_id, user_id).await;
    } else {
        return Err(anyhow!("No data source or dataset id provided"));
    }
}

async fn run_dataset_sql_handler(
    sql: &String,
    dataset_id: &Uuid,
    user_id: &Uuid,
) -> Result<DataObject> {
    let has_dataset_access = match has_dataset_access(user_id, dataset_id).await {
        Ok(has_access) => has_access,
        Err(e) => return Err(e),
    };

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {}", e)),
    };

    let is_org_admin_or_owner = datasets::table
        .inner_join(data_sources::table.on(datasets::data_source_id.eq(data_sources::id)))
        .inner_join(
            users_to_organizations::table
                .on(data_sources::organization_id.eq(users_to_organizations::organization_id)),
        )
        .filter(users_to_organizations::user_id.eq(user_id))
        .filter(
            users_to_organizations::role
                .eq(UserOrganizationRole::WorkspaceAdmin)
                .or(users_to_organizations::role.eq(UserOrganizationRole::DataAdmin)),
        )
        .select(users_to_organizations::user_id)
        .first::<Uuid>(&mut conn)
        .await
        .is_ok();

    let results = if is_org_admin_or_owner || has_dataset_access {
        match fetch_data(sql, dataset_id).await {
            Ok(results) => results,
            Err(e) => return Err(e),
        }
    } else {
        return Err(anyhow!("User does not have access to this dataset"));
    };

    Ok(results)
}

#[derive(Debug, Serialize)]
pub struct DataObject {
    pub data: Vec<IndexMap<String, DataType>>,
    pub data_metadata: DataMetadataJsonBody,
}

pub async fn fetch_data(sql: &String, dataset_id: &Uuid) -> Result<DataObject> {
    let data = match query_engine(&dataset_id, &sql).await {
        Ok(data) => data,
        Err(e) => {
            return Err(anyhow!(e));
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
) -> Result<DataMetadataJsonBody> {
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

async fn run_data_source_sql_handler(
    sql: &String,
    data_source_id: &Uuid,
    user_id: &Uuid,
) -> Result<DataObject> {
    let data = match modeling_query_engine(data_source_id, sql, user_id).await {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let data_metadata = match process_data_metadata(&data).await {
        Ok(data_metadata) => data_metadata,
        Err(e) => return Err(e),
    };

    let data_object = DataObject {
        data,
        data_metadata,
    };

    Ok(data_object)
}
