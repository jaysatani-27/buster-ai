use anyhow::{anyhow, Result};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde_json::Value;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        lib::{get_pg_pool, UserConfig},
        models::User,
        schema::users,
    },
    routes::ws::{
        users::users_router::{UserEvent, UserRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

pub async fn list_user_colors(user: &User) -> Result<()> {
    let user_subscription = format!("{}", &user.id);

    let list_color_palettes_res = match list_user_colors_handler(&user.id).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting threads: {}", e);
            let err = anyhow!("Error getting threads: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user_subscription,
                WsRoutes::Users(UserRoute::ListColorPalettes),
                WsEvent::Users(UserEvent::ListUserColorPalettes),
                WsErrorCode::InternalServerError,
                "Failed to list datasets.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let list_color_palettes_message = WsResponseMessage::new(
        WsRoutes::Users(UserRoute::ListColorPalettes),
        WsEvent::Users(UserEvent::ListUserColorPalettes),
        list_color_palettes_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user_subscription, &list_color_palettes_message).await {
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

#[derive(Serialize, Deserialize)]
pub struct ColorPaletteWithId {
    pub id: usize,
    pub palette: Vec<String>,
}

async fn list_user_colors_handler(user_id: &Uuid) -> Result<Vec<ColorPaletteWithId>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection: {}", e)),
    };

    let user_colors = match users::table
        .select(users::config)
        .filter(users::id.eq(user_id))
        .first::<Value>(&mut conn)
        .await
    {
        Ok(user_colors) => user_colors,
        Err(e) => {
            tracing::error!("Error getting user colors: {}", e);
            return Err(anyhow!("Error getting user colors: {}", e));
        }
    };

    let user_colors = match serde_json::from_value::<UserConfig>(user_colors) {
        Ok(user_colors) => user_colors,
        Err(e) => {
            tracing::error!("Error serializing user colors: {}", e);
            return Err(anyhow!("Error serializing user colors: {}", e));
        }
    };

    let color_palettes_with_ids: Vec<ColorPaletteWithId> =
        if let Some(color_palettes) = user_colors.color_palettes {
            color_palettes
                .into_iter()
                .enumerate()
                .map(|(index, palette)| ColorPaletteWithId {
                    id: index,
                    palette: palette,
                })
                .collect()
        } else {
            vec![]
        };

    Ok(color_palettes_with_ids)
}
