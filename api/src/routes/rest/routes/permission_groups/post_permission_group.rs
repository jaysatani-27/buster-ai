use anyhow::Result;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::Utc;
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::lib::get_pg_pool;
use crate::database::models::{PermissionGroup, User};
use crate::database::schema::permission_groups;
use crate::routes::rest::ApiResponse;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;

#[derive(Debug, Deserialize)]
pub struct PostPermissionGroupRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct PostPermissionGroupResponse {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Uuid,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub async fn post_permission_group(
    Extension(user): Extension<User>,
    Json(request): Json<PostPermissionGroupRequest>,
) -> Result<ApiResponse<PostPermissionGroupResponse>, (StatusCode, &'static str)> {
    // Check if user is workspace admin or data admin
    let organization_id = get_user_organization_id(&user.id).await.map_err(|e| {
        tracing::error!("Error getting user organization id: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Error getting user organization id")
    })?;

    match is_user_workspace_admin_or_data_admin(&user, &organization_id).await {
        Ok(true) => (),
        Ok(false) => return Err((StatusCode::FORBIDDEN, "Insufficient permissions")),
        Err(e) => {
            tracing::error!("Error checking user permissions: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error checking user permissions",
            ));
        }
    }

    let permission_group = match post_permission_group_handler(user, request).await {
        Ok(group) => group,
        Err(e) => {
            tracing::error!("Error creating permission group: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error creating permission group",
            ));
        }
    };

    Ok(ApiResponse::JsonData(PostPermissionGroupResponse {
        id: permission_group.id,
        name: permission_group.name,
        organization_id: permission_group.organization_id,
        created_by: permission_group.created_by,
        updated_by: permission_group.updated_by,
        created_at: permission_group.created_at,
        updated_at: permission_group.updated_at,
    }))
}

async fn post_permission_group_handler(
    user: User,
    request: PostPermissionGroupRequest,
) -> Result<PermissionGroup> {
    let mut conn = get_pg_pool().get().await?;
    let organization_id = get_user_organization_id(&user.id).await?;

    let permission_group = PermissionGroup {
        id: Uuid::new_v4(),
        name: request.name,
        organization_id,
        created_by: user.id,
        updated_by: user.id,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    insert_into(permission_groups::table)
        .values(&permission_group)
        .execute(&mut *conn)
        .await?;

    Ok(permission_group)
}
