use anyhow::Result;
use axum::extract::Path;
use axum::{Extension, Json};

use crate::database::enums::UserOrganizationStatus;
use crate::database::models::User;
use crate::database::schema::{users, users_to_organizations};
use crate::database::{enums::UserOrganizationRole, lib::get_pg_pool};
use crate::routes::rest::ApiResponse;
use crate::utils::clients::sentry_utils::send_sentry_error;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;
use axum::http::StatusCode;
use diesel::{update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: Option<String>,
    pub email: String,
    pub role: UserOrganizationRole,
    pub status: UserOrganizationStatus,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<UserOrganizationRole>,
}

pub async fn update_user(
    Extension(user): Extension<User>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<ApiResponse<()>, (StatusCode, &'static str)> {
    match update_user_handler(&user, &user_id, body).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error getting user information: {:?}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting user information",
            ));
        }
    };

    Ok(ApiResponse::NoContent)
}

pub async fn update_user_handler(
    auth_user: &User,
    user_id: &Uuid,
    change: UpdateUserRequest,
) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(_e) => {
            return Err(anyhow::anyhow!("Error getting postgres connection"));
        }
    };

    let user_organization_id = match get_user_organization_id(&user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Error getting user organization id: {:?}",
                e
            ));
        }
    };

    if &auth_user.id == user_id {
        return Err(anyhow::anyhow!("Cannot update self"));
    };

    match is_user_workspace_admin_or_data_admin(auth_user, &user_organization_id).await {
        Ok(true) => (),
        Ok(false) => return Err(anyhow::anyhow!("Insufficient permissions")),
        Err(e) => {
            tracing::error!("Error checking user permissions: {:?}", e);
            return Err(anyhow::anyhow!("Error checking user permissions"));
        }
    }

    if let Some(name) = change.name {
        match update(users::table)
            .filter(users::id.eq(user_id))
            .set(users::name.eq(name))
            .execute(&mut conn)
            .await
        {
            Ok(user) => user,
            Err(e) => return Err(anyhow::anyhow!("Error updating user: {:?}", e)),
        };
    }

    if let Some(role) = change.role {
        match update(users_to_organizations::table)
            .filter(users_to_organizations::user_id.eq(user_id))
            .set(users_to_organizations::role.eq(role))
            .execute(&mut conn)
            .await
        {
            Ok(user_organization_role_update) => user_organization_role_update,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error updating user organization role: {:?}",
                    e
                ))
            }
        };
    }

    Ok(())
}
