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
use crate::database::schema::{dataset_permissions, datasets};
use crate::routes::rest::ApiResponse;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;

#[derive(Debug, Serialize)]
pub struct DatasetInfo {
    pub id: Uuid,
    pub name: String,
    pub assigned: bool,
}

pub async fn list_datasets(
    Extension(user): Extension<User>,
    Path(user_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<DatasetInfo>>, (StatusCode, &'static str)> {
    let datasets = match list_datasets_handler(user, user_id).await {
        Ok(datasets) => datasets,
        Err(e) => {
            tracing::error!("Error listing datasets: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Error listing datasets"));
        }
    };

    Ok(ApiResponse::JsonData(datasets))
}

async fn list_datasets_handler(user: User, user_id: Uuid) -> Result<Vec<DatasetInfo>> {
    let mut conn = get_pg_pool().get().await?;
    let organization_id = get_user_organization_id(&user_id).await?;

    if !is_user_workspace_admin_or_data_admin(&user, &organization_id).await? {
        return Err(anyhow::anyhow!("User is not authorized to list datasets"));
    }

    let datasets = match datasets::table
        .left_join(
            dataset_permissions::table.on(dataset_permissions::dataset_id
                .eq(datasets::id)
                .and(dataset_permissions::permission_type.eq("user"))
                .and(dataset_permissions::permission_id.eq(user_id))
                .and(dataset_permissions::deleted_at.is_null())),
        )
        .filter(datasets::organization_id.eq(organization_id))
        .filter(datasets::deleted_at.is_null())
        .select((
            datasets::id,
            datasets::name,
            diesel::dsl::sql::<diesel::sql_types::Bool>("dataset_permissions.id IS NOT NULL"),
        ))
        .load::<(Uuid, String, bool)>(&mut *conn)
        .await
    {
        Ok(datasets) => datasets,
        Err(e) => {
            tracing::error!("Error listing datasets: {:?}", e);
            return Err(anyhow::anyhow!("Error listing datasets"));
        }
    };

    Ok(datasets
        .into_iter()
        .map(|(id, name, assigned)| DatasetInfo { id, name, assigned })
        .collect())
}
