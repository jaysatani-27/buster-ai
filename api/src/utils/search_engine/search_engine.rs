use sqlx::Row;
use std::sync::Arc;
use tokio_stream::StreamExt;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    database::{
        enums::{AssetType, IdentityType, UserOrganizationRole},
        lib::{get_pg_pool, get_sqlx_pool, PgPool},
        schema::{
            asset_permissions, collections, dashboards, data_sources, datasets,
            datasets_to_permission_groups, messages, organizations, permission_groups,
            permission_groups_to_identities, teams, teams_to_users, terms, users,
            users_to_organizations,
        },
    },
    utils::clients::typesense::{self, CollectionName, Document, SearchRequestObject},
};

#[derive(Serialize, Debug)]
pub struct MessageSearchResult {
    pub id: Uuid,
    #[serde(rename = "name")]
    pub title: String,
    pub summary_question: String,
    pub updated_at: DateTime<Utc>,
    pub highlights: Vec<String>,
    pub score: f64,
    #[serde(rename = "type")]
    pub type_: SearchObjectType,
}

#[derive(Serialize, Debug)]
pub struct GenericSearchResult {
    pub id: Uuid,
    pub name: String,
    pub updated_at: DateTime<Utc>,
    pub highlights: Vec<String>,
    pub score: f64,
    #[serde(rename = "type")]
    pub type_: SearchObjectType,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum SearchObject {
    Message(MessageSearchResult),
    Collection(GenericSearchResult),
    Dashboard(GenericSearchResult),
    DataSource(GenericSearchResult),
    Dataset(GenericSearchResult),
    PermissionGroup(GenericSearchResult),
    Team(GenericSearchResult),
    Term(GenericSearchResult),
}

impl SearchObject {
    pub fn updated_at(&self) -> DateTime<Utc> {
        match self {
            SearchObject::Message(m) => m.updated_at,
            SearchObject::Collection(c) => c.updated_at,
            SearchObject::Dashboard(d) => d.updated_at,
            SearchObject::DataSource(ds) => ds.updated_at,
            SearchObject::Dataset(d) => d.updated_at,
            SearchObject::PermissionGroup(pg) => pg.updated_at,
            SearchObject::Team(t) => t.updated_at,
            SearchObject::Term(t) => t.updated_at,
        }
    }

    pub fn score(&self) -> f64 {
        match self {
            SearchObject::Message(m) => m.score,
            SearchObject::Collection(c) => c.score,
            SearchObject::Dashboard(d) => d.score,
            SearchObject::DataSource(ds) => ds.score,
            SearchObject::Dataset(d) => d.score,
            SearchObject::PermissionGroup(pg) => pg.score,
            SearchObject::Team(t) => t.score,
            SearchObject::Term(t) => t.score,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SearchObjectType {
    Thread,
    Collection,
    Dashboard,
    DataSource,
    Dataset,
    PermissionGroup,
    Team,
    Term,
}

impl ToString for SearchObjectType {
    fn to_string(&self) -> String {
        match self {
            SearchObjectType::Thread => "thread".to_string(),
            SearchObjectType::Collection => "collection".to_string(),
            SearchObjectType::Dashboard => "dashboard".to_string(),
            SearchObjectType::DataSource => "data_source".to_string(),
            SearchObjectType::Dataset => "dataset".to_string(),
            SearchObjectType::PermissionGroup => "permission_group".to_string(),
            SearchObjectType::Team => "team".to_string(),
            SearchObjectType::Term => "term".to_string(),
        }
    }
}

pub struct SearchOptions {
    pub num_results: i64,
    pub asset_types: Vec<SearchObjectType>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            num_results: 10,
            asset_types: vec![],
        }
    }
}

impl SearchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_custom_options(num_results: i64, asset_types: Vec<SearchObjectType>) -> Self {
        SearchOptions {
            num_results,
            asset_types,
        }
    }

