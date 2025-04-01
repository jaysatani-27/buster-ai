use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref TYPESENSE_API_KEY: String = env::var("TYPESENSE_API_KEY").unwrap_or("xyz".to_string());
    static ref TYPESENSE_API_HOST: String = env::var("TYPESENSE_API_HOST").unwrap_or("http://localhost:8108".to_string());
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CollectionName {
    Messages,
    Collections,
    Datasets,
    PermissionGroups,
    Teams,
    DataSources,
    Terms,
    Dashboards,
    #[serde(untagged)]
    StoredValues(String),
}

impl CollectionName {
    pub fn to_string(&self) -> String {
        match self {
            CollectionName::Messages => "messages".to_string(),
            CollectionName::Collections => "collections".to_string(),
            CollectionName::Datasets => "datasets".to_string(),
            CollectionName::PermissionGroups => "permission_groups".to_string(),
            CollectionName::Teams => "teams".to_string(),
            CollectionName::DataSources => "data_sources".to_string(),
            CollectionName::Terms => "terms".to_string(),
            CollectionName::Dashboards => "dashboards".to_string(),
            CollectionName::StoredValues(s) => s.clone(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct MessageDocument {
    pub id: Uuid,
    pub name: String,
    pub summary_question: String,
    pub organization_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct GenericDocument {
    pub id: Uuid,
    pub name: String,
    pub organization_id: Uuid,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StoredValueDocument {
    pub id: Uuid,
    pub value: String,
    pub dataset_id: Uuid,
    pub dataset_column_id: Uuid,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Document {
    Message(MessageDocument),
    Dashboard(GenericDocument),
    Dataset(GenericDocument),
    PermissionGroup(GenericDocument),
    Team(GenericDocument),
    DataSource(GenericDocument),
    Term(GenericDocument),
    Collection(GenericDocument),
    StoredValue(StoredValueDocument),
}

impl Document {
    pub fn id(&self) -> Uuid {
        match self {
            Document::Message(m) => m.id,
            Document::Dashboard(d) => d.id,
            Document::Dataset(d) => d.id,
            Document::PermissionGroup(d) => d.id,
            Document::Team(d) => d.id,
            Document::DataSource(d) => d.id,
            Document::Term(d) => d.id,
            Document::Collection(d) => d.id,
            Document::StoredValue(d) => d.id,
        }
    }

    pub fn into_stored_value_document(&self) -> &StoredValueDocument {
        match self {
            Document::StoredValue(d) => d,
            _ => panic!("Document is not a StoredValueDocument"),
        }
    }
}

pub async fn upsert_document(collection: CollectionName, document: Document) -> Result<()> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("X-TYPESENSE-API-KEY", TYPESENSE_API_KEY.parse()?);

    match client
        .request(
            reqwest::Method::POST,
            format!(
                "{}/collections/{}/documents?action=upsert",
                *TYPESENSE_API_HOST,
                collection.to_string()
            ),
        )
        .headers(headers)
        .json(&document)
        .send()
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error sending request: {}", e)),
    };

    Ok(())
}

#[derive(Serialize, Debug)]
pub struct SearchRequest {
    pub searches: Vec<SearchRequestObject>,
}

#[derive(Serialize, Debug)]
pub struct SearchRequestObject {
    pub collection: CollectionName,
    pub q: String,
    pub query_by: String,
    pub prefix: bool,
    pub exclude_fields: String,
    pub highlight_fields: String,
    pub use_cache: bool,
    pub filter_by: String,
    pub vector_query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct RequestParams {
    pub collection_name: CollectionName,
}

#[derive(Deserialize)]
pub struct Highlight {
    pub field: String,
    pub matched_tokens: Vec<String>,
}

#[derive(Deserialize)]
pub struct HybridSearchInfo {
    pub rank_fusion_score: f64,
}

#[derive(Deserialize)]
pub struct Hit {
    pub document: Document,
    pub highlights: Vec<Highlight>,
    pub hybrid_search_info: HybridSearchInfo,
}

#[derive(Deserialize)]
pub struct SearchResult {
    pub request_params: RequestParams,
    pub hits: Vec<Hit>,
}

#[derive(Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

pub async fn search_documents(search_reqs: Vec<SearchRequestObject>) -> Result<SearchResponse> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("X-TYPESENSE-API-KEY", TYPESENSE_API_KEY.parse()?);

    let search_req = SearchRequest {
        searches: search_reqs,
    };

    let res = match client
        .request(
            reqwest::Method::POST,
            format!("{}/multi_search", *TYPESENSE_API_HOST),
        )
        .headers(headers)
        .json(&search_req)
        .send()
        .await
    {
        Ok(res) => {
            let res = if res.status().is_success() {
                res
            } else {
                return Err(anyhow!(
                    "Error sending request: {}",
                    res.text().await.unwrap()
                ));
            };

            res
        }
        Err(e) => return Err(anyhow!("Error sending request: {}", e)),
    };

    let search_response = match res.json::<SearchResponse>().await {
        Ok(search_response) => search_response,
        Err(e) => return Err(anyhow!("Error parsing response: {}", e)),
    };

    Ok(search_response)
}

pub async fn create_collection(schema: Value) -> Result<()> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("X-TYPESENSE-API-KEY", TYPESENSE_API_KEY.parse()?);

    match client
        .request(
            reqwest::Method::POST,
            format!("{}/collections", *TYPESENSE_API_HOST,),
        )
        .headers(headers)
        .json(&schema)
        .send()
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error sending request: {}", e)),
    };

    Ok(())
}

pub async fn delete_collection(collection_name: &String, filter_by: &String) -> Result<()> {
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("X-TYPESENSE-API-KEY", TYPESENSE_API_KEY.parse()?);

    match client
        .request(
            reqwest::Method::DELETE,
            format!(
                "{}/collections/{}/documents?filter_by={}",
                *TYPESENSE_API_HOST, collection_name, filter_by
            ),
        )
        .headers(headers)
        .send()
        .await
    {
        Ok(_) => (),
        Err(e) => return Err(anyhow!("Error sending request: {}", e)),
    };

    Ok(())
}

pub async fn bulk_insert_documents<T>(collection_name: &String, documents: &Vec<T>) -> Result<()>
where
    T: serde::Serialize,
{
    let client = reqwest::Client::builder().build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    headers.insert("X-TYPESENSE-API-KEY", TYPESENSE_API_KEY.parse()?);

    let serialized_documents = documents
        .iter()
        .map(|doc| serde_json::to_string(doc))
        .collect::<Result<Vec<String>, _>>()?
        .join("\n");

    match client
        .request(
            reqwest::Method::POST,
            format!(
                "{}/collections/{}/documents/import?action=create",
                *TYPESENSE_API_HOST, collection_name
            ),
        )
        .headers(headers)
        .body(serialized_documents)
        .send()
        .await
    {
        Ok(res) => {
            if res.status().is_success() {
                tracing::info!("Res: {:?}", res.text().await.unwrap());
                return Ok(());
            }
            tracing::error!("Error sending request: {}", res.text().await.unwrap());
            return Err(anyhow!("Error sending bulk insert request"));
        }
        Err(e) => return Err(anyhow!("Error sending request: {}", e)),
    };
}
