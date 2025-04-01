
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::models::User;

use super::{post_organization::post_organization, update_organization::update_organization};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum OrganizationRoute {
    #[serde(rename = "/organizations/post")]
    Post,
    #[serde(rename = "/organizations/update")]
    Update,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OrganizationEvent {
    Post,
    Update,
}

pub async fn organizations_router(
    route: OrganizationRoute,
    data: Value,
    user: &User,
) -> Result<()> {
    match route {
        OrganizationRoute::Post => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            post_organization(user, req).await?;
        }
        OrganizationRoute::Update => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            update_organization(user, req).await?;
        }
    };

    Ok(())
}

impl OrganizationRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/organizations/post" => Ok(Self::Post),
            "/organizations/update" => Ok(Self::Update),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
