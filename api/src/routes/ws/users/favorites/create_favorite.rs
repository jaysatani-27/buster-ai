use anyhow::{anyhow, Result};

use diesel::{insert_into, update, upsert::excluded, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    database::{
        enums::AssetType,
        lib::get_pg_pool,
        models::{User, UserFavorite},
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
    pub asset_type: AssetType,
    pub index: Option<usize>,
}

pub async fn create_favorite(user: &User, req: CreateFavoriteReq) -> Result<()> {
    let create_favorite_res =
        match create_favorite_handler(&user, &req.id, &req.asset_type, req.index).await {
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

async fn create_favorite_handler(
    user: &User,
    id: &Uuid,
    asset_type: &AssetType,
    index: Option<usize>,
) -> Result<Vec<FavoriteEnum>> {
    let index = index.unwrap_or(0);

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    match update(user_favorites::table)
        .set(user_favorites::order_index.eq(user_favorites::order_index + 1))
        .filter(user_favorites::user_id.eq(user.id))
        .filter(user_favorites::order_index.ge(index as i32))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error updating user favorite: {}", e)),
    };

    let user_favorite = UserFavorite {
        asset_type: asset_type.clone(),
        user_id: user.id,
        asset_id: *id,
        order_index: index as i32,
        created_at: chrono::Utc::now(),
        deleted_at: None,
    };

    match insert_into(user_favorites::table)
        .values(user_favorite)
        .on_conflict((
            user_favorites::user_id,
            user_favorites::asset_id,
            user_favorites::asset_type,
        ))
        .do_update()
        .set((
            user_favorites::deleted_at.eq(excluded(user_favorites::deleted_at)),
            user_favorites::order_index.eq(excluded(user_favorites::order_index)),
        ))
        .execute(&mut conn)
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error inserting or updating user favorite: {}", e)),
    };

    let user_favorites = match list_user_favorites(user).await {
        Ok(favorites) => favorites,
        Err(e) => return Err(e),
    };

    Ok(user_favorites)
}
