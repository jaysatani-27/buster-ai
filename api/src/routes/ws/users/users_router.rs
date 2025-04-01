
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::models::User;

use super::{
    color_palettes::{
        create_user_color_palette::create_user_color_palette,
        delete_user_color_palette::delete_user_color_palette,
        list_user_color_palettes::list_user_colors,
        update_user_color_palette::update_user_color_palette,
    },
    favorites::{
        create_favorite::create_favorite, delete_favorite::delete_favorite,
        list_favorites::list_favorites, update_favorites::update_favorites_route,
    },
    invite_users::invite_users,
    list_users::list_users,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum UserRoute {
    #[serde(rename = "/users/get")]
    Get,
    #[serde(rename = "/users/list")]
    List,
    #[serde(rename = "/users/invite")]
    Invite,
    #[serde(rename = "/users/colors/list")]
    ListColorPalettes,
    #[serde(rename = "/users/colors/post")]
    PostColorPalette,
    #[serde(rename = "/users/colors/delete")]
    DeleteColorPalette,
    #[serde(rename = "/users/colors/update")]
    UpdateColorPalette,
    #[serde(rename = "/users/favorites/list")]
    ListFavorites,
    #[serde(rename = "/users/favorites/post")]
    CreateFavorite,
    #[serde(rename = "/users/favorites/delete")]
    DeleteFavorite,
    #[serde(rename = "/users/favorites/update")]
    UpdateFavorite,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum UserEvent {
    ListUserColorPalettes,
    CreateUserColorPalette,
    ListFavorites,
    CreateFavorite,
    DeleteFavorite,
    UpdateFavorite,
    GetUser,
    ListUsers,
}

pub async fn users_router(
    route: UserRoute,
    data: Value,
    user: &User,
) -> Result<()> {
    match route {
        UserRoute::Get => {
            // let req = serde_json::from_value(data)?;

            // get_user(user, req).await?;
        }
        UserRoute::List => {
            let req = serde_json::from_value(data)?;

            list_users(user, req).await?;
        }
        UserRoute::ListColorPalettes => {
            list_user_colors(user).await?;
        }
        UserRoute::PostColorPalette => {
            let req = serde_json::from_value(data)?;

            create_user_color_palette(user, req).await?;
        }
        UserRoute::DeleteColorPalette => {
            let req = serde_json::from_value(data)?;

            delete_user_color_palette(user, req).await?;
        }
        UserRoute::UpdateColorPalette => {
            let req = serde_json::from_value(data)?;

            update_user_color_palette(user, req).await?;
        }
        UserRoute::ListFavorites => {
            list_favorites(user).await?;
        }
        UserRoute::CreateFavorite => {
            let req = serde_json::from_value(data)?;

            create_favorite(user, req).await?;
        }
        UserRoute::DeleteFavorite => {
            let req = serde_json::from_value(data)?;

            delete_favorite(user, req).await?;
        }
        UserRoute::UpdateFavorite => {
            let req = serde_json::from_value(data)?;

            update_favorites_route(user, req).await?;
        }
        UserRoute::Invite => {
            let req = serde_json::from_value(data)?;

            invite_users(user, req).await?;
        }
    };

    Ok(())
}

impl UserRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/users/list" => Ok(Self::List),
            "/users/get" => Ok(Self::Get),
            "/users/colors/list" => Ok(Self::ListColorPalettes),
            "/users/colors/post" => Ok(Self::PostColorPalette),
            "/users/colors/delete" => Ok(Self::DeleteColorPalette),
            "/users/colors/update" => Ok(Self::UpdateColorPalette),
            "/users/favorites/list" => Ok(Self::ListFavorites),
            "/users/favorites/post" => Ok(Self::CreateFavorite),
            "/users/favorites/delete" => Ok(Self::DeleteFavorite),
            "/users/favorites/update" => Ok(Self::UpdateFavorite),
            "/users/invite" => Ok(Self::Invite),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
