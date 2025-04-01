use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::AssetPermissionRole,
        lib::{get_pg_pool, get_sqlx_pool},
        models::User,
        schema::dashboards,
    },
    routes::ws::{
        dashboards::dashboards_router::{DashboardEvent, DashboardRoute},
        ws::{WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::send_ws_message,
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::dashboard_utils::get_bulk_user_dashboard_permission;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteDashboardRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Serialize, Debug, Clone)]
pub struct DeleteDashboardResponse {
    pub ids: Vec<Uuid>,
}

pub async fn delete_dashboard(user: &User, req: DeleteDashboardRequest) -> Result<()> {
    let response = match delete_dashboard_handler(&user.id, req.ids).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Error deleting dashboard: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    };

    let delete_dashboard_message = WsResponseMessage::new(
        WsRoutes::Dashboards(DashboardRoute::Delete),
        WsEvent::Dashboards(DashboardEvent::DeleteDashboard),
        response,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &delete_dashboard_message).await {
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

async fn delete_dashboard_handler(
    user_id: &Uuid,
    ids: Vec<Uuid>,
) -> Result<DeleteDashboardResponse> {
    let roles = match get_bulk_user_dashboard_permission(&user_id, &ids).await {
        Ok(roles) => roles,
        Err(e) => return Err(anyhow!("Error getting dashboard permissions: {:?}", e)),
    };

    let filtered_ids_to_delete: Vec<Uuid> = ids
        .into_iter()
        .filter(|id| match roles.get(id) {
            Some(role) if *role != AssetPermissionRole::Viewer => true,
            _ => false,
        })
        .collect();

    let update_dashboard_record_handle = {
        let ids = filtered_ids_to_delete.clone();
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => {
                    return Err(anyhow!("Error getting connection: {}", e));
                }
            };

            let _ = match update(dashboards::table)
                .filter(dashboards::id.eq_any(&ids))
                .set(dashboards::deleted_at.eq(Some(Utc::now())))
                .execute(&mut conn)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow!("Error updating dashboards: {}", e));
                }
            };

            Ok(())
        })
    };

    let dashboard_search_handle = {
        let ids = filtered_ids_to_delete.clone();
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("Unable to get connection from pool: {:?}", e);
                    send_sentry_error(&e.to_string(), None);
                    return;
                }
            };

            let query = diesel::sql_query(
                "UPDATE asset_search 
                SET deleted_at = NOW()
                WHERE asset_id = ANY($1) AND asset_type = 'dashboard'"
            )
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(&ids);

            if let Err(e) = query.execute(&mut conn).await {
                tracing::error!("Failed to update asset search: {:?}", e);
                send_sentry_error(&e.to_string(), None);
            }
        })
    };

    if let Err(e) = update_dashboard_record_handle.await {
        return Err(anyhow!("Error in dashboard update: {:?}", e));
    }

    if let Err(e) = dashboard_search_handle.await {
        return Err(anyhow!("Error in dashboard search update: {:?}", e));
    }

    Ok(DeleteDashboardResponse {
        ids: filtered_ids_to_delete,
    })
}
