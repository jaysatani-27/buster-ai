use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{database::models::User, routes::ws::ws::SubscriptionRwLock};

use super::{
    delete_collection::delete_collection, get_collection::get_collection,
    list_collections::list_collections, post_collection::post_collection, unsubscribe::unsubscribe,
    update_collection::update_collection,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum CollectionRoute {
    #[serde(rename = "/collections/list")]
    List,
    #[serde(rename = "/collections/get")]
    Get,
    #[serde(rename = "/collections/post")]
    Post,
    #[serde(rename = "/collections/unsubscribe")]
    Unsubscribe,
    #[serde(rename = "/collections/update")]
    Update,
    #[serde(rename = "/collections/delete")]
    Delete,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum CollectionEvent {
    ListCollections,
    CollectionState,
    DeleteCollections,
    Unsubscribed,
}

pub async fn collections_router(
    route: CollectionRoute,
    data: Value,
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
) -> Result<()> {
    match route {
        CollectionRoute::List => {
            let req = serde_json::from_value(data)?;

            list_collections(user, req).await?;
        }
        CollectionRoute::Get => {
            let req = serde_json::from_value(data)?;

            get_collection(subscriptions, user_group, user, req).await?;
        }
        CollectionRoute::Unsubscribe => {
            let req = serde_json::from_value(data)?;

            unsubscribe(subscriptions, user, user_group, req).await?;
        }
        CollectionRoute::Post => {
            let req = serde_json::from_value(data)?;

            post_collection(subscriptions, user_group, user, req).await?;
        }
        CollectionRoute::Update => {
            let req = serde_json::from_value(data)?;

            update_collection(subscriptions, user_group, user, req).await?;
        }
        CollectionRoute::Delete => {
            let req = serde_json::from_value(data)?;

            delete_collection(user, req).await?;
        }
    };

    Ok(())
}

impl CollectionRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/collections/list" => Ok(Self::List),
            "/collections/get" => Ok(Self::Get),
            "/collections/post" => Ok(Self::Post),
            "/collections/unsubscribe" => Ok(Self::Unsubscribe),
            "/collections/update" => Ok(Self::Update),
            "/collections/delete" => Ok(Self::Delete),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
