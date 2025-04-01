use std::sync::Arc;

use anyhow::{anyhow, Result};
use diesel::{insert_into, upsert::excluded, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{
    enums::AssetType,
    lib::get_pg_pool,
    models::{User, UserFavorite},
    schema::{collections, collections_to_assets, dashboards, messages, threads, user_favorites},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FavoriteIdAndType {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub type_: AssetType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserFavoritesReq {
    pub favorites: Vec<FavoriteIdAndType>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CollectionFavorites {
    pub collection_id: Uuid,
    pub collection_name: String,
    pub assets: Vec<FavoriteObject>,
    #[serde(rename = "type")]
    pub type_: AssetType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FavoriteObject {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: AssetType,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum FavoriteEnum {
    Collection(CollectionFavorites),
    Object(FavoriteObject),
}

pub async fn list_user_favorites(user: &User) -> Result<Vec<FavoriteEnum>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let user_favorites = match user_favorites::table
        .select((user_favorites::asset_id, user_favorites::asset_type))
        .filter(user_favorites::user_id.eq(user.id))
        .filter(user_favorites::deleted_at.is_null())
        .order(user_favorites::order_index.asc())
        .load::<(Uuid, AssetType)>(&mut conn)
        .await
    {
        Ok(favorites) => favorites,
        Err(e) => return Err(anyhow!("Error loading user favorites: {:?}", e)),
    };

    let dashboard_favorites = {
        let dashboard_ids = Arc::new(
            user_favorites
                .iter()
                .filter(|(_, f)| f == &AssetType::Dashboard)
                .map(|f| f.0)
                .collect::<Vec<Uuid>>(),
        );
        tokio::spawn(async move { get_favorite_dashboards(dashboard_ids) })
    };

    let collection_favorites = {
        let collection_ids = Arc::new(
            user_favorites
                .iter()
                .filter(|(_, f)| f == &AssetType::Collection)
                .map(|f| f.0)
                .collect::<Vec<Uuid>>(),
        );
        tokio::spawn(async move { get_assets_from_collections(collection_ids) })
    };
    let threads_favorites = {
        let thread_ids = Arc::new(
            user_favorites
                .iter()
                .filter(|(_, f)| f == &AssetType::Thread)
                .map(|f| f.0)
                .collect::<Vec<Uuid>>(),
        );
        tokio::spawn(async move { get_favorite_threads(thread_ids) })
    };

    let (dashboard_fav_res, collection_fav_res, threads_fav_res) =
        match tokio::try_join!(dashboard_favorites, collection_favorites, threads_favorites) {
            Ok((dashboard_fav_res, collection_fav_res, threads_fav_res)) => {
                (dashboard_fav_res, collection_fav_res, threads_fav_res)
            }
            Err(e) => {
                tracing::error!("Error getting favorite assets: {}", e);
                return Err(anyhow!("Error getting favorite assets: {}", e));
            }
        };

    let favorite_dashboards = match dashboard_fav_res.await {
        Ok(dashboards) => dashboards,
        Err(e) => {
            tracing::error!("Error getting favorite dashboards: {}", e);
            return Err(anyhow!("Error getting favorite dashboards: {}", e));
        }
    };

    let favorite_collections = match collection_fav_res.await {
        Ok(collections) => collections,
        Err(e) => {
            tracing::error!("Error getting favorite collections: {}", e);
            return Err(anyhow!("Error getting favorite collections: {}", e));
        }
    };

    let favorite_threads = match threads_fav_res.await {
        Ok(threads) => threads,
        Err(e) => {
            tracing::error!("Error getting favorite threads: {}", e);
            return Err(anyhow!("Error getting favorite threads: {}", e));
        }
    };

    let mut favorites: Vec<FavoriteEnum> = Vec::with_capacity(user_favorites.len());

    for favorite in &user_favorites {
        match favorite.1 {
            AssetType::Dashboard => {
                if let Some(dashboard) = favorite_dashboards.iter().find(|d| d.id == favorite.0) {
                    favorites.push(FavoriteEnum::Object(dashboard.clone()));
                }
            }
            AssetType::Collection => {
                if let Some(collection) = favorite_collections
                    .iter()
                    .find(|c| c.collection_id == favorite.0)
                {
                    favorites.push(FavoriteEnum::Collection(collection.clone()));
                }
            }
            AssetType::Thread => {
                if let Some(thread) = favorite_threads.iter().find(|t| t.id == favorite.0) {
                    favorites.push(FavoriteEnum::Object(thread.clone()));
                }
            }
        }
    }

    Ok(favorites)
}

async fn get_favorite_threads(thread_ids: Arc<Vec<Uuid>>) -> Result<Vec<FavoriteObject>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let thread_records: Vec<(Uuid, Option<String>)> = match threads::table
        .inner_join(messages::table.on(threads::id.eq(messages::thread_id)))
        .select((threads::id, messages::title))
        .filter(threads::id.eq_any(thread_ids.as_ref()))
        .filter(threads::deleted_at.is_null())
        .filter(messages::deleted_at.is_null())
        .filter(messages::draft_session_id.is_null())
        .distinct_on(threads::id)
        .order((threads::id, messages::created_at.desc()))
        .load::<(Uuid, Option<String>)>(&mut conn)
        .await
    {
        Ok(thread_records) => thread_records,
        Err(diesel::NotFound) => return Err(anyhow!("Threads not found")),
        Err(e) => return Err(anyhow!("Error loading thread records: {:?}", e)),
    };

    let favorite_threads = thread_records
        .iter()
        .map(|(id, name)| FavoriteObject {
            id: id.clone(),
            name: name.clone().unwrap_or_else(|| String::from("Untitled")),
            type_: AssetType::Thread,
        })
        .collect();

    Ok(favorite_threads)
}

async fn get_favorite_dashboards(dashboard_ids: Arc<Vec<Uuid>>) -> Result<Vec<FavoriteObject>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let dashboard_records: Vec<(Uuid, String)> = match dashboards::table
        .select((dashboards::id, dashboards::name))
        .filter(dashboards::id.eq_any(dashboard_ids.as_ref()))
        .filter(dashboards::deleted_at.is_null())
        .load::<(Uuid, String)>(&mut conn)
        .await
    {
        Ok(dashboard_records) => dashboard_records,
        Err(diesel::NotFound) => return Err(anyhow!("Dashboards not found")),
        Err(e) => return Err(anyhow!("Error loading dashboard records: {:?}", e)),
    };

    let favorite_dashboards = dashboard_records
        .iter()
        .map(|(id, name)| FavoriteObject {
            id: id.clone(),
            name: name.clone(),
            type_: AssetType::Dashboard,
        })
        .collect();
    Ok(favorite_dashboards)
}

async fn get_assets_from_collections(
    collection_ids: Arc<Vec<Uuid>>,
) -> Result<Vec<CollectionFavorites>> {
    let dashboards_handle = {
        let collection_ids = Arc::clone(&collection_ids);
        tokio::spawn(async move { get_dashboards_from_collections(&collection_ids).await })
    };

    let threads_handle = {
        let collection_ids = Arc::clone(&collection_ids);
        tokio::spawn(async move { get_threads_from_collections(&collection_ids).await })
    };

    let collection_name_handle = {
        let collection_ids = Arc::clone(&collection_ids);
        tokio::spawn(async move { get_collection_names(&collection_ids).await })
    };

    let (dashboards_res, threads_res, collection_name_res) =
        match tokio::join!(dashboards_handle, threads_handle, collection_name_handle) {
            (Ok(dashboards), Ok(threads), Ok(collection_name)) => {
                (dashboards, threads, collection_name)
            }
            _ => {
                return Err(anyhow!(
                    "Error getting dashboards or threads from collection"
                ))
            }
        };

    let dashboards = match dashboards_res {
        Ok(dashboards) => dashboards,
        Err(e) => return Err(anyhow!("Error getting dashboards from collection: {:?}", e)),
    };

    let threads = match threads_res {
        Ok(threads) => threads,
        Err(e) => return Err(anyhow!("Error getting threads from collection: {:?}", e)),
    };

    let collection_names = match collection_name_res {
        Ok(collection_names) => collection_names,
        Err(e) => return Err(anyhow!("Error getting collection name: {:?}", e)),
    };

    let mut collection_favorites: Vec<CollectionFavorites> = Vec::new();

    for (collection_id, collection_name) in collection_names {
        let mut assets = Vec::new();

        assets.extend(
            dashboards
                .iter()
                .filter_map(|(dash_collection_id, favorite_object)| {
                    if *dash_collection_id == collection_id {
                        Some(favorite_object.clone())
                    } else {
                        None
                    }
                }),
        );

        assets.extend(
            threads
                .iter()
                .filter_map(|(thread_collection_id, favorite_object)| {
                    if *thread_collection_id == collection_id {
                        Some(favorite_object.clone())
                    } else {
                        None
                    }
                }),
        );

        collection_favorites.push(CollectionFavorites {
            collection_id,
            collection_name,
            assets,
            type_: AssetType::Collection,
        });
    }

    Ok(collection_favorites)
}

async fn get_collection_names(collection_ids: &Vec<Uuid>) -> Result<Vec<(Uuid, String)>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let collection_names = match collections::table
        .select((collections::id, collections::name))
        .filter(collections::id.eq_any(collection_ids))
        .filter(collections::deleted_at.is_null())
        .load::<(Uuid, String)>(&mut conn)
        .await
    {
        Ok(collection_names) => collection_names,
        Err(e) => return Err(anyhow!("Error loading collection name: {:?}", e)),
    };
    Ok(collection_names)
}

async fn get_dashboards_from_collections(
    collection_ids: &Vec<Uuid>,
) -> Result<Vec<(Uuid, FavoriteObject)>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let dashboard_records: Vec<(Uuid, Uuid, String)> = match dashboards::table
        .inner_join(
            collections_to_assets::table.on(dashboards::id.eq(collections_to_assets::asset_id)),
        )
        .select((
            collections_to_assets::collection_id,
            dashboards::id,
            dashboards::name,
        ))
        .filter(collections_to_assets::collection_id.eq_any(collection_ids))
        .filter(collections_to_assets::asset_type.eq(AssetType::Dashboard))
        .filter(dashboards::deleted_at.is_null())
        .filter(collections_to_assets::deleted_at.is_null())
        .load::<(Uuid, Uuid, String)>(&mut conn)
        .await
    {
        Ok(dashboard_records) => dashboard_records,
        Err(e) => return Err(anyhow!("Error loading dashboard records: {:?}", e)),
    };

    let dashboard_objects: Vec<(Uuid, FavoriteObject)> = dashboard_records
        .iter()
        .map(|(collection_id, id, name)| {
            (
                *collection_id,
                FavoriteObject {
                    id: *id,
                    name: name.clone(),
                    type_: AssetType::Dashboard,
                },
            )
        })
        .collect();
    Ok(dashboard_objects)
}

async fn get_threads_from_collections(
    collection_ids: &Vec<Uuid>,
) -> Result<Vec<(Uuid, FavoriteObject)>> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let threads_records: Vec<(Uuid, Uuid, Option<String>)> = match threads::table
        .inner_join(
            collections_to_assets::table.on(threads::id.eq(collections_to_assets::asset_id)),
        )
        .inner_join(messages::table.on(threads::id.eq(messages::thread_id)))
        .select((
            collections_to_assets::collection_id,
            threads::id,
            messages::title,
        ))
        .filter(collections_to_assets::asset_type.eq(AssetType::Thread))
        .filter(collections_to_assets::collection_id.eq_any(collection_ids))
        .filter(threads::deleted_at.is_null())
        .filter(collections_to_assets::deleted_at.is_null())
        .filter(messages::deleted_at.is_null())
        .filter(messages::draft_session_id.is_null())
        .order((threads::id, messages::created_at.desc()))
        .distinct_on(threads::id)
        .load::<(Uuid, Uuid, Option<String>)>(&mut conn)
        .await
    {
        Ok(threads_records) => threads_records,
        Err(e) => return Err(anyhow!("Error loading threads records: {:?}", e)),
    };

    let thread_objects: Vec<(Uuid, FavoriteObject)> = threads_records
        .iter()
        .map(|(collection_id, id, name)| {
            (
                collection_id.clone(),
                FavoriteObject {
                    id: id.clone(),
                    name: name.clone().unwrap_or_else(|| String::from("Untitled")),
                    type_: AssetType::Thread,
                },
            )
        })
        .collect();
    Ok(thread_objects)
}

