use anyhow::{anyhow, Result};
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::{SharingSetting, UserOrganizationRole, UserOrganizationStatus},
        lib::get_pg_pool,
        models::{Organization, User, UserToOrganization},
        schema::{organizations, users_to_organizations},
    },
    routes::ws::{
        organizations::organization_router::{OrganizationEvent, OrganizationRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{get_user_information, send_error_message, send_ws_message, UserInfoObject},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostOrganizationRequest {
    pub name: String,
}

pub async fn post_organization(user: &User, req: PostOrganizationRequest) -> Result<()> {
    let org_state = match post_organization_handler(user, req.name).await {
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

async fn post_organization_handler(user: &User, name: String) -> Result<UserInfoObject> {
    let domain = if user.email.contains("@") {
        Some(user.email.split("@").nth(1).unwrap_or("").to_string())
    } else {
        None
    };

    let organization = Organization {
        id: Uuid::new_v4(),
        name,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        domain,
    };

    let organization_user = UserToOrganization {
        user_id: user.id.clone(),
        organization_id: organization.id,
        role: UserOrganizationRole::WorkspaceAdmin,
        sharing_setting: SharingSetting::Public,
        edit_sql: true,
        upload_csv: true,
        export_assets: true,
        email_slack_enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        deleted_at: None,
        created_by: user.id.clone(),
        updated_by: user.id.clone(),
        deleted_by: None,
        status: UserOrganizationStatus::Active,
    };

    let mut conn = get_pg_pool().get().await?;

    match insert_into(organizations::table)
        .values(&organization)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting organization: {}", e)),
    };

    match insert_into(users_to_organizations::table)
        .values(&organization_user)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting organization user: {}", e)),
    };

    let user_info = match get_user_information(&user.id).await {
        Ok(organization_state) => organization_state,
        Err(e) => return Err(anyhow!("Error getting organization state: {}", e)),
    };

    Ok(user_info)
}
