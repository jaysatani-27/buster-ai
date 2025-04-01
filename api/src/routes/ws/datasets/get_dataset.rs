use anyhow::{anyhow, Result};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::models::User,
    routes::ws::{
        datasets::datasets_router::{DatasetEvent, DatasetRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::dataset_utils::{get_dataset_state, DatasetState};

use crate::utils::query_engine::{data_types::DataType, query_engine::query_engine};
use indexmap::IndexMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDatasetReq {
    pub id: Uuid,
}

#[derive(Serialize, Clone)]
pub struct GetDatasetResponse {
    #[serde(flatten)]
    pub dataset_state: DatasetState,
    pub data: Vec<IndexMap<String, DataType>>,
}

pub async fn get_dataset(user: &User, req: GetDatasetReq) -> Result<()> {
    let dataset_state = match get_dataset_state(&req.id, &user.id).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting dataset: {}", e);
            let err = anyhow!("Error getting dataset: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Datasets(DatasetRoute::Get),
                WsEvent::Datasets(DatasetEvent::GetDataset),
                WsErrorCode::InternalServerError,
                "Failed to get dataset.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let data = if !dataset_state.dataset.database_name.is_empty() {
        let schema = dataset_state.dataset.schema.clone();
        let database_name = dataset_state.dataset.database_name.clone();
        let sql = format!("SELECT * FROM {}.{} LIMIT 25", schema, database_name);
        match query_engine(&req.id, &sql).await {
            Ok(data) => data,
            Err(e) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let get_dataset_response = GetDatasetResponse {
        dataset_state,
        data,
    };

    let get_dataset_message = WsResponseMessage::new(
        WsRoutes::Datasets(DatasetRoute::Get),
        WsEvent::Datasets(DatasetEvent::GetDataset),
        get_dataset_response,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &get_dataset_message).await {
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