pub async fn update_favorites(user: &User, favorites: &Vec<Uuid>) -> Result<()> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection from pool: {:?}", e)),
    };

    let favorite_records = match user_favorites::table
        .select((user_favorites::asset_id, user_favorites::asset_type))
        .filter(user_favorites::user_id.eq(user.id))
        .filter(user_favorites::deleted_at.is_null())
        .load::<(Uuid, AssetType)>(&mut conn)
        .await
    {
        Ok(favorites) => favorites,
        Err(e) => return Err(anyhow!("Error loading favorites: {:?}", e)),
    };

    // Create a map of asset_id to AssetType
    let favorite_map: std::collections::HashMap<Uuid, AssetType> =
        favorite_records.into_iter().collect();

    let mut new_favs = vec![];

    // Iterate through the favorites in the order they were provided
    for (index, favorite_id) in favorites.iter().enumerate() {
        if let Some(asset_type) = favorite_map.get(favorite_id) {
            let new_fav = UserFavorite {
                user_id: user.id,
                asset_id: *favorite_id,
                asset_type: asset_type.clone(),
                order_index: index as i32,
                created_at: chrono::Utc::now(),
                deleted_at: None,
            };
            new_favs.push(new_fav);
        }
    }

    match insert_into(user_favorites::table)
        .values(new_favs)
        .on_conflict((
            user_favorites::user_id,
            user_favorites::asset_id,
            user_favorites::asset_type,
        ))
        .do_update()
        .set(user_favorites::order_index.eq(excluded(user_favorites::order_index)))
        .execute(&mut conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Error updating favorites: {:?}", e)),
    }
}
