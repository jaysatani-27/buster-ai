use anyhow::{anyhow, Result};
use diesel::{update, ExpressionMethods};
use diesel_async::RunQueryDsl;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        lib::get_pg_pool,
        models::User,
        schema::organizations,
    },
    routes::ws::{
        organizations::organization_router::{OrganizationEvent, OrganizationRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub id: Uuid,
    pub name: String,
}

pub async fn update_organization(user: &User, req: UpdateOrganizationRequest) -> Result<()> {
    let org_state = match update_organization_handler(user, req.id, req.name).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error creating organization: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Organizations(OrganizationRoute::Post),
                WsEvent::Organizations(OrganizationEvent::Post),
                WsErrorCode::InternalServerError,
                "Failed to create organization.".to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let post_organization_message = WsResponseMessage::new(
        WsRoutes::Organizations(OrganizationRoute::Post),
        WsEvent::Organizations(OrganizationEvent::Post),
        org_state,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &post_organization_message).await {
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

async fn update_organization_handler(user: &User, id: Uuid, name: String) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting pg connection: {}", e)),
    };

    match update(organizations::table)
        .set(organizations::name.eq(name))
        .filter(organizations::id.eq(id))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting organization: {}", e)),
    }

    Ok(())
}
