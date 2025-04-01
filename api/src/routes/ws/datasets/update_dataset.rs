use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{AsChangeset, ExpressionMethods};
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::DatasetType,
        lib::get_pg_pool,
        models::User,
        schema::{dataset_columns, datasets},
    },
    routes::ws::{
        datasets::datasets_router::{DatasetEvent, DatasetRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::sentry_utils::send_sentry_error,
        query_engine::{
            credentials::get_data_source_credentials,
            import_dataset_columns::import_dataset_columns, write_query_engine::write_query_engine,
        },
    },
};

use super::dataset_utils::{
    generate_col_descriptions, generate_dataset_descriptions, get_dataset_state,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateDatasetDefReq {
    pub sql: String,
    pub schema: String,
    #[serde(rename = "identifier")]
    pub database_name: String,
    #[serde(rename = "type")]
    pub type_: DatasetType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateDatasetReq {
    pub id: Uuid,
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub when_to_use: Option<String>,
    pub when_not_to_use: Option<String>,
    pub data_source_id: Option<Uuid>,
    pub dataset_definition: Option<UpdateDatasetDefReq>,
}

pub async fn update_dataset(user: &User, req: UpdateDatasetReq) -> Result<()> {
    match update_dataset_handler(
        &user.id,
        &req.id,
        req.name,
        req.enabled,
        req.when_to_use,
        req.when_not_to_use,
        req.dataset_definition,
        req.data_source_id,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error updating dataset: {}", e);
            let err = anyhow!("Error updating dataset: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Datasets(DatasetRoute::Update),
                WsEvent::Datasets(DatasetEvent::UpdateDataset),
                WsErrorCode::InternalServerError,
                "Failed to update dataset.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let dataset_state = match get_dataset_state(&req.id, &user.id).await {
        Ok(dataset_state) => dataset_state,
        Err(e) => return Err(anyhow!("Error getting dataset state: {}", e)),
    };

    let update_dataset_message = WsResponseMessage::new(
        WsRoutes::Datasets(DatasetRoute::Update),
        WsEvent::Datasets(DatasetEvent::UpdateDataset),
        dataset_state,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &update_dataset_message).await {
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

#[derive(Debug, AsChangeset)]
#[diesel(table_name = datasets)]
pub struct DatasetChangeset {
    pub name: Option<String>,
    pub database_name: Option<String>,
    pub enabled: Option<bool>,
    pub when_to_use: Option<String>,
    pub when_not_to_use: Option<String>,
    pub definition: Option<String>,
    pub type_: Option<DatasetType>,
    pub schema: Option<String>,
    pub data_source_id: Option<Uuid>,
}

async fn update_dataset_handler(
    user_id: &Uuid,
    id: &Uuid,
    name: Option<String>,
    enabled: Option<bool>,
    when_to_use: Option<String>,
    when_not_to_use: Option<String>,
    dataset_def: Option<UpdateDatasetDefReq>,
    data_source_id: Option<Uuid>,
) -> Result<()> {
    let mut dataset_changeset = DatasetChangeset {
        database_name: None,
        enabled,
        when_to_use,
        when_not_to_use,
        definition: None,
        type_: None,
        schema: None,
        name,
        data_source_id,
    };

    let dataset_state = match get_dataset_state(id, user_id).await {
        Ok(dataset_state) => dataset_state,
        Err(e) => return Err(anyhow!("Error getting dataset state: {}", e)),
    };

    if !dataset_state.dataset.imported {
        if let Some(dataset_def) = dataset_def {
            if !dataset_state.dataset.definition.is_empty() {
                match clean_up_view(
                    id,
                    &dataset_state.dataset.type_,
                    &dataset_state.dataset.schema,
                    &dataset_state.dataset.database_name,
                )
                .await
                {
                    Ok(_) => (),
                    Err(e) => return Err(anyhow!("Error dropping view: {}", e)),
                }
            };

            match create_view(
                &id,
                &dataset_def.sql,
                &dataset_def.type_,
                &dataset_def.schema,
                &dataset_def.database_name,
            )
            .await
            {
                Ok(_) => (),
                Err(e) => return Err(anyhow!("Error creating view: {}", e)),
            }

            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => return Err(anyhow!("Error getting connection: {}", e)),
            };

            match diesel::update(dataset_columns::table)
                .filter(dataset_columns::dataset_id.eq(&id))
                .set(dataset_columns::deleted_at.eq(Some(Utc::now())))
                .execute(&mut conn)
                .await
            {
                Ok(_) => (),
                Err(e) => return Err(anyhow!("Error deleting dataset columns: {}", e)),
            };

            let credentials = match get_data_source_credentials(
                &dataset_state.data_source.secret_id,
                &dataset_state.data_source.type_,
                false,
            )
            .await
            {
                Ok(credentials) => credentials,
                Err(e) => return Err(anyhow!("Error getting data source credentials: {}", e)),
            };

            match import_dataset_columns(
                &dataset_state.dataset.id,
                &dataset_def.database_name,
                &dataset_def.schema,
                &credentials,
                None,
            )
            .await
            {
                Ok(_) => (),
                Err(e) => {
                    return Err(anyhow!("Error importing dataset columns: {}", e));
                }
            }

            dataset_changeset.database_name = Some(dataset_def.database_name);
            dataset_changeset.type_ = Some(dataset_def.type_);
            dataset_changeset.definition = Some(dataset_def.sql);
            dataset_changeset.schema = Some(dataset_def.schema);
        }
    }

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection: {}", e)),
    };

    match diesel::update(datasets::table)
        .filter(datasets::id.eq(&id))
        .set(&dataset_changeset)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error updating dataset: {}", e)),
    };

    let dataset_state = match get_dataset_state(id, user_id).await {
        Ok(dataset_state) => dataset_state,
        Err(e) => return Err(anyhow!("Error getting dataset state: {}", e)),
    };

    if dataset_changeset.definition.is_some() {
        let dataset_state = Arc::new(dataset_state.clone());
        let user_id = Arc::new(user_id.clone());

        let col_descriptions_handle = {
            let dataset_state = Arc::clone(&dataset_state);
            let user_id = Arc::clone(&user_id);
            tokio::spawn(async move { generate_col_descriptions(dataset_state, user_id).await })
        };

        let dataset_description_handle = {
            let dataset_state = dataset_state.clone();

            tokio::spawn(async move { generate_dataset_descriptions(dataset_state, user_id).await })
        };

        let (col_descriptions_res, dataset_description_res) =
            match tokio::try_join!(col_descriptions_handle, dataset_description_handle,) {
                Ok((col_descriptions_res, dataset_description_res)) => {
                    (col_descriptions_res, dataset_description_res)
                }
                Err(e) => {
                    return Err(anyhow!(
                        "Error generating col descriptions or dataset descriptions: {}",
                        e
                    ))
                }
            };

        match col_descriptions_res {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Error generating col descriptions: {}", e)),
        };

        match dataset_description_res {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Error generating dataset descriptions: {}", e)),
        };
    }

    Ok(())
}

async fn create_view(
    dataset_id: &Uuid,
    sql: &String,
    type_: &DatasetType,
    schema: &String,
    database_name: &String,
) -> Result<()> {
    let view_name = format!("{}.{}", schema, database_name);
    let view_sql = match type_ {
        DatasetType::View => format!("CREATE OR REPLACE VIEW {} AS {}", view_name, sql),
        DatasetType::MaterializedView => {
            format!("CREATE MATERIALIZED VIEW {} AS {}", view_name, sql)
        }
        _ => return Err(anyhow!("Invalid dataset type for view creation")),
    };

    match write_query_engine(dataset_id, &view_sql).await {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Failed to create view: {}", e)),
    };

    Ok(())
}

async fn clean_up_view(
    dataset_id: &Uuid,
    type_: &DatasetType,
    schema: &String,
    database_name: &String,
) -> Result<()> {
    let view_name = format!("{}.{}", schema, database_name);
    let drop_sql = match type_ {
        DatasetType::View => format!("DROP VIEW IF EXISTS {}", view_name),
        DatasetType::MaterializedView => format!("DROP MATERIALIZED VIEW IF EXISTS {}", view_name),
        _ => return Err(anyhow!("Invalid dataset type for view dropping")),
    };

    match write_query_engine(dataset_id, &drop_sql).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Failed to drop view: {}", e)),
    }
}
