
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::models::User;

use super::{
    delete_term::delete_term, get_term::get_term, list_terms::list_terms, post_term::post_term,
    update_term::update_term,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TermRoute {
    #[serde(rename = "/terms/list")]
    List,
    #[serde(rename = "/terms/get")]
    Get,
    #[serde(rename = "/terms/post")]
    Post,
    #[serde(rename = "/terms/update")]
    Update,
    #[serde(rename = "/terms/delete")]
    Delete,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TermEvent {
    ListTerms,
    GetTerm,
    PostTerm,
    UpdateTerm,
    DeleteTerm,
}

pub async fn terms_router(
    route: TermRoute,
    data: Value,
    user: &User,
) -> Result<()> {
    match route {
        TermRoute::List => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            list_terms(user, req).await?;
        }
        TermRoute::Get => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            get_term(user, req).await?;
        }
        TermRoute::Post => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            post_term(user, req).await?;
        }
        TermRoute::Update => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            update_term(user, req).await?;
        }
        TermRoute::Delete => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            delete_term(user, req).await?;
        }
    };

    Ok(())
}

impl TermRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/terms/list" => Ok(Self::List),
            "/terms/get" => Ok(Self::Get),
            "/terms/post" => Ok(Self::Post),
            "/terms/update" => Ok(Self::Update),
            "/terms/delete" => Ok(Self::Delete),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
