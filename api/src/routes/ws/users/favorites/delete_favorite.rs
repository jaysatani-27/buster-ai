use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::{update, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        lib::get_pg_pool,
        models::User,
        schema::user_favorites,
    },
    routes::ws::{
        users::users_router::{UserEvent, UserRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::favorites_utils::{list_user_favorites, FavoriteEnum};

#[derive(Deserialize)]
pub struct CreateFavoriteReq {
    pub id: Uuid,
}

pub async fn delete_favorite(user: &User, req: CreateFavoriteReq) -> Result<()> {
    let create_favorite_res = match delete_favorite_handler(&user, &req.id).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error creating favorite: {}", e);
            let err = anyhow!("Error creating favorite: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Users(UserRoute::CreateFavorite),
                WsEvent::Users(UserEvent::CreateFavorite),
                WsErrorCode::InternalServerError,
                "Failed to create favorite.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let create_favorite_message = WsResponseMessage::new(
        WsRoutes::Users(UserRoute::CreateFavorite),
        WsEvent::Users(UserEvent::CreateFavorite),
        create_favorite_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &create_favorite_message).await {
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

async fn delete_favorite_handler(user: &User, id: &Uuid) -> Result<Vec<FavoriteEnum>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    match update(user_favorites::table)
        .set(user_favorites::deleted_at.eq(Some(Utc::now())))
        .filter(user_favorites::user_id.eq(user.id))
        .filter(user_favorites::asset_id.eq(id))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error deleting favorite: {:?}", e)),
    };

    let user_favorites = match list_user_favorites(user).await {
        Ok(favorites) => favorites,
        Err(e) => return Err(e),
    };

    Ok(user_favorites)
}
