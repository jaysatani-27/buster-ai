use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{insert_into, update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::DatasetType,
        lib::get_pg_pool,
        models::{Dataset, User},
        schema::{dataset_columns, datasets},
    },
    routes::ws::{
        datasets::datasets_router::{DatasetEvent, DatasetRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::{sentry_utils::send_sentry_error, typesense},
        query_engine::{
            credentials::get_data_source_credentials,
            import_dataset_columns::import_dataset_columns,
        },
        user::user_info::get_user_organization_id,
    },
};

use super::dataset_utils::{
    generate_col_descriptions, generate_dataset_descriptions, get_dataset_state, DatasetState,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostDataSetReq {
    pub name: Option<String>,
    pub data_source_id: Uuid,
    pub dataset_id: Option<Uuid>,
}

pub async fn post_dataset(user: &User, req: PostDataSetReq) -> Result<()> {
    let name = req.name.unwrap_or_else(|| "Untitled Dataset".to_string());

    let post_dataset_res =
        match post_dataset_handler(&user.id, &name, &req.data_source_id, &req.dataset_id).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error posting dataset: {}", e);
                let err = anyhow!("Error posting dataset: {}", e);
                send_sentry_error(&e.to_string(), Some(&user.id));
                send_error_message(
                    &user.id.to_string(),
                    WsRoutes::Datasets(DatasetRoute::Post),
                    WsEvent::Datasets(DatasetEvent::PostDataset),
                    WsErrorCode::InternalServerError,
                    "Failed to post dataset.".to_string(),
                    user,
                )
                .await?;
                return Err(err);
            }
        };

    let post_dataset_message = WsResponseMessage::new(
        WsRoutes::Datasets(DatasetRoute::Post),
        WsEvent::Datasets(DatasetEvent::PostDataset),
        post_dataset_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &post_dataset_message).await {
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

async fn post_dataset_handler(
    user_id: &Uuid,
    name: &String,
    data_source_id: &Uuid,
    dataset_id: &Option<Uuid>,
) -> Result<DatasetState> {
    let dataset_id = match dataset_id {
        Some(dataset_id) => {
            update_dataset(user_id, dataset_id, name).await?;

            dataset_id.clone()
        }
        None => {
            let dataset_id = create_dataset(user_id, name, data_source_id).await?;

            dataset_id
        }
    };

    let dataset_state = match get_dataset_state(&dataset_id, &user_id).await {
        Ok(dataset_state) => dataset_state,
        Err(e) => return Err(anyhow!("Error getting dataset state: {}", e)),
    };

    Ok(dataset_state)
}

async fn update_dataset(user_id: &Uuid, dataset_id: &Uuid, name: &String) -> Result<()> {
    let dataset = match get_dataset_state(dataset_id, user_id).await {
        Ok(dataset) => dataset,
        Err(e) => return Err(anyhow!("Error getting dataset state: {}", e)),
    };

    let credentials = match get_data_source_credentials(
        &dataset.data_source.secret_id,
        &dataset.data_source.type_,
        false,
    )
    .await
    {
        Ok(credentials) => credentials,
        Err(e) => return Err(anyhow!("Error getting data source credentials: {}", e)),
    };

    match clear_dataset_columns(&dataset.dataset.id).await {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error clearing dataset columns: {}", e)),
    };

    match import_dataset_columns(
        &dataset.dataset.id,
        &dataset.dataset.database_name,
        &dataset.dataset.schema,
        &credentials,
        None,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error importing dataset columns: {}", e)),
    };

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    match update(datasets::table)
        .set(datasets::name.eq(name))
        .filter(datasets::id.eq(dataset_id))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error updating dataset: {}", e)),
    };

    let dataset_state = match get_dataset_state(dataset_id, user_id).await {
        Ok(dataset_state) => dataset_state,
        Err(e) => return Err(anyhow!("Error getting dataset state: {}", e)),
    };

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

    Ok(())
}

async fn create_dataset(user_id: &Uuid, name: &String, data_source_id: &Uuid) -> Result<Uuid> {
    let user_org_id = match get_user_organization_id(user_id).await {
        Ok(org_id) => org_id,
        Err(e) => return Err(anyhow!("Error getting user organization id: {}", e)),
    };

    let dataset = Dataset {
        id: Uuid::new_v4(),
        name: name.to_string(),
        data_source_id: data_source_id.to_owned(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        database_name: "".to_string(),
        when_to_use: None,
        when_not_to_use: None,
        type_: DatasetType::View,
        definition: "".to_string(),
        schema: "".to_string(),
        enabled: false,
        created_by: user_id.to_owned(),
        updated_by: user_id.to_owned(),
        deleted_at: None,
        imported: false,
        organization_id: user_org_id,
        yml_file: None,
        model: None,
        database_identifier: None,
    };

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    match insert_into(datasets::table)
        .values(&dataset)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting dataset: {}", e)),
    };

    Ok(dataset.id)
}

async fn clear_dataset_columns(dataset_id: &Uuid) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    match diesel::delete(dataset_columns::table)
        .filter(dataset_columns::dataset_id.eq(dataset_id))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error deleting dataset columns: {}", e)),
    };

    Ok(())
}
