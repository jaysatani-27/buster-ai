use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use uuid::Uuid;

use crate::{
    database::models::User,
    routes::ws::{
        ws::{SubscriptionRwLock, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_ws_message, unsubscribe_from_stream},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::collections_router::{CollectionEvent, CollectionRoute};

#[derive(Deserialize, Debug, Clone)]
pub struct UnsubscribeFromCollectionRequest {
    pub id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsubscribeFromCollectionResponse {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeftCollectionResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub dashboard_id: Uuid,
}

pub async fn unsubscribe(
    subscriptions: &Arc<SubscriptionRwLock>,
    user: &User,
    user_group: &String,
    req: UnsubscribeFromCollectionRequest,
) -> Result<()> {
    if let Some(id) = &req.id {
        let subscription = format!("collection:{}", id);

        match unsubscribe_from_stream(subscriptions, &subscription, user_group, &user.id).await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("Error unsubscribing from stream: {}", e));
            }
        }

        let left_collection_res = LeftCollectionResponse {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            dashboard_id: id.clone(),
        };

        let left_subscription_ws_message = WsResponseMessage::new(
            WsRoutes::Collections(CollectionRoute::Unsubscribe),
            WsEvent::Collections(CollectionEvent::Unsubscribed),
            vec![left_collection_res],
            None,
            user,
            WsSendMethod::All,
        );

        match send_ws_message(&subscription, &left_subscription_ws_message).await {
            Ok(_) => (),
            Err(e) => {
                let err = anyhow!("Error sending ws message: {}", e);
                send_sentry_error(&err.to_string(), Some(&user.id));
                return Err(err);
            }
        }

        let unsubscribe_res = UnsubscribeFromCollectionResponse { id: id.clone() };

        let unsubscribe_ws_message = WsResponseMessage::new(
            WsRoutes::Collections(CollectionRoute::Unsubscribe),
            WsEvent::Collections(CollectionEvent::Unsubscribed),
            vec![unsubscribe_res],
            None,
            user,
            WsSendMethod::SenderOnly,
        );

        match send_ws_message(&user.id.to_string(), &unsubscribe_ws_message).await {
            Ok(_) => (),
            Err(e) => {
                let err = anyhow!("Error sending ws message: {}", e);
                send_sentry_error(&err.to_string(), Some(&user.id));
                return Err(err);
            }
        }
    };

    Ok(())
}
