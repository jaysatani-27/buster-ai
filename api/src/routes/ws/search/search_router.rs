
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::models::User;

use super::search::search;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum SearchRoute {
    #[serde(rename = "/search")]
    Search,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SearchEvent {
    Search,
}

pub async fn search_router(
    route: SearchRoute,
    data: Value,
    user: &User,
) -> Result<()> {
    match route {
        SearchRoute::Search => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            search(user, req).await?;
        }
    };

    Ok(())
}

impl SearchRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/search" => Ok(Self::Search),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
