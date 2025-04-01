use anyhow::{anyhow, Result};

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

use super::favorites_utils::list_user_favorites;

pub async fn list_favorites(user: &User) -> Result<()> {
    let list_favorites_res = match list_user_favorites(user).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting collections: {}", e);
            let err = anyhow!("Error getting collections: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Users(UserRoute::ListFavorites),
                WsEvent::Users(UserEvent::ListFavorites),
                WsErrorCode::InternalServerError,
                "Failed to list collections.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let list_favorites_message = WsResponseMessage::new(
        WsRoutes::Users(UserRoute::ListFavorites),
        WsEvent::Users(UserEvent::ListFavorites),
        list_favorites_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &list_favorites_message).await {
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
