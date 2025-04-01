use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::{AssetPermissionRole, AssetType, IdentityType},
        lib::{get_pg_pool, get_sqlx_pool},
        models::{AssetPermission, Dashboard, User},
        schema::{asset_permissions, dashboards},
    },
    routes::ws::{
        dashboards::dashboards_router::{DashboardEvent, DashboardRoute},
        ws::{SubscriptionRwLock, WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message, subscribe_to_stream},
    },
    utils::{clients::sentry_utils::send_sentry_error, user::user_info::get_user_organization_id},
};

use super::dashboards_router::JoinedDashboard;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostDashboardRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn post_dashboard(
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
    req: PostDashboardRequest,
) -> Result<()> {
    let dashboard_id = Uuid::new_v4();

    let dashboard_subscription = format!("dashboard:{}", dashboard_id);

    match subscribe_to_stream(subscriptions, &dashboard_subscription, user_group, &user.id).await {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error subscribing to dashboard: {}", e)),
    };

    let dashboard = match create_dashboad_handler(&user, &dashboard_id, req).await {
        Ok(dashboard) => dashboard,
        Err(e) => {
            tracing::error!("Error getting threads: {}", e);
            let err = anyhow!("Error getting threads: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Dashboards(DashboardRoute::Post),
                WsEvent::Dashboards(DashboardEvent::PostDashboard),
                WsErrorCode::InternalServerError,
                "Failed to create dashboard.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let join_dashboard_response = JoinedDashboard {
        id: user.id.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        dashboard_id: dashboard.id,
    };

    let join_thread_ws_message = WsResponseMessage::new(
        WsRoutes::Dashboards(DashboardRoute::Post),
        WsEvent::Dashboards(DashboardEvent::JoinedDashboard),
        vec![join_dashboard_response],
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&dashboard_subscription, &join_thread_ws_message).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending message to pubsub: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(anyhow!("Error sending message to pubsub: {}", e));
        }
    }

    let dashboard_message_ws_message = WsResponseMessage::new(
        WsRoutes::Dashboards(DashboardRoute::Post),
        WsEvent::Dashboards(DashboardEvent::PostDashboard),
        dashboard,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&dashboard_subscription, &dashboard_message_ws_message).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending message to pubsub: {}", e);
            return Err(anyhow!("Error sending message to pubsub: {}", e));
        }
    }

    Ok(())
}

async fn create_dashboad_handler(
    user: &User,
    dashboard_id: &Uuid,
    req: PostDashboardRequest,
) -> Result<Dashboard> {
    let organization_id = get_user_organization_id(&user.id).await?;

    let dashboard = Dashboard {
        id: *dashboard_id,
        name: req.name,
        description: req.description,
        config: json!({}),
        created_by: user.id,
        updated_by: user.id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        publicly_accessible: false,
        publicly_enabled_by: None,
        public_expiry_date: None,
        password_secret_id: None,
        organization_id,
    };

    let user_to_dashboard = AssetPermission {
        identity_id: user.id,
        identity_type: IdentityType::User,
        asset_id: dashboard.id,
        asset_type: AssetType::Dashboard,
        role: AssetPermissionRole::Owner,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        created_by: user.id,
        updated_by: user.id,
    };

    let dashboard_insert = {
        let dashboard = dashboard.clone();
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => return Err(anyhow!("Error getting pg pool: {:?}", e)),
            };

            match insert_into(dashboards::table)
                .values(&dashboard)
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Error inserting dashboard: {:?}", e)),
            }
        })
    };

    let permission_insert = {
        tokio::spawn(async move {
            let mut conn = match get_pg_pool().get().await {
                Ok(conn) => conn,
                Err(e) => return Err(anyhow!("Error getting pg pool: {:?}", e)),
            };

            match insert_into(asset_permissions::table)
                .values(&user_to_dashboard)
                .execute(&mut conn)
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Error inserting user to dashboard: {:?}", e)),
            }
        })
    };

    let dashboard_id = dashboard.id.clone();
    let dashboard_name = dashboard.name.clone();
    let organization_id = dashboard.organization_id.clone();

    let dashboard_search_handle = tokio::spawn(async move {
        let mut conn = match get_pg_pool().get().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("Unable to get connection from pool: {:?}", e);
                send_sentry_error(&e.to_string(), None);
                return;
            }
        };

        let query = diesel::sql_query(
            "INSERT INTO asset_search (asset_id, asset_type, content, organization_id)
            VALUES ($1, 'dashboard', $2, $3)
            ON CONFLICT (asset_id, asset_type) 
            DO UPDATE SET
                content = EXCLUDED.content,
                updated_at = NOW()"
        )
        .bind::<diesel::sql_types::Uuid, _>(dashboard_id)
        .bind::<diesel::sql_types::Text, _>(dashboard_name)
        .bind::<diesel::sql_types::Uuid, _>(organization_id);

        if let Err(e) = query.execute(&mut conn).await {
            tracing::error!("Failed to update asset search: {:?}", e);
            send_sentry_error(&e.to_string(), None);
        }
    });

    if let Err(e) = dashboard_insert.await {
        return Err(anyhow!("Error in dashboard insert: {:?}", e));
    }

    if let Err(e) = permission_insert.await {
        return Err(anyhow!("Error in permission insert: {:?}", e));
    }

    if let Err(e) = dashboard_search_handle.await {
        return Err(anyhow!("Error in dashboard search insert: {:?}", e));
    }

    Ok(dashboard)
}
