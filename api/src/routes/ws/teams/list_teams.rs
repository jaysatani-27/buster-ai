use anyhow::{anyhow, Result};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        enums::IdentityType,
        lib::get_pg_pool,
        models::User,
        schema::{permission_groups_to_identities, teams, teams_to_users},
    },
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{clients::sentry_utils::send_sentry_error, user::user_info::get_user_organization_id},
};

use super::teams_routes::{TeamEvent, TeamRoute};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListTeamsFilter {
    pub permission_group_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub belongs_to: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListTeamsRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(flatten)]
    pub filters: Option<ListTeamsFilter>,
}
pub async fn list_teams(user: &User, req: ListTeamsRequest) -> Result<()> {
    let list_teams_res =
        match list_teams_handler(&user.id, req.page, req.page_size, req.filters).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error getting teams: {}", e);
                let err = anyhow!("Error getting teams: {}", e);
                send_sentry_error(&e.to_string(), Some(&user.id));
                send_error_message(
                    &user.id.to_string(),
                    WsRoutes::Teams(TeamRoute::List),
                    WsEvent::Teams(TeamEvent::ListTeams),
                    WsErrorCode::InternalServerError,
                    "Failed to list teams.".to_string(),
                    user,
                )
                .await?;
                return Err(err);
            }
        };

    let list_teams_message = WsResponseMessage::new(
        WsRoutes::Teams(TeamRoute::List),
        WsEvent::Teams(TeamEvent::ListTeams),
        list_teams_res,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &list_teams_message).await {
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

#[derive(Serialize)]
pub struct TeamObject {
    pub id: Uuid,
    pub name: String,
    pub belongs_to: Option<bool>,
}

async fn list_teams_handler(
    user_id: &Uuid,
    page: Option<i64>,
    page_size: Option<i64>,
    filters: Option<ListTeamsFilter>,
) -> Result<Vec<TeamObject>> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(25);

    let organization_id = get_user_organization_id(user_id).await?;

    let team_objects = if let Some(filters) = filters {
        if let Some(permission_group_id) = filters.permission_group_id {
            query_permission_groups_teams(
                &organization_id,
                &permission_group_id,
                page,
                page_size,
                filters.belongs_to,
            )
            .await?
        } else if let Some(user_id_to_query) = filters.user_id {
            query_user_teams(
                &organization_id,
                &user_id_to_query,
                page,
                page_size,
                filters.belongs_to,
            )
            .await?
        } else {
            query_teams(user_id, page, page_size).await?
        }
    } else {
        query_teams(user_id, page, page_size).await?
    };

    Ok(team_objects)
}

async fn query_teams(user_id: &Uuid, page: i64, page_size: i64) -> Result<Vec<TeamObject>> {
    let mut conn = get_pg_pool().get().await?;

    let teams_records: Vec<(Uuid, String)> = match teams::table
        .inner_join(teams_to_users::table)
        .select((teams::id, teams::name))
        .filter(teams_to_users::user_id.eq(user_id))
        .filter(teams::deleted_at.is_null())
        .limit(page_size)
        .offset(page * page_size)
        .load::<(Uuid, String)>(&mut conn)
        .await
    {
        Ok(teams) => teams,
        Err(e) => {
            return Err(anyhow!("Error getting teams: {}", e));
        }
    };

    let teams = teams_records
        .iter()
        .map(|(id, name)| TeamObject {
            id: *id,
            name: name.clone(),
            belongs_to: None,
        })
        .collect();

    Ok(teams)
}

async fn query_permission_groups_teams(
    organization_id: &Uuid,
    permission_group_id: &Uuid,
    page: i64,
    page_size: i64,
    only_owned: Option<bool>,
) -> Result<Vec<TeamObject>> {
    let mut conn = get_pg_pool().get().await?;

    let permission_group_team_results = if let Some(_) = only_owned {
        teams::table
            .inner_join(
                permission_groups_to_identities::table.on(teams::id
                    .eq(permission_groups_to_identities::identity_id)
                    .and(permission_groups_to_identities::identity_type.eq(IdentityType::Team))),
            )
            .select((
                teams::id,
                teams::name,
                permission_groups_to_identities::permission_group_id.nullable(),
            ))
            .filter(permission_groups_to_identities::permission_group_id.eq(&permission_group_id))
            .filter(permission_groups_to_identities::deleted_at.is_null())
            .filter(teams::organization_id.eq(organization_id))
            .filter(teams::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>)>(&mut conn)
            .await
    } else {
        teams::table
            .left_join(
                permission_groups_to_identities::table.on(teams::id
                    .eq(permission_groups_to_identities::identity_id)
                    .and(
                        permission_groups_to_identities::identity_type
                            .eq(IdentityType::Team)
                            .and(permission_groups_to_identities::deleted_at.is_null())
                            .and(
                                permission_groups_to_identities::permission_group_id
                                    .eq(&permission_group_id),
                            ),
                    )),
            )
            .select((
                teams::id,
                teams::name,
                permission_groups_to_identities::permission_group_id.nullable(),
            ))
            .filter(teams::deleted_at.is_null())
            .filter(teams::organization_id.eq(organization_id))
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>)>(&mut conn)
            .await
    };

    let permission_group_team_results: Vec<(Uuid, String, Option<Uuid>)> =
        match permission_group_team_results {
            Ok(teams) => teams,
            Err(e) => return Err(anyhow!("Error getting teams: {}", e)),
        };

    println!(
        "permission_group_team_results: {:?}",
        permission_group_team_results
    );

    let team_objects: Vec<TeamObject> = permission_group_team_results
        .into_iter()
        .map(|(id, name, permission_group_id)| TeamObject {
            id,
            name,
            belongs_to: Some(permission_group_id.is_some()),
        })
        .collect();

    Ok(team_objects)
}

async fn query_user_teams(
    organization_id: &Uuid,
    user_id: &Uuid, // this is the user being queried, not the user who made the req.
    page: i64,
    page_size: i64,
    only_owned: Option<bool>,
) -> Result<Vec<TeamObject>> {
    let mut conn = get_pg_pool().get().await?;

    let team_results = if let Some(true) = only_owned {
        teams::table
            .inner_join(teams_to_users::table)
            .select((teams::id, teams::name, teams_to_users::user_id.nullable()))
            .filter(teams_to_users::user_id.eq(user_id))
            .filter(teams::organization_id.eq(organization_id))
            .filter(teams::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>)>(&mut conn)
            .await
    } else {
        teams::table
            .left_join(
                teams_to_users::table.on(teams::id
                    .eq(teams_to_users::team_id)
                    .and(teams_to_users::user_id.eq(user_id))),
            )
            .select((teams::id, teams::name, teams_to_users::user_id.nullable()))
            .filter(teams::organization_id.eq(organization_id))
            .filter(teams::deleted_at.is_null())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, String, Option<Uuid>)>(&mut conn)
            .await
    };

    let team_results: Vec<(Uuid, String, Option<Uuid>)> = match team_results {
        Ok(teams) => teams,
        Err(e) => return Err(anyhow!("Error getting teams: {}", e)),
    };

    let team_objects = team_results
        .into_iter()
        .map(|(id, name, user_id)| TeamObject {
            id,
            name,
            belongs_to: Some(user_id.is_some()),
        })
        .collect();

    Ok(team_objects)
}
