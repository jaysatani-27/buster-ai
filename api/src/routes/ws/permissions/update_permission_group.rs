use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{dsl::not, insert_into, update, AsChangeset, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    database::{
        enums::IdentityType,
        lib::get_pg_pool,
        models::{DatasetToPermissionGroup, User},
        schema::{datasets_to_permission_groups, permission_groups},
    },
    routes::ws::{
        permissions::permissions_router::{PermissionEvent, PermissionRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::permissions_utils::{
    add_identities_to_permission_group, get_permission_group_state,
    remove_identities_from_permission_group, PermissionGroupState,
};

#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = permission_groups)]
pub struct PermissionGroupUpdateBody {
    pub name: Option<String>,
    pub updated_by: Option<Uuid>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePermissionGroupRequest {
    pub id: Uuid,
    #[serde(flatten)]
    pub permission_group_update_body: Option<PermissionGroupUpdateBody>,
    pub datasets: Option<Vec<Uuid>>,
    pub teams: Option<Vec<Uuid>>,
    pub users: Option<Vec<Uuid>>,
}

pub async fn update_permission_group(user: &User, req: UpdatePermissionGroupRequest) -> Result<()> {
    let permission_group_state = match update_permission_group_handler(
        &user.id,
        &req.id,
        req.permission_group_update_body,
        req.datasets,
        req.teams,
        req.users,
    )
    .await
    {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error updating permission group: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Permissions(PermissionRoute::UpdatePermissionGroup),
                WsEvent::Permissions(PermissionEvent::UpdatePermissionGroup),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let update_permission_group_message = WsResponseMessage::new(
        WsRoutes::Permissions(PermissionRoute::UpdatePermissionGroup),
        WsEvent::Permissions(PermissionEvent::UpdatePermissionGroup),
        permission_group_state,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &update_permission_group_message).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error sending ws message: {}", e);
            let err = anyhow!("Error sending ws message: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(err);
        }
    }

    Ok(())
}

async fn update_permission_group_handler(
    user_id: &Uuid,
    id: &Uuid,
    update_body: Option<PermissionGroupUpdateBody>,
    datasets: Option<Vec<Uuid>>,
    teams: Option<Vec<Uuid>>,
    users: Option<Vec<Uuid>>,
) -> Result<PermissionGroupState> {
    let user_id = Arc::new(user_id.clone());
    let permission_group_id = Arc::new(id.clone());

    let update_permission_group = {
        let permission_group_id = permission_group_id.clone();
        let user_id = user_id.clone();
        tokio::spawn(async move {
            if let Err(e) =
                update_permission_group_data(user_id, permission_group_id, update_body).await
            {
                tracing::error!("Error in update_permission_group_data: {}", e);
            }
        })
    };

    let update_identities = {
        let permission_group_id = permission_group_id.clone();
        let user_id = user_id.clone();
        tokio::spawn(async move {
            if let Err(e) =
                update_permission_group_identities(user_id, permission_group_id, users, teams).await
            {
                tracing::error!("Error in update_permission_group_identities: {}", e);
            }
        })
    };

    let update_datasets = {
        let permission_group_id = permission_group_id.clone();
        let datasets = datasets.clone();
        tokio::spawn(async move {
            if let Err(e) = update_permission_group_datasets(permission_group_id, datasets).await {
                tracing::error!("Error in update_permission_group_datasets: {}", e);
            }
        })
    };

    let (_, _, _) = tokio::try_join!(update_permission_group, update_identities, update_datasets)?;

    let permission_group_state = match get_permission_group_state(&permission_group_id).await {
        Ok(state) => state,
        Err(e) => return Err(e),
    };

    Ok(permission_group_state)
}

async fn update_permission_group_data(
    user_id: Arc<Uuid>,
    permission_group_id: Arc<Uuid>,
    update_body: Option<PermissionGroupUpdateBody>,
) -> Result<()> {
    if let Some(mut permission_group_changeset) = update_body {
        if permission_group_changeset.name.is_none() {
            return Ok(());
        }

        permission_group_changeset.updated_by = Some(*user_id);
        permission_group_changeset.updated_at = Some(Utc::now());

        let mut conn = get_pg_pool().get().await?;

        match diesel::update(permission_groups::table)
            .filter(permission_groups::id.eq(*permission_group_id))
            .set(&permission_group_changeset)
            .execute(&mut conn)
            .await
        {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Error updating permission group: {}", e)),
        }
    }
    Ok(())
}

async fn update_permission_group_identities(
    user_id: Arc<Uuid>,
    permission_group_id: Arc<Uuid>,
    users: Option<Vec<Uuid>>,
    teams: Option<Vec<Uuid>>,
) -> Result<()> {
    if users.is_some() || teams.is_some() {
        let mut identities = vec![];
        if let Some(users) = users {
            identities.extend(
                users
                    .into_iter()
                    .map(|user_id| (IdentityType::User, user_id)),
            );
        }
        if let Some(teams) = teams {
            identities.extend(
                teams
                    .into_iter()
                    .map(|team_id| (IdentityType::Team, team_id)),
            );
        }

        let ids = identities
            .iter()
            .map(|(_, identity_id)| identity_id.clone())
            .collect::<Vec<Uuid>>();

        let add_identities_handle = {
            let permission_group_id = permission_group_id.clone();
            let identities = identities.clone();
            tokio::spawn(async move {
                add_identities_to_permission_group(&user_id, &permission_group_id, identities).await
            })
        };

        let ids = Arc::new(ids);

        let remove_teams_handle = {
            let permission_group_id = permission_group_id.clone();
            let identity_type = IdentityType::Team;
            let ids = ids.clone();
            tokio::spawn(async move {
                remove_identities_from_permission_group(&permission_group_id, ids, identity_type)
                    .await
            })
        };

        let remove_users_handle = {
            let permission_group_id = permission_group_id.clone();
            let identity_type = IdentityType::User;
            let ids = ids.clone();
            tokio::spawn(async move {
                remove_identities_from_permission_group(&permission_group_id, ids, identity_type)
                    .await
            })
        };

        let (_, _, _) = tokio::try_join!(
            add_identities_handle,
            remove_teams_handle,
            remove_users_handle
        )?;
    }

    Ok(())
}

async fn update_permission_group_datasets(
    permission_group_id: Arc<Uuid>,
    datasets: Option<Vec<Uuid>>,
) -> Result<()> {
    if let Some(datasets) = datasets {
        let datasets = Arc::new(datasets);

        let add_datasets_handle = {
            let permission_group_id = permission_group_id.clone();
            let datasets = datasets.clone();
            tokio::spawn(async move {
                add_permission_groups_to_datasets(permission_group_id, datasets).await
            })
        };

        let remove_datasets_handle = {
            let permission_group_id = permission_group_id.clone();
            let datasets = datasets.clone();
            tokio::spawn(async move {
                remove_permission_groups_from_datasets(permission_group_id, datasets).await
            })
        };

        let (_, _) = tokio::try_join!(add_datasets_handle, remove_datasets_handle)?;
    }

    Ok(())
}

async fn add_permission_groups_to_datasets(
    permission_group_id: Arc<Uuid>,
    datasets: Arc<Vec<Uuid>>,
) -> Result<()> {
    let permission_groups_to_datasets: Vec<DatasetToPermissionGroup> = datasets
        .iter()
        .map(|dataset_id| DatasetToPermissionGroup {
            dataset_id: *dataset_id,
            permission_group_id: *permission_group_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        })
        .collect();

    let mut conn = get_pg_pool().get().await?;

    match insert_into(datasets_to_permission_groups::table)
        .values(&permission_groups_to_datasets)
        .on_conflict((
            datasets_to_permission_groups::dataset_id,
            datasets_to_permission_groups::permission_group_id,
        ))
        .do_update()
        .set((
            datasets_to_permission_groups::deleted_at.eq(None::<chrono::DateTime<Utc>>),
            datasets_to_permission_groups::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error adding permission groups to datasets: {}", e)),
    }

    Ok(())
}

async fn remove_permission_groups_from_datasets(
    permission_group_id: Arc<Uuid>,
    datasets: Arc<Vec<Uuid>>,
) -> Result<()> {
    let mut conn = get_pg_pool().get().await?;

    match update(datasets_to_permission_groups::table)
        .set(datasets_to_permission_groups::deleted_at.eq(Some(Utc::now())))
        .filter(datasets_to_permission_groups::permission_group_id.eq(permission_group_id.as_ref()))
        .filter(not(
            datasets_to_permission_groups::dataset_id.eq_any(datasets.as_ref())
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error removing permission groups from datasets: {}", e);
            return Err(anyhow!(
                "Error removing permission groups from datasets: {}",
                e
            ));
        }
    }

    Ok(())
}
