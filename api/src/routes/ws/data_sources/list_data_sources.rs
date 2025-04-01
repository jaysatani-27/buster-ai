use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::{DataSourceType, UserOrganizationRole},
        lib::get_pg_pool,
        models::User,
        schema::{data_sources, organizations, users_to_organizations},
    },
    routes::ws::{
        data_sources::data_sources_router::{DataSourceEvent, DataSourceRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListDataSourcesRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListDataSourceObject {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: DataSourceType,
    pub updated_at: DateTime<Utc>,
}

pub async fn list_data_sources(user: &User, req: ListDataSourcesRequest) -> Result<()> {
    let list_data_sources_res =
        match list_data_sources_handler(&user.id, req.page, req.page_size).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error getting threads: {}", e);
                let err = anyhow!("Error getting threads: {}", e);
                send_sentry_error(&e.to_string(), Some(&user.id));
                send_error_message(
                    &user.id.to_string(),
                    WsRoutes::DataSources(DataSourceRoute::List),
                    WsEvent::DataSources(DataSourceEvent::ListDataSources),
                    WsErrorCode::InternalServerError,
                    "Failed to list data sources.".to_string(),
                    user,
                )
                .await?;
                return Err(err);
            }
        };

    let list_data_sources_message = WsResponseMessage::new(
        WsRoutes::DataSources(DataSourceRoute::List),
        WsEvent::DataSources(DataSourceEvent::ListDataSources),
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

async fn list_data_sources_handler(
    user_id: &Uuid,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<Vec<ListDataSourceObject>> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(25);

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let data_sources = match data_sources::table
        .inner_join(organizations::table.on(data_sources::organization_id.eq(organizations::id)))
        .inner_join(
            users_to_organizations::table
                .on(organizations::id.eq(users_to_organizations::organization_id)),
        )
        .select((
            data_sources::id,
            data_sources::name,
            data_sources::type_,
            data_sources::updated_at,
        ))
        .filter(users_to_organizations::user_id.eq(user_id))
        .filter(
            users_to_organizations::role
                .eq(UserOrganizationRole::WorkspaceAdmin)
                .or(users_to_organizations::role.eq(UserOrganizationRole::DataAdmin)),
        )
        .filter(users_to_organizations::deleted_at.is_null())
        .filter(data_sources::deleted_at.is_null())
        .filter(organizations::deleted_at.is_null())
        .order(data_sources::created_at.desc())
        .limit(page_size)
        .offset(page * page_size)
        .load::<(Uuid, String, DataSourceType, DateTime<Utc>)>(&mut conn)
        .await
    {
        Ok(data_sources) => data_sources,
        Err(diesel::NotFound) => return Ok(vec![]),
        Err(e) => return Err(anyhow!("Error loading data sources: {:?}", e)),
    };

    let data_sources = data_sources
        .iter()
        .map(|(id, name, type_, updated_at)| ListDataSourceObject {
            id: id.clone(),
            name: name.clone(),
            type_: type_.clone(),
            updated_at: updated_at.clone(),
        })
        .collect();

    Ok(data_sources)
}
