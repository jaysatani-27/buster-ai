use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        lib::get_pg_pool,
        models::User,
        schema::datasets,
    },
    routes::ws::{
        datasets::datasets_router::{DatasetEvent, DatasetRoute},
        ws::{WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::send_ws_message,
    },
    utils::clients::sentry_utils::send_sentry_error,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDatasetRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDatasetResponse {
    pub ids: Vec<Uuid>,
}

pub async fn delete_dataset(user: &User, req: DeleteDatasetRequest) -> Result<()> {
    let response = match delete_dataset_handler(req.ids).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Error deleting dataset: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    };

    let delete_dataset_message = WsResponseMessage::new(
        WsRoutes::Datasets(DatasetRoute::Delete),
        WsEvent::Datasets(DatasetEvent::DeleteDatasets),
        response,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &delete_dataset_message).await {
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

async fn delete_dataset_handler(ids: Vec<Uuid>) -> Result<DeleteDatasetResponse> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting connection: {}", e));
        }
    };

    match update(datasets::table)
        .filter(datasets::id.eq_any(&ids))
        .set(datasets::deleted_at.eq(Some(Utc::now())))
        .execute(&mut conn)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow!("Error updating datasets: {}", e));
        }
    };

    Ok(DeleteDatasetResponse { ids })
}
