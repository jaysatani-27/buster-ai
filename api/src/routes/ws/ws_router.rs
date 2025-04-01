use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    database::models::User,
    routes::ws::{
        dashboards::dashboards_router::dashboards_router,
        datasets::datasets_router::datasets_router,
    },
};

use super::{
    collections::collections_router::{collections_router, CollectionRoute},
    dashboards::dashboards_router::DashboardRoute,
    data_sources::data_sources_router::{data_sources_router, DataSourceRoute},
    datasets::datasets_router::DatasetRoute,
    organizations::organization_router::{organizations_router, OrganizationRoute},
    permissions::permissions_router::{permissions_router, PermissionRoute},
    search::search_router::{search_router, SearchRoute},
    sql::sql_router::{sql_router, SqlRoute},
    teams::teams_routes::{teams_router, TeamRoute},
    terms::terms_router::{terms_router, TermRoute},
    threads_and_messages::threads_router::{threads_router, ThreadRoute},
    users::users_router::{users_router, UserRoute},
    ws::SubscriptionRwLock,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum WsRoutes {
    Threads(ThreadRoute),
    Dashboards(DashboardRoute),
    Datasets(DatasetRoute),
    Users(UserRoute),
    Collections(CollectionRoute),
    Sql(SqlRoute),
    Teams(TeamRoute),
    DataSources(DataSourceRoute),
    Permissions(PermissionRoute),
    Terms(TermRoute),
    Search(SearchRoute),
    Organizations(OrganizationRoute),
}

impl WsRoutes {
    pub fn from_str(path: &str) -> Result<Self> {
        let first_segment = path
            .split('/')
            .nth(1)
            .ok_or_else(|| anyhow!("Invalid path"))?;

        match first_segment {
            "threads" => Ok(Self::Threads(ThreadRoute::from_str(path)?)),
            "dashboards" => Ok(Self::Dashboards(DashboardRoute::from_str(path)?)),
            "datasets" => Ok(Self::Datasets(DatasetRoute::from_str(path)?)),
            "users" => Ok(Self::Users(UserRoute::from_str(path)?)),
            "collections" => Ok(Self::Collections(CollectionRoute::from_str(path)?)),
            "sql" => Ok(Self::Sql(SqlRoute::from_str(path)?)),
            "teams" => Ok(Self::Teams(TeamRoute::from_str(path)?)),
            "data_sources" => Ok(Self::DataSources(DataSourceRoute::from_str(path)?)),
            "permissions" => Ok(Self::Permissions(PermissionRoute::from_str(path)?)),
            "terms" => Ok(Self::Terms(TermRoute::from_str(path)?)),
            "search" => Ok(Self::Search(SearchRoute::from_str(path)?)),
            "organizations" => Ok(Self::Organizations(OrganizationRoute::from_str(path)?)),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}

pub async fn ws_router(
    route: String,
    payload: Value,
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
) -> Result<()> {
    let parsed_route: WsRoutes = match WsRoutes::from_str(&route) {
        Ok(parsed_route) => parsed_route,
        Err(e) => {
            return Err(anyhow!("Error parsing route: {:?}", e));
        }
    };

    let result = match parsed_route {
        WsRoutes::Threads(threads_route) => {
            threads_router(threads_route, payload, subscriptions, user_group, user).await
        }
        WsRoutes::Dashboards(dashboards_route) => {
            dashboards_router(dashboards_route, payload, subscriptions, user_group, user).await
        }
        WsRoutes::Datasets(datasets_route) => datasets_router(datasets_route, payload, user).await,
        WsRoutes::Permissions(permissions_route) => {
            permissions_router(permissions_route, payload, user).await
        }
        WsRoutes::Users(users_route) => users_router(users_route, payload, user).await,
        WsRoutes::Collections(collections_route) => {
            collections_router(collections_route, payload, subscriptions, user_group, user).await
        }
        WsRoutes::Sql(sql_route) => sql_router(sql_route, payload, user).await,
        WsRoutes::Teams(teams_route) => teams_router(teams_route, payload, user).await,
        WsRoutes::DataSources(data_sources_route) => {
            data_sources_router(data_sources_route, payload, user).await
        }
        WsRoutes::Terms(terms_route) => terms_router(terms_route, payload, user).await,
        WsRoutes::Search(search_route) => search_router(search_route, payload, user).await,
        WsRoutes::Organizations(organizations_route) => {
            organizations_router(organizations_route, payload, user).await
        }
    };

    if let Err(e) = result {
        tracing::error!("Error: {}", e);
    }

    Ok(())
}
