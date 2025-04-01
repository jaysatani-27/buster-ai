use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{AsChangeset, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::StoredValuesStatus,
        lib::get_pg_pool,
        models::User,
        schema::dataset_columns,
    },
    routes::ws::{
        datasets::datasets_router::{DatasetEvent, DatasetRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::{sentry_utils::send_sentry_error, typesense},
        query_engine::values_index::start_stored_values_sync,
    },
};

use super::dataset_utils::get_dataset_state_from_col_id;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateDatasetColumnReq {
    pub id: Uuid,
    pub description: Option<String>,
    pub stored_values: Option<bool>,
}

pub async fn update_dataset_column(user: &User, req: UpdateDatasetColumnReq) -> Result<()> {
    match update_dataset_column_handler(&req.id, req.description, req.stored_values).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error updating dataset column: {}", e);
            let err = anyhow!("Error updating dataset column: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Datasets(DatasetRoute::UpdateColumn),
                WsEvent::Datasets(DatasetEvent::UpdateDatasetColumn),
                WsErrorCode::InternalServerError,
                "Failed to update dataset column.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let dataset_state = match get_dataset_state_from_col_id(&req.id, &user.id).await {
        Ok(dataset_state) => dataset_state,
        Err(e) => {
            tracing::error!("Error getting dataset state: {}", e);
            let err = anyhow!("Error getting dataset state: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(err);
        }
    };

    let update_dataset_column_message = WsResponseMessage::new(
        WsRoutes::Datasets(DatasetRoute::UpdateColumn),
        WsEvent::Datasets(DatasetEvent::UpdateDatasetColumn),
        dataset_state,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &update_dataset_column_message).await {
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
#[diesel(table_name = dataset_columns)]
pub struct DatasetColumnChangeset {
    pub description: Option<String>,
    pub stored_values: Option<bool>,
    pub stored_values_status: Option<StoredValuesStatus>,
    pub stored_values_error: Option<String>,
    pub stored_values_count: Option<i64>,
    pub stored_values_last_synced: Option<chrono::DateTime<Utc>>,
    pub updated_at: chrono::DateTime<Utc>,
}

async fn update_dataset_column_handler(
    dataset_column_id: &Uuid,
    description: Option<String>,
    stored_values: Option<bool>,
) -> Result<()> {
    let stored_values_status = if let Some(true) = stored_values {
        {
            let dataset_column_id = dataset_column_id.clone();
            tokio::spawn(async move {
                match start_stored_values_sync(&dataset_column_id).await {
                    Ok(_) => (),
                    Err(e) => return Err(anyhow!("Error starting stored values sync: {}", e)),
                }

                Ok(())
            });
        }

        Some(StoredValuesStatus::Syncing)
    } else if let Some(false) = stored_values {
        match delete_stored_values(dataset_column_id).await {
            Ok(_) => None,
            Err(e) => return Err(anyhow!("Error deleting stored values: {}", e)),
        }
    } else {
        None
    };

    let dataset_column_changeset = DatasetColumnChangeset {
        description,
        stored_values,
        stored_values_status,
        stored_values_error: None,
        stored_values_count: None,
        stored_values_last_synced: None,
        updated_at: Utc::now(),
    };

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    match diesel::update(dataset_columns::table)
        .filter(dataset_columns::id.eq(dataset_column_id))
        .set(&dataset_column_changeset)
        .execute(&mut conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Error updating dataset column: {}", e)),
    }
}

async fn delete_stored_values(dataset_column_id: &Uuid) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    let dataset_id = match dataset_columns::table
        .select(dataset_columns::dataset_id)
        .filter(dataset_columns::id.eq(dataset_column_id))
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(dataset_id) => dataset_id,
        Err(e) => return Err(anyhow!("Error getting dataset id: {}", e)),
    };

    match typesense::delete_collection(
        &format!("dataset_index_{}", dataset_id),
        &format!("dataset_column_id:={}", dataset_column_id),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error deleting stored values: {}", e)),
    };

    Ok(())
}
