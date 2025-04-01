use anyhow::Result;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tokio::spawn;
use uuid::Uuid;

use crate::database::enums::IdentityType;
use crate::database::lib::get_pg_pool;
use crate::database::models::{PermissionGroupToIdentity, User};
use crate::database::schema::permission_groups_to_identities;
use crate::routes::rest::ApiResponse;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserAssignment {
    pub id: Uuid,
    pub assigned: bool,
}

/// Update user assignments for a permission group
/// Accepts a list of user assignments to add or remove from the permission group
pub async fn put_users(
    Extension(user): Extension<User>,
    Path(permission_group_id): Path<Uuid>,
    Json(assignments): Json<Vec<UserAssignment>>,
) -> Result<ApiResponse<()>, (StatusCode, &'static str)> {
    match put_users_handler(user, permission_group_id, assignments).await {
        Ok(_) => Ok(ApiResponse::NoContent),
        Err(e) => {
            tracing::error!("Error assigning users to permission group: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error assigning users to permission group",
            ));
        }
    }
}

async fn put_users_handler(
    user: User,
    permission_group_id: Uuid,
    assignments: Vec<UserAssignment>,
) -> Result<()> {
    let organization_id = get_user_organization_id(&user.id).await?;

    if !is_user_workspace_admin_or_data_admin(&user, &organization_id).await? {
        return Err(anyhow::anyhow!(
            "User is not authorized to assign users to permission group"
        ));
    }

    let (to_assign, to_unassign): (Vec<_>, Vec<_>) =
        assignments.into_iter().partition(|a| a.assigned);

    let assign_handle = {
        let permission_group_id = permission_group_id;
        spawn(async move {
            if !to_assign.is_empty() {
                let mut conn = get_pg_pool().get().await?;
                let values: Vec<_> = to_assign
                    .into_iter()
                    .map(|user_assignment| PermissionGroupToIdentity {
                        permission_group_id,
                        identity_id: user_assignment.id,
                        identity_type: IdentityType::User,
                        deleted_at: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        created_by: user.id.clone(),
                        updated_by: user.id,
                    })
                    .collect();

                diesel::insert_into(permission_groups_to_identities::table)
                    .values(&values)
                    .on_conflict((
                        permission_groups_to_identities::permission_group_id,
                        permission_groups_to_identities::identity_id,
                        permission_groups_to_identities::identity_type,
                    ))
                    .do_update()
                    .set(
                        permission_groups_to_identities::deleted_at
                            .eq(None::<chrono::DateTime<chrono::Utc>>),
                    )
                    .execute(&mut *conn)
                    .await?;
            }
            Ok::<_, anyhow::Error>(())
        })
    };

    let unassign_handle = {
        let permission_group_id = permission_group_id;
        spawn(async move {
            if !to_unassign.is_empty() {
                let mut conn = get_pg_pool().get().await?;
                diesel::update(permission_groups_to_identities::table)
                    .filter(
                        permission_groups_to_identities::permission_group_id
                            .eq(permission_group_id)
                            .and(
                                permission_groups_to_identities::identity_id
                                    .eq_any(to_unassign.iter().map(|a| a.id)),
                            )
                            .and(
                                permission_groups_to_identities::identity_type
                                    .eq(IdentityType::User),
                            ),
                    )
                    .set(permission_groups_to_identities::deleted_at.eq(chrono::Utc::now()))
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
