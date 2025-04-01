use anyhow::{anyhow, Result};
use reqwest::header::AUTHORIZATION;
use std::env;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::clients::sentry_utils::send_sentry_error;

lazy_static::lazy_static! {
    static ref POSTHOG_API_KEY: String = env::var("POSTHOG_API_KEY").expect("POSTHOG_API_KEY must be set");
}

const POSTHOG_API_URL: &str = "https://us.i.posthog.com/capture/";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PosthogEvent {
    event: PosthogEventType,
    distinct_id: Option<Uuid>,
    properties: PosthogEventProperties,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PosthogEventType {
    MetricCreated,
    MetricFollowUp,
    MetricAddedToDashboard,
    MetricViewed,
    DashboardViewed,
    TitleManuallyUpdated,
    SqlManuallyUpdated,
    ChartStylingManuallyUpdated,
    ChartStylingAutoUpdated,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PosthogEventProperties {
    MetricCreated(MetricCreatedProperties),
    MetricFollowUp(MetricFollowUpProperties),
    MetricAddedToDashboard(MetricAddedToDashboardProperties),
    MetricViewed(MetricViewedProperties),
    DashboardViewed(DashboardViewedProperties),
    TitleManuallyUpdated(TitleManuallyUpdatedProperties),
    SqlManuallyUpdated(SqlManuallyUpdatedProperties),
    ChartStylingManuallyUpdated(ChartStylingManuallyUpdatedProperties),
    ChartStylingAutoUpdated(ChartStylingAutoUpdatedProperties),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricCreatedProperties {
    pub message_id: Uuid,
    pub thread_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricFollowUpProperties {
    pub message_id: Uuid,
    pub thread_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricAddedToDashboardProperties {
    pub message_id: Uuid,
    pub thread_id: Uuid,
    pub dashboard_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricViewedProperties {
    pub message_id: Uuid,
    pub thread_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DashboardViewedProperties {
    pub dashboard_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TitleManuallyUpdatedProperties {
    pub message_id: Uuid,
    pub thread_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SqlManuallyUpdatedProperties {
    pub message_id: Uuid,
    pub thread_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChartStylingManuallyUpdatedProperties {
    pub message_id: Uuid,
    pub thread_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChartStylingAutoUpdatedProperties {
    pub message_id: Uuid,
    pub thread_id: Uuid,
}

pub async fn send_posthog_event_handler(
    event_type: PosthogEventType,
    user_id: Option<Uuid>,
    properties: PosthogEventProperties,
) -> Result<()> {
    let event = PosthogEvent {
        event: event_type,
        distinct_id: user_id,
        properties,
    };

    tokio::spawn(async move {
        match send_event(event).await {
            Ok(_) => (),
            Err(e) => {
                send_sentry_error(&e.to_string(), None);
                tracing::error!("Unable to send event to Posthog: {:?}", e);
            }
        }
    });

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PosthogRequest {
    #[serde(flatten)]
    event: PosthogEvent,
    api_key: String,
}

async fn send_event(event: PosthogEvent) -> Result<()> {
    let client = reqwest::Client::new();

    let headers = {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", POSTHOG_API_KEY.to_string())
                .parse()
                .unwrap(),
        );
        headers
    };

    let posthog_req = PosthogRequest {
        event,
        api_key: POSTHOG_API_KEY.to_string(),
    };

    match client
        .post(POSTHOG_API_URL)
        .headers(headers)
        .json(&posthog_req)
        .send()
        .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Unable to send request to Posthog: {:?}", e);
            return Err(anyhow!(e));
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_send_event() {
        dotenv().ok();

        let _ = send_event(PosthogEvent {
            event: PosthogEventType::MetricCreated,
            distinct_id: Some(Uuid::parse_str("c2dd64cd-f7f3-4884-bc91-d46ae431901e").unwrap()),
            properties: PosthogEventProperties::MetricCreated(MetricCreatedProperties {
                message_id: Uuid::parse_str("c2dd64cd-f7f3-4884-bc91-d46ae431901e").unwrap(),
                thread_id: Uuid::parse_str("c2dd64cd-f7f3-4884-bc91-d46ae431901e").unwrap(),
            }),
        })
        .await;
    }
}
