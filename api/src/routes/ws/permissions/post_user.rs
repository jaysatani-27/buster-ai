use anyhow::{anyhow, Result};
use diesel::insert_into;
use diesel_async::RunQueryDsl;

use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    database::{
        enums::{SharingSetting, UserOrganizationRole, UserOrganizationStatus},
        lib::{get_pg_pool, UserConfig},
        models::{User, UserToOrganization},
        schema::{users, users_to_organizations},
    },
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{clients::sentry_utils::send_sentry_error, user::user_info::get_user_organization_id},
};

use super::{
    permissions_router::{PermissionEvent, PermissionRoute},
    permissions_utils::{get_user_permission_group_state, UserPermissionGroupState},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostUserRequest {
    pub email: String,
    pub role: UserOrganizationRole,
}

pub async fn post_user(user: &User, req: PostUserRequest) -> Result<()> {
    let permission_group_state = match post_user_handler(&user.id, req.email, req.role).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error creating user: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::PostUser),
                WsEvent::Permissions(PermissionEvent::PostUser),
                WsErrorCode::InternalServerError,
                "Failed to create user.".to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let post_user_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::PostUser),
        WsEvent::Permissions(PermissionEvent::PostUser),
        permission_group_state,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &post_user_message).await {
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

async fn post_user_handler(
    user_id: &Uuid,
    email: String,
    role: UserOrganizationRole,
) -> Result<UserPermissionGroupState> {
    let user_organization_id = match get_user_organization_id(user_id).await {
        Ok(user_organization_id) => user_organization_id,
        Err(e) => return Err(anyhow!("Error getting user organization id: {}", e)),
    };

    let new_user_id = Uuid::new_v4();

    let new_user = User {
        id: new_user_id.clone(),
        email: email.clone(),
        name: None,
        config: serde_json::to_value(UserConfig {
            color_palettes: None,
            last_used_color_palette: None,
        })?,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        attributes: json!({
            "user_id": new_user_id.to_string(),
            "organization_id": user_organization_id.to_string(),
            "user_email": email,
            "organization_role": role.to_string(),
        }),
    };

    let user_to_organization = UserToOrganization {
        user_id: new_user.id,
        organization_id: user_organization_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        role,
        sharing_setting: SharingSetting::Organization,
        edit_sql: true,
        upload_csv: true,
        export_assets: true,
        email_slack_enabled: false,
        deleted_at: None,
        created_by: *user_id,
        updated_by: *user_id,
        deleted_by: None,
        status: UserOrganizationStatus::Active,
    };

    let mut conn = get_pg_pool().get().await?;

    match insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting permission group: {}", e)),
    };

    match insert_into(users_to_organizations::table)
        .values(&user_to_organization)
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting user to organization: {}", e)),
    };

    // TODO: Need to invite the user via email.

    let user_permission_group_state = match get_user_permission_group_state(&new_user.id).await {
        Ok(user_permission_group_state) => user_permission_group_state,
        Err(e) => return Err(anyhow!("Error getting permission group state: {}", e)),
    };

    Ok(user_permission_group_state)
}
