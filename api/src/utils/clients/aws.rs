use anyhow::{anyhow, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::Client;
use uuid::Uuid;

pub async fn create_db_secret(secret: String) -> Result<String> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let secret_id = Uuid::new_v4().to_string();

    match client
        .create_secret()
        .name(secret_id.clone())
        .secret_string(secret)
        .send()
        .await
    {
        Ok(res) => {
            tracing::info!("Successfully created secret in AWS.");
            tracing::info!("Secret ID: {}", secret_id);
            tracing::info!("Secret ARN: {}", res.arn.unwrap());
        }
        Err(e) => {
            tracing::error!("There was an issue while creating the secret in AWS.");
            return Err(anyhow!(e));
        }
    }

    Ok(secret_id)
}

pub async fn delete_db_secret(secret_id: String) -> Result<()> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let _secret_response = match client.delete_secret().secret_id(secret_id).send().await {
        Ok(secret_response) => secret_response,
        Err(e) => return Err(anyhow!(e)),
    };

    Ok(())
}

pub async fn read_secret(secret_id: String) -> Result<String> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);

    let secret_response = client.get_secret_value().secret_id(secret_id).send().await;

    let secret = match secret_response {
        Ok(secret) => secret,
        Err(e) => return Err(anyhow!(e)),
    };

    let secret_string = match secret.secret_string {
        Some(secret_string) => secret_string,
        None => return Err(anyhow!("There was no secret string in the response")),
    };

    return Ok(secret_string);
}
