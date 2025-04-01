use anyhow::Result;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tokio::spawn;
use uuid::Uuid;

use crate::database::lib::get_pg_pool;
use crate::database::models::{DatasetGroupPermission, User};
use crate::database::schema::dataset_groups_permissions;
use crate::routes::rest::ApiResponse;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;

#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetGroupAssignment {
    pub id: Uuid,
    pub assigned: bool,
}

pub async fn put_dataset_groups(
    Extension(user): Extension<User>,
    Path(user_id): Path<Uuid>,
    Json(assignments): Json<Vec<DatasetGroupAssignment>>,
) -> Result<ApiResponse<()>, (StatusCode, &'static str)> {
    match put_dataset_groups_handler(user, user_id, assignments).await {
        Ok(_) => Ok(ApiResponse::NoContent),
        Err(e) => {
            tracing::error!("Error assigning dataset groups: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error assigning dataset groups",
            ));
        }
    }
}

async fn put_dataset_groups_handler(
    user: User,
    user_id: Uuid,
    assignments: Vec<DatasetGroupAssignment>,
) -> Result<()> {
    let organization_id = get_user_organization_id(&user_id).await?;

    if !is_user_workspace_admin_or_data_admin(&user, &organization_id).await? {
        return Err(anyhow::anyhow!(
            "User is not authorized to assign dataset groups"
        ));
    };

    let (to_assign, to_unassign): (Vec<_>, Vec<_>) =
        assignments.into_iter().partition(|a| a.assigned);

    let assign_handle = {
        let user_id = user_id;
        spawn(async move {
            if !to_assign.is_empty() {
                let mut conn = get_pg_pool().get().await?;
                let values: Vec<_> = to_assign
                    .into_iter()
                    .map(|group| DatasetGroupPermission {
                        id: Uuid::new_v4(),
                        dataset_group_id: group.id,
                        permission_id: user_id,
                        permission_type: "user".to_string(),
                        organization_id: organization_id,
                        deleted_at: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    })
                    .collect();

                diesel::insert_into(dataset_groups_permissions::table)
                    .values(&values)
                    .on_conflict((
                        dataset_groups_permissions::dataset_group_id,
                        dataset_groups_permissions::permission_id,
                        dataset_groups_permissions::permission_type,
                    ))
                    .do_update()
                    .set(
                        dataset_groups_permissions::deleted_at
                            .eq(None::<chrono::DateTime<chrono::Utc>>),
                    )
                    .execute(&mut *conn)
                    .await?;
            }
            Ok::<_, anyhow::Error>(())
        })
    };

    let unassign_handle = {
        let user_id = user_id;
        spawn(async move {
            if !to_unassign.is_empty() {
                let mut conn = get_pg_pool().get().await?;
                diesel::update(dataset_groups_permissions::table)
                    .filter(
                        dataset_groups_permissions::dataset_group_id
                            .eq_any(to_unassign.iter().map(|a| a.id))
                            .and(dataset_groups_permissions::permission_id.eq(user_id))
                            .and(dataset_groups_permissions::permission_type.eq("user")),
                    )
                    .set(dataset_groups_permissions::deleted_at.eq(chrono::Utc::now()))
                    .execute(&mut *conn)
                    .await?;
            }
            Ok::<_, anyhow::Error>(())
        })
    };

    let (assign_result, unassign_result) = tokio::try_join!(assign_handle, unassign_handle)?;
    assign_result?;
    unassign_result?;

    Ok(())
}
