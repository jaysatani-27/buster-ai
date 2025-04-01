use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::Row;
use tokio_stream::StreamExt;
use uuid::Uuid;

use serde::Deserialize;

use crate::database::lib::get_sqlx_pool;
use crate::database::models::User;
use crate::routes::ws::ws::WsSendMethod;
use crate::utils::search_engine::search_engine::{
    MessageSearchResult, SearchObject, SearchObjectType,
};
use crate::utils::user::user_info::get_user_organization_id;

#[derive(Deserialize)]
pub struct SearchReq {
    pub query: String,
    pub num_results: Option<i64>,
    pub exclude_threads: Option<bool>,
    pub exclude_collections: Option<bool>,
    pub exclude_dashboards: Option<bool>,
    pub exclude_data_sources: Option<bool>,
    pub exclude_datasets: Option<bool>,
    pub exclude_permission_groups: Option<bool>,
    pub exclude_teams: Option<bool>,
    pub exclude_terms: Option<bool>,
}

use crate::{
    routes::ws::{
        search::search_router::{SearchEvent, SearchRoute},
        ws::{WsErrorCode, WsEvent, WsResponseMessage},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::sentry_utils::send_sentry_error,
        search_engine::search_engine::{search_engine, SearchOptions},
    },
};

pub async fn search(user: &User, req: SearchReq) -> Result<()> {
    let search_res = match search_handler(user.id, req).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error performing search: {}", e);
            let err = anyhow!("Error performing search: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Search(SearchRoute::Search),
                WsEvent::Search(SearchEvent::Search),
                WsErrorCode::InternalServerError,
                "Failed to perform search.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let search_message = WsResponseMessage::new(
        WsRoutes::Search(SearchRoute::Search),
        WsEvent::Search(SearchEvent::Search),
        search_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &search_message).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error sending ws message: {}", e);
            let err = anyhow!("Error sending ws message: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(err);
        }
    }

    Ok(())
}

async fn search_handler(user_id: Uuid, req: SearchReq) -> Result<Vec<SearchObject>> {
    let num_results = req.num_results.unwrap_or(25);

    let mut asset_types = vec![];

    if !req.exclude_threads.unwrap_or(false) {
        asset_types.push(SearchObjectType::Thread);
    }

    if !req.exclude_collections.unwrap_or(false) {
        asset_types.push(SearchObjectType::Collection);
    }
    if !req.exclude_dashboards.unwrap_or(false) {
        asset_types.push(SearchObjectType::Dashboard);
    }
    if !req.exclude_data_sources.unwrap_or(false) {
        asset_types.push(SearchObjectType::DataSource);
    }
    if !req.exclude_datasets.unwrap_or(false) {
        asset_types.push(SearchObjectType::Dataset);
    }
    if !req.exclude_permission_groups.unwrap_or(false) {
        asset_types.push(SearchObjectType::PermissionGroup);
    }
    if !req.exclude_teams.unwrap_or(false) {
        asset_types.push(SearchObjectType::Team);
    }
    if !req.exclude_terms.unwrap_or(false) {
        asset_types.push(SearchObjectType::Term);
    }

    let results = if req.query.is_empty() {
        list_assets_handler(user_id, asset_types, num_results).await?
    } else {
        let options =
            SearchOptions::with_custom_options(req.num_results.unwrap_or(25), asset_types);

        let user_organization_id = match get_user_organization_id(&user_id).await {
            Ok(organization_id) => organization_id,
            Err(e) => {
                return Err(anyhow!("Error getting user organization id: {}", e));
            }
        };

        search_engine(user_id, user_organization_id, req.query.clone(), options).await?
    };

    let filtered_results: Vec<SearchObject> = if req.query.is_empty() {
        results
    } else {
        results
            .into_iter()
            .filter(|result| match result {
                SearchObject::Message(message) => !message.highlights.is_empty(),
                SearchObject::Dashboard(dashboard) => !dashboard.highlights.is_empty(),
                SearchObject::Dataset(dataset) => !dataset.highlights.is_empty(),
                SearchObject::PermissionGroup(permission_group) => {
                    !permission_group.highlights.is_empty()
                }
                SearchObject::Team(team) => !team.highlights.is_empty(),
                SearchObject::DataSource(data_source) => !data_source.highlights.is_empty(),
                SearchObject::Term(term) => !term.highlights.is_empty(),
                SearchObject::Collection(collection) => !collection.highlights.is_empty(),
            })
            .collect()
    };

    Ok(filtered_results)
}

pub async fn list_assets_handler(
    user_id: Uuid,
    asset_types: Vec<SearchObjectType>,
    num_results: i64,
) -> Result<Vec<SearchObject>> {
    let mut conn = get_sqlx_pool().acquire().await?;

    let query = format!(
        r#"
        WITH distinct_assets AS (
            SELECT DISTINCT ON (content, asset_type)
                asset_search.asset_id,
                asset_search.content,
                asset_search.updated_at,
                asset_search.asset_type
            FROM 
                asset_search
            INNER JOIN
                asset_permissions
            ON 
                asset_search.asset_id = asset_permissions.asset_id
            WHERE 
                asset_search.asset_type IN ({})
                AND (asset_permissions.identity_id = '{}')
                AND asset_search.deleted_at IS NULL
                AND asset_permissions.deleted_at IS NULL
        )
        SELECT *
        FROM distinct_assets
        ORDER BY updated_at DESC
        LIMIT {};
        "#,
        asset_types
            .iter()
            .map(|t| format!("'{}'", t.to_string()))
            .collect::<Vec<_>>()
            .join(","),
        user_id,
        num_results
    );

    let mut results = sqlx::raw_sql(&query).fetch(&mut *conn);
    let mut results_vec = Vec::new();
    while let Some(row) = results.try_next().await? {
        let id: Uuid = match row.try_get("asset_id") {
            Ok(id) => id,
            Err(e) => return Err(anyhow!("Error getting asset_id: {:?}", e)),
        };
        let content: String = match row.try_get("content") {
            Ok(content) => content,
            Err(e) => return Err(anyhow!("Error getting content: {:?}", e)),
        };
        let updated_at: DateTime<Utc> = match row.try_get("updated_at") {
            Ok(updated_at) => updated_at,
            Err(e) => return Err(anyhow!("Error getting updated_at: {:?}", e)),
        };

        let asset_type: SearchObjectType = match row.try_get("asset_type") {
            Ok(asset_type) => match asset_type {
                "thread" => SearchObjectType::Thread,
                "collection" => SearchObjectType::Collection,
                "dashboard" => SearchObjectType::Dashboard,
                "data_source" => SearchObjectType::DataSource,
                "dataset" => SearchObjectType::Dataset,
                "permission_group" => SearchObjectType::PermissionGroup,
                "team" => SearchObjectType::Team,
                "term" => SearchObjectType::Term,
                _ => return Err(anyhow!("Invalid asset type: {:?}", asset_type)),
            },
            Err(e) => return Err(anyhow!("Error getting asset_type: {:?}", e)),
        };

        results_vec.push(SearchObject::Message(MessageSearchResult {
            id,
            title: content.clone(),
            updated_at,
            summary_question: content,
            highlights: vec![],
            score: 0.0,
            type_: asset_type,
        }));
    }
    let results = results_vec;

    Ok(results)
}
