use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use tokio::fs;

use crate::utils::{BusterClient, PostDataSourcesRequest};

use super::{buster_credentials::BusterCredentials, project_files::get_current_project};

#[derive(Debug, Serialize, Deserialize)]
pub struct DbtProfiles {
    #[serde(flatten)]
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub target: String,
    pub outputs: HashMap<String, Output>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub threads: Option<u32>,
    // TODO: Make this a struct for each of the different db types
    #[serde(flatten)]
    pub credential: Credential,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Credential {
    Postgres(PostgresCredentials),
    MySQL(MySqlCredentials),
    Bigquery(BigqueryCredentials),
    SqlServer(SqlServerCredentials),
    Redshift(PostgresCredentials),
    Databricks(DatabricksCredentials),
    Snowflake(SnowflakeCredentials),
    Starrocks(MySqlCredentials),
}

impl Credential {
    pub fn get_schema(&self) -> String {
        match self {
            Credential::Postgres(cred) => cred.schema.clone(),
            _ => "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AthenaCredentials {
    pub data_source: String,
    pub db_database: String,
    pub aws_access_key: String,
    pub aws_secret_access: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BigqueryCredentials {
    pub credentials_json: Value,
    pub project_id: String,
    pub dataset_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabricksCredentials {
    pub host: String,
    pub api_key: String,
    pub warehouse_id: String,
    pub catalog_name: String,
    pub schemas: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MariadbCredentials {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub jump_host: Option<String>,
    pub ssh_username: Option<String>,
    pub ssh_private_key: Option<String>,
    #[serde(rename = "schemas")]
    pub databases: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MySqlCredentials {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub jump_host: Option<String>,
    pub ssh_username: Option<String>,
    pub ssh_private_key: Option<String>,
    #[serde(rename = "schemas")]
    pub databases: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostgresCredentials {
    pub host: String,
    pub port: u16,
    #[serde(alias = "user")]
    pub username: String,
    #[serde(alias = "pass")]
    pub password: String,
    #[serde(rename = "dbname")]
    pub database: String,
    pub schema: String,
    pub jump_host: Option<String>,
    pub ssh_username: Option<String>,
    pub ssh_private_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnowflakeCredentials {
    pub account_id: String,
    pub warehouse_id: String,
    pub database_id: String,
    pub username: String,
    pub password: String,
    pub role: Option<String>,
    pub schemas: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SqlServerCredentials {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub jump_host: Option<String>,
    pub ssh_username: Option<String>,
    pub ssh_private_key: Option<String>,
    pub schemas: Option<Vec<String>>,
}

pub async fn get_dbt_profiles_yml() -> Result<DbtProfiles> {
    let mut path = home_dir().unwrap_or_default();
    path.push(".dbt");
    path.push("profiles.yml");

    if !fs::try_exists(&path).await? {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    let contents = fs::read_to_string(path).await?;
    Ok(serde_yaml::from_str(&contents)?)
}

// Returns a list of tuples containing the profile name, environment, and credentials.
pub async fn get_dbt_profile_credentials(
    profile_names: &Vec<String>,
) -> Result<Vec<(String, String, Credential)>> {
    let profiles = get_dbt_profiles_yml().await?;

    let mut credentials = Vec::new();
    for name in profile_names {
        if let Some(profile) = profiles.profiles.get(name) {
            for (env, output) in &profile.outputs {
                credentials.push((name.clone(), env.clone(), output.credential.clone()));
            }
        }
    }

    Ok(credentials)
}

pub async fn upload_dbt_profiles_to_buster(
    credentials: Vec<(String, String, Credential)>,
    buster_creds: BusterCredentials,
) -> Result<()> {
    let buster = BusterClient::new(buster_creds.url, buster_creds.api_key)?;

    let mut req_body = Vec::new();
    for (name, env, cred) in credentials {
        req_body.push(PostDataSourcesRequest {
            name,
            env,
            credential: cred,
        });
    }

    if let Err(e) = buster.post_data_sources(req_body).await {
        return Err(anyhow::anyhow!(
            "Failed to upload dbt profiles to Buster: {}",
            e
        ));
    };

    Ok(())
}

pub async fn get_project_profile() -> Result<(String, Profile)> {
    let project_config = get_current_project().await?;

    let dbt_profiles = get_dbt_profiles_yml().await?;

    let profile = dbt_profiles
        .profiles
        .get(&project_config.profile)
        .ok_or(anyhow::anyhow!(
            "Profile not found: {}",
            project_config.profile
        ))?;

    Ok((project_config.profile, profile.clone()))
}
