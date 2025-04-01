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

use super::threads_router::{ThreadEvent, ThreadRoute};

#[derive(Deserialize, Debug, Clone)]
pub struct UnsubscribeFromThreadRequest {
    pub id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsubscribeFromThreadResponse {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeftThreadResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub thread_id: Uuid,
}

pub async fn unsubscribe(
    subscriptions: &Arc<SubscriptionRwLock>,
    user: &User,
    user_group: &String,
    req: UnsubscribeFromThreadRequest,
) -> Result<()> {
    if let Some(id) = &req.id {
        let subscription = format!("thread:{}", id);

        unsubscribe_from_stream(&subscriptions, &subscription, user_group, &user.id).await?;

        let left_thread_res = LeftThreadResponse {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            thread_id: id.clone(),
        };

        let left_subscription_ws_message = WsResponseMessage::new(
            WsRoutes::Threads(ThreadRoute::Get),
            WsEvent::Threads(ThreadEvent::LeftThread),
            vec![left_thread_res],
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

        let unsubscribe_res = UnsubscribeFromThreadResponse { id: id.clone() };

        let unsubscribe_ws_message = WsResponseMessage::new(
            WsRoutes::Threads(ThreadRoute::Unsubscribe),
            WsEvent::Threads(ThreadEvent::Unsubscribed),
            vec![unsubscribe_res],
            None,
            user,
            WsSendMethod::All,
        );

        match send_ws_message(&user.id.to_string(), &unsubscribe_ws_message).await {
            Ok(_) => (),
            Err(e) => {
                let err = anyhow!("Error sending ws message: {}", e);
                send_sentry_error(&err.to_string(), Some(&user.id));
                return Err(err);
            }
        }
    }

    Ok(())
}
