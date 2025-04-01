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
use crate::database::schema::{teams, teams_to_users};
use crate::routes::rest::ApiResponse;
use crate::utils::security::checks::is_user_workspace_admin_or_data_admin;
use crate::utils::user::user_info::get_user_organization_id;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamInfoRole {
    Member,
    Manager,
    None,
}

#[derive(Debug, Serialize)]
pub struct TeamInfo {
    pub id: Uuid,
    pub name: String,
    pub user_count: i64,
    pub role: TeamInfoRole,
}

pub async fn list_teams(
    Extension(user): Extension<User>,
    Path(user_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<TeamInfo>>, (StatusCode, &'static str)> {
    let teams = match list_teams_handler(user, user_id).await {
        Ok(teams) => teams,
        Err(e) => {
            tracing::error!("Error listing teams: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Error listing teams"));
        }
    };

    Ok(ApiResponse::JsonData(teams))
}

async fn list_teams_handler(user: User, user_id: Uuid) -> Result<Vec<TeamInfo>> {
    let mut conn = get_pg_pool().get().await?;
    let organization_id = get_user_organization_id(&user_id).await?;

    if !is_user_workspace_admin_or_data_admin(&user, &organization_id).await? {
        return Err(anyhow::anyhow!("User is not authorized to list teams"));
    }

    let teams = match teams::table
        .left_join(
            teams_to_users::table.on(teams::id
                .eq(teams_to_users::team_id)
                .and(teams_to_users::deleted_at.is_null())
                .and(teams_to_users::user_id.eq(user_id))),
        )
        .select((
            teams::id,
            teams::name,
            diesel::dsl::sql::<diesel::sql_types::BigInt>(
                "COALESCE(count(teams_to_users.user_id), 0)",
            ),
            diesel::dsl::sql::<diesel::sql_types::Text>(
                "COALESCE(teams_to_users.role::text, 'none')",
            ),
        ))
        .group_by((
            teams::id,
            teams::name,
            teams_to_users::user_id,
            teams_to_users::role,
        ))
        .filter(teams::organization_id.eq(organization_id))
        .filter(teams::deleted_at.is_null())
        .order_by(teams::created_at.desc())
        .load::<(Uuid, String, i64, String)>(&mut *conn)
        .await
    {
        Ok(teams) => teams,
        Err(e) => {
            tracing::error!("Error listing teams: {:?}", e);
            return Err(anyhow::anyhow!("Error listing teams"));
        }
    };

    Ok(teams
        .into_iter()
        .map(|(id, name, count, role)| TeamInfo {
            id,
            name,
            user_count: count,
            role: match role.as_str() {
                "member" => TeamInfoRole::Member,
                "manager" => TeamInfoRole::Manager,
                "none" => TeamInfoRole::None,
                _ => unreachable!(),
            },
        })
        .collect())
}
