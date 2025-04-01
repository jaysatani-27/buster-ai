use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{update, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::{DataSourceType, UserOrganizationRole},
        lib::get_pg_pool,
        models::User,
        schema::{data_sources, users_to_organizations},
    },
    routes::ws::{
        data_sources::data_sources_router::{DataSourceEvent, DataSourceRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::{sentry_utils::send_sentry_error, supabase_vault::update_secret},
        query_engine::{
            credentials::Credential, test_data_source_connections::test_data_source_connection,
        },
    },
};

use super::data_source_utils::data_source_utils::{get_data_source_state, DataSourceState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateDataSourceReq {
    pub id: Uuid,
    pub credentials: Credential,
}

pub async fn update_data_source(user: &User, req: UpdateDataSourceReq) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting postgres connection: {}", e));
        }
    };

    let db_type = match data_sources::table
        .filter(data_sources::id.eq(&req.id))
        .select(data_sources::type_)
        .first::<String>(&mut conn)
        .await
    {
        Ok(db_type) => DataSourceType::from_str(&db_type).unwrap(),
        Err(diesel::NotFound) => {
            return Err(anyhow!("Data source not found"));
        }
        Err(e) => {
            return Err(anyhow!("Error getting data source type: {}", e));
        }
    };

    match test_data_source_connection(&db_type, &req.credentials).await {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Unabled to connect to the data source: {}", e));
        }
    };

    let post_data_source_res =
        match update_data_source_handler(&user.id, req.id, req.credentials).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error getting data source: {}", e);
                let err = anyhow!("Error getting data source: {}", e);
                send_sentry_error(&e.to_string(), Some(&user.id));
                send_error_message(
                    &user.id.to_string(),
                    WsRoutes::DataSources(DataSourceRoute::Update),
                    WsEvent::DataSources(DataSourceEvent::UpdateDataSource),
                    WsErrorCode::InternalServerError,
                    "Failed to get data source.".to_string(),
                    user,
                )
                .await?;
                return Err(err);
            }
        };

    let post_data_source_message = WsResponseMessage::new(
        WsRoutes::DataSources(DataSourceRoute::Update),
        WsEvent::DataSources(DataSourceEvent::UpdateDataSource),
        post_data_source_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &post_data_source_message).await {
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

async fn update_data_source_handler(
    user_id: &Uuid,
    id: Uuid,
    credentials: Credential,
) -> Result<DataSourceState> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting postgres connection: {}", e));
        }
    };

    let secret_id = match data_sources::table
        .inner_join(
            users_to_organizations::table
                .on(data_sources::organization_id.eq(users_to_organizations::organization_id)),
        )
        .select(data_sources::secret_id)
        .filter(data_sources::deleted_at.is_null())
        .filter(
            users_to_organizations::role
                .eq(UserOrganizationRole::WorkspaceAdmin)
                .or(users_to_organizations::role.eq(UserOrganizationRole::DataAdmin)),
        )
        .filter(users_to_organizations::user_id.eq(user_id))
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(secret_id) => secret_id,
        Err(diesel::NotFound) => {
            return Err(anyhow!("User does not have appropriate permissions"));
        }
        Err(e) => {
            return Err(anyhow!("Error getting user organization: {}", e));
        }
    };

    let secret_value = match serde_json::to_string(&credentials) {
        Ok(secret_value) => secret_value,
        Err(e) => {
            return Err(anyhow!("Error converting credentials to string: {}", e));
        }
    };

    let update_secret_handle =
        tokio::spawn(async move { update_secret(&secret_id, &secret_value).await });

    let update_data_source_handle = {
        let user_id = user_id.clone();
        let id = id.clone();
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => {
                    return Err(anyhow!("Error getting postgres connection: {}", e));
                }
            };

            match update(data_sources::table)
                .set((
                    data_sources::updated_by.eq(&user_id),
                    data_sources::updated_at.eq(Utc::now()),
                ))
                .filter(data_sources::id.eq(&id))
                .execute(&mut conn)
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    return Err(anyhow!("Error updating data source: {}", e));
                }
            };

            Ok(())
        })
    };

    match tokio::try_join!(update_secret_handle, update_data_source_handle) {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error updating data source: {}", e));
        }
    };

    let data_source_state = match get_data_source_state(user_id, id).await {
        Ok(data_source_state) => data_source_state,
        Err(e) => {
            return Err(anyhow!("Error getting data source state: {}", e));
        }
    };

    Ok(data_source_state)
}
