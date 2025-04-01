use anyhow::{anyhow, Result};
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::models::User,
    routes::ws::{
        data_sources::data_sources_router::{DataSourceEvent, DataSourceRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::data_source_utils::data_source_utils::get_data_source_state;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetDataSourceRequest {
    pub id: Uuid,
}

pub async fn get_data_source(user: &User, req: GetDataSourceRequest) -> Result<()> {
    let list_data_sources_res = match get_data_source_state(&user.id, req.id).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting data source: {}", e);
            let err = anyhow!("Error getting data source: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::DataSources(DataSourceRoute::Get),
                WsEvent::DataSources(DataSourceEvent::GetDataSource),
                WsErrorCode::InternalServerError,
                "Failed to get data source.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let list_data_sources_message = WsResponseMessage::new(
        WsRoutes::DataSources(DataSourceRoute::Get),
        WsEvent::DataSources(DataSourceEvent::GetDataSource),
        list_data_sources_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &list_data_sources_message).await {
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
