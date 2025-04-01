use anyhow::Result;
use axum::{extract::Path, http::StatusCode, Extension};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::lib::get_pg_pool;
use crate::database::models::User;
use crate::database::schema::dataset_groups;
use crate::routes::rest::ApiResponse;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;

pub async fn delete_dataset_group(
    Extension(user): Extension<User>,
    Path(dataset_group_id): Path<Uuid>,
) -> Result<ApiResponse<()>, (StatusCode, &'static str)> {
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

    match delete_dataset_group_handler(dataset_group_id).await {
        Ok(_) => Ok(ApiResponse::NoContent),
        Err(e) => {
            tracing::error!("Error deleting dataset group: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error deleting dataset group",
            ))
        }
    }
}

async fn delete_dataset_group_handler(dataset_group_id: Uuid) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    let rows_affected = diesel::update(
        dataset_groups::table
            .filter(dataset_groups::id.eq(dataset_group_id))
            .filter(dataset_groups::deleted_at.is_null()),
    )
    .set(dataset_groups::deleted_at.eq(Some(Utc::now())))
    .execute(&mut *conn)
    .await?;

    if rows_affected == 0 {
        return Err(anyhow::anyhow!("Dataset group not found"));
    }

    Ok(())
} 