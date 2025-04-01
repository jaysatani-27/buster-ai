use anyhow::{anyhow, Result};
use diesel::{update, ExpressionMethods, QueryDsl};
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

use super::list_user_color_palettes::ColorPaletteWithId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateUserColorPaletteRequest {
    pub id: usize,
    pub color_palette: Vec<String>,
}

pub async fn update_user_color_palette(
    user: &User,
    req: UpdateUserColorPaletteRequest,
) -> Result<()> {
    let user_subscription = format!("{}", &user.id);

    let create_user_color_palette_res = match update_user_color_palette_handler(&user.id, req).await
    {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting threads: {}", e);
            let err = anyhow!("Error getting threads: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user_subscription,
                WsRoutes::Users(UserRoute::PostColorPalette),
                WsEvent::Users(UserEvent::CreateUserColorPalette),
                WsErrorCode::InternalServerError,
                "Failed to list datasets.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let create_user_color_palette_message = WsResponseMessage::new(
        WsRoutes::Users(UserRoute::PostColorPalette),
        WsEvent::Users(UserEvent::CreateUserColorPalette),
        create_user_color_palette_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user_subscription, &create_user_color_palette_message).await {
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

async fn update_user_color_palette_handler(
    user_id: &Uuid,
    req: UpdateUserColorPaletteRequest,
) -> Result<Vec<ColorPaletteWithId>> {
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
        Err(e) => return Err(anyhow!("Error getting user colors: {}", e)),
    };

    let mut user_config = match serde_json::from_value::<UserConfig>(user_colors) {
        Ok(user_colors) => user_colors,
        Err(e) => return Err(anyhow!("Error serializing user colors: {}", e)),
    };

    if let Some(color_palettes) = &mut user_config.color_palettes {
        color_palettes[req.id] = req.color_palette;
    } else {
        user_config.color_palettes = Some(vec![req.color_palette]);
    }

    let update_user_config = user_config.clone();
    let update_user_id = user_id.clone();

    tokio::spawn(async move {
        match update_user_color_palette_task(update_user_id, update_user_config).await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error updating user color palette: {}", e);
                send_sentry_error(&e.to_string(), Some(&update_user_id));
            }
        };
    });

    let color_palettes_with_ids: Vec<ColorPaletteWithId> =
        if let Some(color_palettes) = user_config.color_palettes {
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

async fn update_user_color_palette_task(user_id: Uuid, user_config: UserConfig) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection: {}", e)),
    };

    let user_config_json = match serde_json::to_value(user_config) {
        Ok(user_config_json) => user_config_json,
        Err(e) => return Err(anyhow!("Error serializing user config: {}", e)),
    };

    match update(users::table)
        .set(users::config.eq(user_config_json))
        .filter(users::id.eq(user_id))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error updating user color palette: {}", e)),
    }

    Ok(())
}
