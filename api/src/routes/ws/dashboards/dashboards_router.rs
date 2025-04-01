use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use uuid::Uuid;

use crate::{database::models::User, routes::ws::ws::SubscriptionRwLock};

use super::{
    delete_dashboard::delete_dashboard, get_dashboard::get_dashboard,
    list_dashboards::list_dashboards, post_dashboard::post_dashboard, unsubscribe::unsubscribe,
    update_dashboard::update_dashboard,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum DashboardRoute {
    #[serde(rename = "/dashboards/list")]
    List,
    #[serde(rename = "/dashboards/get")]
    Get,
    #[serde(rename = "/dashboards/post")]
    Post,
    #[serde(rename = "/dashboards/unsubscribe")]
    Unsubscribe,
    #[serde(rename = "/dashboards/update")]
    Update,
    #[serde(rename = "/dashboards/delete")]
    Delete,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DashboardEvent {
    Unsubscribed,
    LeftDashboard,
    GetDashboardsList,
    GetDashboardState,
    PostDashboard,
    FetchingData,
    UpdateDashboard,
    JoinedDashboard,
    DeleteDashboard,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JoinedDashboard {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeftDashboard {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub dashboard_id: Uuid,
}

pub async fn dashboards_router(
    route: DashboardRoute,
    data: Value,
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
) -> Result<()> {
    match route {
        DashboardRoute::List => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            list_dashboards(user, req).await?;
        }
        DashboardRoute::Get => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            get_dashboard(subscriptions, user_group, user, req).await?;
        }
        DashboardRoute::Unsubscribe => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            unsubscribe(subscriptions, user, user_group, req).await?;
        }
        DashboardRoute::Post => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            post_dashboard(subscriptions, user_group, user, req).await?;
        }
        DashboardRoute::Update => {
            println!("Request received in dashboards_router: {:?}", data);
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            update_dashboard(subscriptions, user_group, user, req).await?;
        }
        DashboardRoute::Delete => {
            let req = match serde_json::from_value(data) {
                Ok(req) => req,
                Err(e) => return Err(anyhow!("Error parsing request: {}", e)),
            };

            delete_dashboard(user, req).await?;
        }
    };

    Ok(())
}

impl DashboardRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/dashboards/list" => Ok(Self::List),
            "/dashboards/get" => Ok(Self::Get),
            "/dashboards/post" => Ok(Self::Post),
            "/dashboards/unsubscribe" => Ok(Self::Unsubscribe),
            "/dashboards/update" => Ok(Self::Update),
            "/dashboards/delete" => Ok(Self::Delete),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
