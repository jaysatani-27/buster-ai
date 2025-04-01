use anyhow::Result;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Extension;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use uuid::Uuid;

use crate::database::enums::IdentityType;
use crate::database::lib::get_pg_pool;
use crate::database::models::User;
use crate::database::schema::{permission_groups_to_identities, users, users_to_organizations};
use crate::routes::rest::ApiResponse;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;

/// Represents user information with their assignment status to a permission group
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub assigned: bool,
}

/// List users that can be associated with a permission group
/// Returns users with their current assignment status to the specified permission group
pub async fn list_users(
    Extension(user): Extension<User>,
    Path(permission_group_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<UserInfo>>, (StatusCode, &'static str)> {
    let users = match list_users_handler(user, permission_group_id).await {
        Ok(users) => users,
        Err(e) => {
            tracing::error!("Error listing users for permission group: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error listing users for permission group",
            ));
        }
    };

    Ok(ApiResponse::JsonData(users))
}

async fn list_users_handler(user: User, permission_group_id: Uuid) -> Result<Vec<UserInfo>> {
    let mut conn = get_pg_pool().get().await?;
    let organization_id = get_user_organization_id(&user.id).await?;

    if !is_user_workspace_admin_or_data_admin(&user, &organization_id).await? {
        return Err(anyhow::anyhow!(
            "User is not authorized to list users for permission group"
        ));
    }

    // Query users and their assignment status to the permission group
    let users = users::table
        .left_join(
            permission_groups_to_identities::table.on(permission_groups_to_identities::identity_id
                .eq(users::id)
                .and(permission_groups_to_identities::permission_group_id.eq(permission_group_id))
                .and(permission_groups_to_identities::identity_type.eq(IdentityType::User))
                .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .inner_join(
            users_to_organizations::table.on(users_to_organizations::user_id
                .eq(users::id)
                .and(users_to_organizations::organization_id.eq(organization_id))
                .and(users_to_organizations::deleted_at.is_null())),
        )
        .select((
            users::id,
            users::name.nullable(),
            users::email,
            diesel::dsl::sql::<diesel::sql_types::Bool>(
                "permission_groups_to_identities.identity_id IS NOT NULL",
            ),
        ))
        .order_by(users::created_at.desc())
        .load::<(Uuid, Option<String>, String, bool)>(&mut *conn)
        .await?;

    Ok(users
        .into_iter()
        .map(|(id, name, email, assigned)| UserInfo {
            id,
            name: name.unwrap_or("".to_string()),
            email,
            assigned,
        })
        .collect())
}
