use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{update, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::UserOrganizationRole,
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
    utils::clients::{sentry_utils::send_sentry_error, supabase_vault::delete_secret},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteDataSourceReq {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteDataSourceRes {
    pub id: Uuid,
}

pub async fn delete_data_source(user: &User, req: DeleteDataSourceReq) -> Result<()> {
    match delete_data_source_handler(&user.id, req.id).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting data source: {}", e);
            let err = anyhow!("Error getting data source: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::DataSources(DataSourceRoute::Delete),
                WsEvent::DataSources(DataSourceEvent::DeleteDataSource),
                WsErrorCode::InternalServerError,
                "Failed to get data source.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let delete_data_source_res = DeleteDataSourceRes { id: req.id };

    let post_data_source_message = WsResponseMessage::new(
        WsRoutes::DataSources(DataSourceRoute::Delete),
        WsEvent::DataSources(DataSourceEvent::DeleteDataSource),
        delete_data_source_res,
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

async fn delete_data_source_handler(user_id: &Uuid, id: Uuid) -> Result<()> {
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

    let delete_secret_handle = { tokio::spawn(async move { delete_secret(&secret_id).await }) };

    let delete_data_source_handle = {
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
                    data_sources::deleted_at.eq(Utc::now()),
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

    match tokio::try_join!(delete_secret_handle, delete_data_source_handle) {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error updating data source: {}", e));
        }
    };

    Ok(())
}
