use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    database::models::User,
    routes::ws::{threads_and_messages::unsubscribe::unsubscribe, ws::SubscriptionRwLock},
};

use super::{
    delete_thread::delete_thread, duplicate_thread::duplicate_thread,
    get_message_data::get_message_data, get_thread::get_thread, list_threads::list_threads,
    post_thread::post_thread::post_thread, update_message::update_message,
    update_thread::update_thread,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ThreadRoute {
    #[serde(rename = "/threads/list")]
    List,
    #[serde(rename = "/threads/get")]
    Get,
    #[serde(rename = "/threads/post")]
    Post,
    #[serde(rename = "/threads/unsubscribe")]
    Unsubscribe,
    #[serde(rename = "/threads/update")]
    Update,
    #[serde(rename = "/threads/delete")]
    Delete,
    #[serde(rename = "/threads/messages/update")]
    UpdateMessage,
    #[serde(rename = "/threads/messages/data")]
    MessageData,
    #[serde(rename = "/threads/duplicate")]
    DuplicateThread,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ThreadEvent {
    Thought,
    InitializeThread,
    ModifyVisualization,
    InitializeDraftSession,
    CheckpointThread,
    CompletedThread,
    JoinedThread,
    LeftThread,
    GeneratingResponse,
    GeneratingMetricTitle,
    #[serde(rename = "GeneratingDescription")]
    GeneratingSummaryQuestion,
    GeneratingTimeFrame,
    FixingSql,
    GetThreadState,
    UpdateThreadState,
    GetThreadsList,
    UpdateThreadsList,
    PostThread,
    IdentifyingDataset,
    IdentifyingTerms,
    GeneratingSql,
    FetchingData,
    DeleteThreadState,
    BulkDeleteThreads,
    SearchThreads,
    Unsubscribed,
    DuplicateThread,
    SqlEvaluation,
}

pub async fn threads_router(
    route: ThreadRoute,
    data: Value,
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
) -> Result<()> {
    match route {
        ThreadRoute::List => {
            let req = serde_json::from_value(data)?;

            list_threads(user, req).await?;
        }
        ThreadRoute::Get => {
            let req = serde_json::from_value(data)?;

            get_thread(subscriptions, user_group, user, req).await?;
        }
        ThreadRoute::Unsubscribe => {
            let req = serde_json::from_value(data)?;

            unsubscribe(&subscriptions, user, user_group, req).await?;
        }
        ThreadRoute::Post => {
            let req = serde_json::from_value(data)?;

            post_thread(subscriptions, user_group, user, req).await?;
        }
        ThreadRoute::Update => {
            let req = serde_json::from_value(data)?;

            update_thread(subscriptions, user_group, user, req).await?;
        }
        ThreadRoute::Delete => {
            let req = serde_json::from_value(data)?;

            delete_thread(subscriptions, user, req).await?;
        }
        ThreadRoute::UpdateMessage => {
            let req = serde_json::from_value(data)?;

            update_message(subscriptions, user_group, user, req).await?;
        }
        ThreadRoute::MessageData => {
            let req = serde_json::from_value(data)?;

            get_message_data(user, req).await?;
        }
        ThreadRoute::DuplicateThread => {
            let req = serde_json::from_value(data)?;

            duplicate_thread(subscriptions, user_group, user, req).await?;
        }
    };

    Ok(())
}

impl ThreadRoute {
    pub fn from_str(path: &str) -> Result<Self> {
        match path {
            "/threads/list" => Ok(Self::List),
            "/threads/get" => Ok(Self::Get),
            "/threads/post" => Ok(Self::Post),
            "/threads/unsubscribe" => Ok(Self::Unsubscribe),
            "/threads/update" => Ok(Self::Update),
            "/threads/delete" => Ok(Self::Delete),
            "/threads/duplicate" => Ok(Self::DuplicateThread),
            "/threads/messages/update" => Ok(Self::UpdateMessage),
            "/threads/messages/data" => Ok(Self::MessageData),
            _ => Err(anyhow!("Invalid path")),
        }
    }
}
