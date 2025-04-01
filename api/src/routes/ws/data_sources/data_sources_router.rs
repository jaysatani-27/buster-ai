
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::database::models::User;

use super::{
    delete_data_source::delete_data_source, get_data_source::get_data_source,
    list_data_sources::list_data_sources, post_data_source::post_data_source,
    update_data_source::update_data_source,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum DataSourceRoute {
    #[serde(rename = "/data_sources/list")]
    List,
    #[serde(rename = "/data_sources/get")]
    Get,
    #[serde(rename = "/data_sources/post")]
    Post,
    #[serde(rename = "/data_sources/update")]
    Update,
    #[serde(rename = "/data_sources/delete")]
    Delete,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DataSourceEvent {
    ListDataSources,
    GetDataSource,
    DeleteDataSource,
    UpdateDataSource,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JoinedDataSource {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeftDataSource {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub dashboard_id: Uuid,
}

pub async fn data_sources_router(route: DataSourceRoute, data: Value, user: &User) -> Result<()> {
    match route {
        DataSourceRoute::List => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            list_data_sources(user, req).await?;
        }
        DataSourceRoute::Get => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            get_data_source(user, req).await?;
        }
        DataSourceRoute::Post => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => {
                    tracing::error!("Error parsing request: {}", e);
                    return Err(anyhow!("Error parsing request: {}", e));
                }
            };

            post_data_source(user, req).await?;
        }
        DataSourceRoute::Update => {
            let req = serde_json::from_value(data)?;

            update_data_source(user, req).await?;
        }
        DataSourceRoute::Delete => {
            let req = serde_json::from_value(data)?;

            delete_data_source(user, req).await?;
        }
    };

    Ok(())
}

impl DataSourceRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/data_sources/list" => Ok(Self::List),
            "/data_sources/get" => Ok(Self::Get),
            "/data_sources/post" => Ok(Self::Post),
            "/data_sources/update" => Ok(Self::Update),
            "/data_sources/delete" => Ok(Self::Delete),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
