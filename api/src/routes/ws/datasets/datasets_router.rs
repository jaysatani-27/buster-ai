
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::models::User;

use super::{
    delete_dataset::delete_dataset, get_dataset::get_dataset, list_datasets::list_datasets,
    post_dataset::post_dataset, update_dataset::update_dataset,
    updated_dataset_column::update_dataset_column,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum DatasetRoute {
    #[serde(rename = "/datasets/list")]
    List,
    #[serde(rename = "/datasets/get")]
    Get,
    #[serde(rename = "/datasets/post")]
    Post,
    #[serde(rename = "/datasets/update")]
    Update,
    #[serde(rename = "/datasets/delete")]
    Delete,
    #[serde(rename = "/datasets/column/update")]
    UpdateColumn,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DatasetEvent {
    ListDatasetsAdmin,
    ListDatasets,
    PostDataset,
    GetDataset,
    UpdateDataset,
    DeleteDatasets,
    UpdateDatasetColumn,
}

pub async fn datasets_router(route: DatasetRoute, data: Value, user: &User) -> Result<()> {
    match route {
        DatasetRoute::List => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            list_datasets(user, req).await?;
        }
        DatasetRoute::Get => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            get_dataset(user, req).await?;
        }
        DatasetRoute::Post => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            post_dataset(user, req).await?;
        }
        DatasetRoute::Update => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            update_dataset(user, req).await?;
        }
        DatasetRoute::Delete => {
            let req = serde_json::from_value(data)?;

            delete_dataset(user, req).await?;
        }
        DatasetRoute::UpdateColumn => {
            let req = serde_json::from_value(data)?;

            update_dataset_column(user, req).await?;
        }
    };

    Ok(())
}

impl DatasetRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/datasets/list" => Ok(Self::List),
            "/datasets/get" => Ok(Self::Get),
            "/datasets/post" => Ok(Self::Post),
            "/datasets/update" => Ok(Self::Update),
            "/datasets/delete" => Ok(Self::Delete),
            "/datasets/column/update" => Ok(Self::UpdateColumn),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
