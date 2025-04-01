use anyhow::{anyhow, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::models::User,
    routes::ws::{
        users::users_router::{UserEvent, UserRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::favorites_utils::{list_user_favorites, update_favorites, FavoriteEnum};

#[derive(Deserialize)]
pub struct UpdateFavoritesReq {
    pub favorites: Vec<Uuid>,
}

pub async fn update_favorites_route(user: &User, req: UpdateFavoritesReq) -> Result<()> {
    let user_favorites = match update_favorite_handler(&user, &req.favorites).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error creating favorite: {}", e);
            let err = anyhow!("Error creating favorite: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Users(UserRoute::UpdateFavorite),
                WsEvent::Users(UserEvent::UpdateFavorite),
                WsErrorCode::InternalServerError,
                "Failed to create favorite.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let create_favorite_message = WsResponseMessage::new(
        WsRoutes::Users(UserRoute::UpdateFavorite),
        WsEvent::Users(UserEvent::UpdateFavorite),
        user_favorites,
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

async fn update_favorite_handler(user: &User, favorites: &Vec<Uuid>) -> Result<Vec<FavoriteEnum>> {
    match update_favorites(user, &favorites).await {
        Ok(_) => (),
        Err(e) => return Err(e),
    };

    let user_favorites = match list_user_favorites(user).await {
        Ok(favorites) => favorites,
        Err(e) => return Err(e),
    };

    Ok(user_favorites)
}
