use anyhow::{anyhow, Result};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};

use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::TeamToUserRole,
        lib::get_pg_pool,
        models::User,
        schema::{teams_to_users, users, users_to_organizations},
    },
    routes::ws::{
        ws::{WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::send_ws_message,
    },
    utils::user::user_info::get_user_organization_id,
};

use super::users_router::{UserEvent, UserRoute};

#[derive(Deserialize, Debug, Clone)]
pub struct ListUsersRequest {
    pub team_id: Option<Uuid>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Serialize, Debug, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: Option<TeamToUserRole>,
}

pub async fn list_users(user: &User, req: ListUsersRequest) -> Result<()> {
    let page = req.page.unwrap_or(0);
    let page_size = req.page_size.unwrap_or(25);

    let user_info = match list_users_handler(user, req.team_id, page, page_size).await {
        Ok(users) => users,
        Err(e) => {
            tracing::error!("Error listing users: {}", e);
            return Err(e);
        }
    };

    let list_users_message = WsResponseMessage::new(
        WsRoutes::Users(UserRoute::List),
        WsEvent::Users(UserEvent::ListUsers),
        user_info,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &list_users_message).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error sending ws message: {}", e);
            return Err(anyhow!("Error sending ws message: {}", e));
        }
    }

    Ok(())
}

async fn list_users_handler(
    user: &User,
    team_id: Option<Uuid>,
    page: i64,
    page_size: i64,
) -> Result<Vec<UserInfo>> {
    let organization_id = get_user_organization_id(&user.id).await?;

    let mut conn = get_pg_pool().get().await?;

    let users = if let Some(team_id) = team_id {
        let user_records = users::table
            .inner_join(
                users_to_organizations::table.on(users::id.eq(users_to_organizations::user_id)),
            )
            .left_join(
                teams_to_users::table.on(users::id
                    .eq(teams_to_users::user_id)
                    .and(teams_to_users::team_id.eq(team_id))),
            )
            .filter(users_to_organizations::organization_id.eq(organization_id))
            .select((
                users::id,
                users::name.nullable(),
                users::email,
                teams_to_users::role.nullable(),
            ))
            .order(users::name.asc())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, Option<String>, String, Option<TeamToUserRole>)>(&mut conn)
            .await?;

        let user_info: Vec<UserInfo> = user_records
            .into_iter()
            .map(|(id, name, email, role)| UserInfo {
                id,
                name: name.unwrap_or_else(|| email.clone()),
                email,
                role,
            })
            .collect();
        user_info
    } else {
        let user_records = users::table
            .inner_join(
                users_to_organizations::table.on(users::id.eq(users_to_organizations::user_id)),
            )
            .filter(users_to_organizations::organization_id.eq(organization_id))
            .select((users::id, users::name.nullable(), users::email))
            .order(users::name.asc())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Uuid, Option<String>, String)>(&mut conn)
            .await?;

        let user_info: Vec<UserInfo> = user_records
            .into_iter()
            .map(|(id, name, email)| UserInfo {
                id,
                name: name.unwrap_or_else(|| email.clone()),
                email,
                role: None,
            })
            .collect();
        user_info
    };

    Ok(users)
}
