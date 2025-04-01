use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{insert_into, BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::{DataSourceOnboardingStatus, DataSourceType, UserOrganizationRole},
        lib::get_pg_pool,
        models::{DataSource, User},
        schema::{data_sources, users_to_organizations},
    },
    routes::ws::{
        data_sources::data_sources_router::{DataSourceEvent, DataSourceRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::{sentry_utils::send_sentry_error, supabase_vault::create_secret},
        query_engine::{
            credentials::Credential, import_datasets::import_datasets,
            test_data_source_connections::test_data_source_connection,
        },
    },
};

use super::data_source_utils::data_source_utils::{get_data_source_state, DataSourceState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostDataSourceReq {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: DataSourceType,
    pub credentials: Credential,
}

pub async fn post_data_source(user: &User, req: PostDataSourceReq) -> Result<()> {
    match test_data_source_connection(&req.type_, &req.credentials).await {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Unabled to connect to the data source: {}", e));
        }
    };

    let post_data_source_res =
        match post_data_source_handler(&user.id, req.name, req.type_, &req.credentials).await {
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

    match import_datasets(
        &req.credentials,
        &post_data_source_res.id,
        &user.id,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Error getting data source: {}", e));
        }
    };

    let post_data_source_message = WsResponseMessage::new(
        WsRoutes::DataSources(DataSourceRoute::Get),
        WsEvent::DataSources(DataSourceEvent::GetDataSource),
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

async fn post_data_source_handler(
    user_id: &Uuid,
    name: String,
    type_: DataSourceType,
    credentials: &Credential,
) -> Result<DataSourceState> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting postgres connection: {}", e));
        }
    };

    let organization_id = match users_to_organizations::table
        .select(users_to_organizations::organization_id)
        .filter(users_to_organizations::deleted_at.is_null())
        .filter(
            users_to_organizations::role
                .eq(UserOrganizationRole::WorkspaceAdmin)
                .or(users_to_organizations::role.eq(UserOrganizationRole::DataAdmin)),
        )
        .filter(users_to_organizations::user_id.eq(user_id))
        .first::<Uuid>(&mut conn)
        .await
    {
        Ok(user_org) => user_org,
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

    let secret_id = match create_secret(&secret_value).await {
        Ok(secret_id) => secret_id,
        Err(e) => {
            return Err(anyhow!("Error creating secret: {}", e));
        }
    };

    let data_source = DataSource {
        id: Uuid::new_v4(),
        name,
        type_,
        secret_id,
        organization_id,
        created_by: *user_id,
        updated_by: *user_id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        onboarding_status: DataSourceOnboardingStatus::NotStarted,
        onboarding_error: None,
        env: "dev".to_string(),
    };

    match insert_into(data_sources::table)
        .values(&data_source)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error inserting data source: {}", e);
            return Err(anyhow!("Error inserting data source: {}", e));
        }
    };

    let data_source_state = match get_data_source_state(user_id, data_source.id).await {
        Ok(data_source_state) => data_source_state,
        Err(e) => {
            return Err(anyhow!("Error getting data source state: {}", e));
        }
    };

    Ok(data_source_state)
}
