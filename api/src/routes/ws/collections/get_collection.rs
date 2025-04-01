use anyhow::{anyhow, Result};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::models::User,
    routes::ws::{
        ws::{SubscriptionRwLock, WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message, subscribe_to_stream},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::{
    collection_utils::get_collection_by_id,
    collections_router::{CollectionEvent, CollectionRoute},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCollectionRequest {
    pub id: Uuid,
}

pub async fn get_collection(
    subscriptions: &Arc<SubscriptionRwLock>,
    user_group: &String,
    user: &User,
    req: GetCollectionRequest,
) -> Result<()> {
    let collection_subscription = format!("collection:{}", &req.id);

    match subscribe_to_stream(
        subscriptions,
        &collection_subscription,
        user_group,
        &user.id,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error subscribing to stream: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            return Err(e);
        }
    };

    let collection = match get_collection_by_id(&user.id, &req.id).await {
        Ok(collection) => collection,
        Err(e) => {
            tracing::error!("Error getting collection: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Collections(CollectionRoute::Get),
                WsEvent::Collections(CollectionEvent::CollectionState),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let post_collection_message = WsResponseMessage::new(
        WsRoutes::Collections(CollectionRoute::Get),
        WsEvent::Collections(CollectionEvent::CollectionState),
        collection,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &post_collection_message).await {
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