    pub fn asset_types_to_string(&self) -> String {
        self.asset_types
            .iter()
            .map(|t| format!("'{}'", t.to_string()))
            .collect::<Vec<_>>()
            .join(",")
    }
}

// TODO: Will need to implement search for shared assets via team.  Likely will just fetch user teams or get them from a cache.
pub async fn search_engine(
    user_id: Uuid,
    organization_id: Uuid,
    query_text: String,
    options: SearchOptions,
) -> Result<Vec<SearchObject>> {
    let mut conn = get_sqlx_pool().acquire().await?;

    let search_terms: Vec<String> = query_text
        .split_whitespace()
        .map(|term| sanitize_search_term(term.to_lowercase()))
        .collect();

    let query = format!(
        r#"
        SELECT DISTINCT ON (asset_search.content, asset_search.asset_type)
            asset_search.asset_id,
            asset_search.content,
            asset_search.updated_at,
            asset_search.asset_type,
            pgroonga_score(asset_search.tableoid, asset_search.ctid) AS rank
        FROM 
            asset_search
        INNER JOIN
            asset_permissions
        ON 
            asset_search.asset_id = asset_permissions.asset_id
        WHERE 
            asset_search.asset_type IN ({})
            AND asset_search.content &@~ '{}'
            AND (asset_permissions.identity_id = '{}' OR asset_permissions.identity_id = '{}')
            AND asset_permissions.deleted_at IS NULL
            AND asset_search.deleted_at IS NULL
        ORDER BY asset_search.content, asset_search.asset_type, rank DESC
        LIMIT {};
        "#,
        options.asset_types_to_string(),
        search_terms
            .iter()
            .map(|term| term.replace('\'', "''"))
            .collect::<Vec<_>>()
            .join(" OR "),
        user_id,
        organization_id,
        options.num_results
    );

    let mut results = sqlx::raw_sql(&query).fetch(&mut *conn);
    let mut results_vec = Vec::new();
    while let Some(row) = results.try_next().await? {
        let content: String = match row.try_get("content") {
            Ok(content) => content,
            Err(e) => return Err(anyhow!("Error getting content: {:?}", e)),
        };

        // Skip empty content
        if content.trim().is_empty() {
            continue;
        }

        let id: Uuid = match row.try_get("asset_id") {
            Ok(id) => id,
            Err(e) => return Err(anyhow!("Error getting asset_id: {:?}", e)),
        };
        let updated_at: DateTime<Utc> = match row.try_get("updated_at") {
            Ok(updated_at) => updated_at,
            Err(e) => return Err(anyhow!("Error getting updated_at: {:?}", e)),
        };
        let score: f64 = match row.try_get("rank") {
            Ok(score) => score,
            Err(e) => return Err(anyhow!("Error getting rank: {:?}", e)),
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

        let highlights = find_highlights(&content, &search_terms);

        results_vec.push(SearchObject::Message(MessageSearchResult {
            id,
            title: content.clone(),
            updated_at,
            summary_question: content,
            highlights,
            score,
            type_: asset_type,
        }));
    }
    let results = results_vec;

    Ok(results)
}

fn find_highlights(content: &str, search_terms: &[String]) -> Vec<String> {
    let content_lower = content.to_lowercase();
    let mut highlights = Vec::new();

    for term in search_terms {
        if let Some(pos) = content_lower.find(term) {
            // Just grab the exact matching portion from the original content
            highlights.push(content[pos..pos + term.len()].to_string());
        }
    }

    highlights
}

fn sanitize_search_term(term: String) -> String {
    // First pass: only allow alphanumeric, spaces, and basic punctuation
    let filtered = term
        .chars()
        .filter(|c| {
            c.is_alphanumeric()
                || c.is_whitespace()
                || matches!(c, '-' | '_' | '.' | ',' | '@' | '#')
        })
        .collect::<String>();

    // Second pass: escape special PostgreSQL operators and wildcards
    let escaped = filtered
        .replace('\\', "\\\\") // Escape backslashes first
        .replace('%', "\\%") // Escape LIKE wildcards
        .replace('_', "\\_")
        .replace('*', "\\*") // Escape full-text search wildcards
        .replace(':', "\\:")
        .replace('&', "\\&")
        .replace('|', "\\|")
        .replace('!', "\\!")
        .replace('(', "\\(") // Escape parentheses
        .replace(')', "\\)");

    // Third pass: prevent SQL comments
    let no_comments = escaped
        .replace("--", "")
        .replace("/*", "")
        .replace("*/", "");

    // Fourth pass: limit length to prevent buffer overflow attacks
    no_comments.chars().take(100).collect()
}
