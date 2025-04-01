
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::models::User;

use super::list_teams::list_teams;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TeamRoute {
    #[serde(rename = "/teams/list")]
    List,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TeamEvent {
    ListTeams,
}

pub async fn teams_router(route: TeamRoute, data: Value, user: &User) -> Result<()> {
    match route {
        TeamRoute::List => {
            let req = serde_json::from_value(data)?;

            list_teams(user, req).await?;
        }
    };

    Ok(())
}

impl TeamRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/teams/list" => Ok(Self::List),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
