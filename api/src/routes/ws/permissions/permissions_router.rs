
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::models::User;

use super::{
    delete_permission_group::delete_permission_group, delete_team::delete_team_permission,
    get_permission_group::get_permission_group, get_team_permissions::get_team_permissions,
    get_user_permissions::get_user_permissions, list_permission_groups::list_permission_groups,
    list_teams::list_teams, list_users::list_users, post_permission_group::post_permission_group,
    post_team::post_team, post_user::post_user, update_permission_group::update_permission_group,
    update_team_permission::update_team_permission, update_user_permission::update_user_permission,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PermissionRoute {
    #[serde(rename = "/permissions/users/get")]
    GetUserPermissions,
    #[serde(rename = "/permissions/groups/get")]
    GetPermissionGroup,
    #[serde(rename = "/permissions/teams/get")]
    GetTeamPermissions,
    #[serde(rename = "/permissions/groups/list")]
    ListPermissionGroups,
    #[serde(rename = "/permissions/users/list")]
    ListUserPermissions,
    #[serde(rename = "/permissions/teams/list")]
    ListTeamPermissions,
    #[serde(rename = "/permissions/groups/post")]
    PostPermissionGroup,
    #[serde(rename = "/permissions/teams/post")]
    PostTeam,
    #[serde(rename = "/permissions/users/post")]
    PostUser,
    #[serde(rename = "/permissions/groups/update")]
    UpdatePermissionGroup,
    #[serde(rename = "/permissions/teams/update")]
    UpdateTeamPermission,
    #[serde(rename = "/permissions/users/update")]
    UpdateUserPermission,
    #[serde(rename = "/permissions/teams/delete")]
    DeleteTeamPermission,
    #[serde(rename = "/permissions/groups/delete")]
    DeletePermissionGroup,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PermissionEvent {
    GetUserPermissions,
    GetPermissionGroup,
    GetTeamPermissions,
    ListUserPermissions,
    ListPermissionGroups,
    ListTeamPermissions,
    PostPermissionGroup,
    PostTeam,
    PostUser,
    UpdatePermissionGroup,
    UpdateTeamPermission,
    UpdateUserPermission,
    DeleteTeamPermission,
    DeletePermissionGroup,
}

pub async fn permissions_router(route: PermissionRoute, data: Value, user: &User) -> Result<()> {
    match route {
        PermissionRoute::GetUserPermissions => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            get_user_permissions(user, req).await?;
        }
        PermissionRoute::GetPermissionGroup => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            get_permission_group(user, req).await?;
        }
        PermissionRoute::GetTeamPermissions => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            get_team_permissions(user, req).await?;
        }
        PermissionRoute::ListPermissionGroups => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            list_permission_groups(user, req).await?;
        }
        PermissionRoute::ListUserPermissions => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            list_users(user, req).await?;
        }
        PermissionRoute::ListTeamPermissions => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            list_teams(user, req).await?;
        }
        PermissionRoute::PostPermissionGroup => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            post_permission_group(user, req).await?;
        }
        PermissionRoute::PostTeam => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            post_team(user, req).await?;
        }
        PermissionRoute::PostUser => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            match post_user(user, req).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Error posting user: {}", e);
                    return Err(e);
                }
            }
        }
        PermissionRoute::UpdateUserPermission => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            update_user_permission(user, req).await?;
        }
        PermissionRoute::UpdatePermissionGroup => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            update_permission_group(user, req).await?;
        }
        PermissionRoute::UpdateTeamPermission => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            match update_team_permission(user, req).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Error updating team permission: {}", e);
                    return Err(e);
                }
            }
        }
        PermissionRoute::DeleteTeamPermission => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            delete_team_permission(user, req).await?;
        }
        PermissionRoute::DeletePermissionGroup => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };
            delete_permission_group(user, req).await?;
        }
    };

    Ok(())
}

impl PermissionRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/permissions/users/get" => Ok(Self::GetUserPermissions),
            "/permissions/groups/get" => Ok(Self::GetPermissionGroup),
            "/permissions/teams/get" => Ok(Self::GetTeamPermissions),
            "/permissions/groups/list" => Ok(Self::ListPermissionGroups),
            "/permissions/users/list" => Ok(Self::ListUserPermissions),
            "/permissions/teams/list" => Ok(Self::ListTeamPermissions),
            "/permissions/groups/post" => Ok(Self::PostPermissionGroup),
            "/permissions/teams/post" => Ok(Self::PostTeam),
            "/permissions/users/post" => Ok(Self::PostUser),
            "/permissions/groups/update" => Ok(Self::UpdatePermissionGroup),
            "/permissions/teams/update" => Ok(Self::UpdateTeamPermission),
            "/permissions/users/update" => Ok(Self::UpdateUserPermission),
            "/permissions/teams/delete" => Ok(Self::DeleteTeamPermission),
            "/permissions/groups/delete" => Ok(Self::DeletePermissionGroup),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
