use anyhow::Result;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Extension;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use uuid::Uuid;

use crate::database::lib::get_pg_pool;
use crate::database::models::User;
use crate::database::schema::{dataset_groups_permissions, users, users_to_organizations};
use crate::routes::rest::ApiResponse;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;

/// Represents user information with their assignment status to a dataset group
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub assigned: bool,
}

/// List users that can be associated with a dataset group
/// Returns users with their current assignment status to the specified dataset group
pub async fn list_users(
    Extension(user): Extension<User>,
    Path(dataset_group_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<UserInfo>>, (StatusCode, &'static str)> {
    let users = match list_users_handler(user, dataset_group_id).await {
        Ok(users) => users,
        Err(e) => {
            tracing::error!("Error listing users for dataset group: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error listing users for dataset group",
            ));
        }
    };

    Ok(ApiResponse::JsonData(users))
}

async fn list_users_handler(user: User, dataset_group_id: Uuid) -> Result<Vec<UserInfo>> {
    let mut conn = get_pg_pool().get().await?;
    let organization_id = get_user_organization_id(&user.id).await?;

    if !is_user_workspace_admin_or_data_admin(&user, &organization_id).await? {
        return Err(anyhow::anyhow!(
            "User is not authorized to list users for dataset group"
        ));
    }

    // Query users and their assignment status to the dataset group
    let users = users::table
        .left_join(
            dataset_groups_permissions::table.on(
                dataset_groups_permissions::permission_id
                    .eq(users::id)
                    .and(dataset_groups_permissions::dataset_group_id.eq(dataset_group_id))
                    .and(dataset_groups_permissions::permission_type.eq("user"))
                    .and(dataset_groups_permissions::deleted_at.is_null()),
            ),
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
                "dataset_groups_permissions.id IS NOT NULL",
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