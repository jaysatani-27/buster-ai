use anyhow::{anyhow, Result};
use diesel::{update, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use uuid::Uuid;

use crate::{
    database::{
        lib::get_pg_pool,
        models::User,
        schema::threads,
    },
    routes::ws::{
        ws::{SubscriptionRwLock, WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::sentry_utils::send_sentry_error,
};

use super::threads_router::{ThreadEvent, ThreadRoute};

#[derive(Deserialize, Debug, Clone)]
pub struct DeleteThreadRequest {
    pub ids: Vec<Uuid>,
}

#[derive(Serialize, Debug, Clone)]
pub struct DeleteThreadRes {
    pub id: Uuid,
}

pub async fn delete_thread(
    subscriptions: &Arc<SubscriptionRwLock>,
    user: &User,
    req: DeleteThreadRequest,
) -> Result<()> {
    let mut delete_tasks = Vec::new();

    for id in req.ids {
        let subscription = format!("thread:{}", id);

        let delete_thread_res = match delete_thread_handler(user, &id).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Error getting thread: {}", e);
                send_sentry_error(&e.to_string(), None);
                send_error_message(
                    &user.id.to_string(),
                    WsRoutes::Threads(ThreadRoute::Delete),
                    WsEvent::Threads(ThreadEvent::DeleteThreadState),
                    WsErrorCode::InternalServerError,
                    "Failed to delete thread.".to_string(),
                    user,
                )
                .await?;
                return Err(anyhow!("Error getting thread: {}", e));
            }
        };

        delete_tasks.push(delete_thread_res);

        let delete_thread_res_message = WsResponseMessage::new(
            WsRoutes::Threads(ThreadRoute::Delete),
            WsEvent::Threads(ThreadEvent::DeleteThreadState),
            vec![id],
            None,
            user,
            WsSendMethod::All,
        );

        match send_ws_message(&subscription, &delete_thread_res_message).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error sending message to pubsub: {}", e);
                let err = anyhow!("Error sending message to pubsub: {}", e);
                send_sentry_error(&err.to_string(), Some(&user.id));
                return Err(err);
            }
        }

        subscriptions.remove(subscription.clone()).await;
    }

    let mut delete_responses = Vec::new();

    for task in delete_tasks {
        match task.await {
            Ok(Ok(res)) => delete_responses.push(res.id),
            Ok(Err(e)) => {
                tracing::error!("Unable to insert thread into database: {:?}", e);
                let err = anyhow!("Unable to insert thread into database: {}", e);
                send_sentry_error(&err.to_string(), Some(&user.id));
                return Err(err);
            }
            Err(e) => {
                tracing::error!("Unable to insert thread into database: {:?}", e);
                let err = anyhow!("Unable to insert thread into database: {}", e);
                send_sentry_error(&err.to_string(), Some(&user.id));
                return Err(err);
            }
        }
    }

    let delete_thread_res_message = WsResponseMessage::new(
        WsRoutes::Threads(ThreadRoute::Delete),
        WsEvent::Threads(ThreadEvent::DeleteThreadState),
        delete_responses,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &delete_thread_res_message).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error sending message to pubsub: {}", e);
            let err = anyhow!("Error sending message to pubsub: {}", e);
            send_sentry_error(&err.to_string(), Some(&user.id));
            return Err(err);
        }
    }

    Ok(())
}

async fn delete_thread_handler(
    user: &User,
    id: &Uuid,
) -> Result<tokio::task::JoinHandle<Result<DeleteThreadRes>>> {
    let mut conn = get_pg_pool().get().await.map_err(|e| {
        tracing::error!("Error getting connection from pool: {}", e);
        let err = anyhow!("Error getting connection: {}", e);
        send_sentry_error(&err.to_string(), Some(&user.id));
        err
    })?;

    let user_id = user.id.clone();
    let id = id.clone();

    let delete_thread_task = tokio::task::spawn(async move {
        match update(threads::table)
            .filter(threads::id.eq(&id))
            .set(threads::deleted_at.eq(chrono::Utc::now()))
            .execute(&mut conn)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Unable to insert thread: {:?}", e);
                let err = anyhow!("Unable to insert thread: {}", e);
                send_sentry_error(&err.to_string(), None);
                return Err(err);
            }
        }

        Ok(DeleteThreadRes { id })
    });

    Ok(delete_thread_task)
}
