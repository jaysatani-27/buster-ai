use anyhow::{anyhow, Result};
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        lib::{get_pg_pool, get_sqlx_pool},
        models::{Term, TermToDataset, User},
        schema::{terms, terms_to_datasets},
    },
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{
        clients::{ai::embedding_router::embedding_router, sentry_utils::send_sentry_error},
        user::user_info::get_user_organization_id,
    },
};

use super::{
    terms_router::{TermEvent, TermRoute},
    terms_utils::{get_term_state, TermState},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostTermRequest {
    pub name: String,
    pub definition: String,
    pub sql_snippet: Option<String>,
    pub dataset_ids: Option<Vec<Uuid>>,
}

pub async fn post_term(user: &User, req: PostTermRequest) -> Result<()> {
    let term = match post_term_handler(&user.id, req).await {
        Ok(term) => term,
        Err(e) => {
            tracing::error!("Error posting term: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Terms(TermRoute::Post),
                WsEvent::Terms(TermEvent::PostTerm),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let post_term_message = WsResponseMessage::new(
        WsRoutes::Terms(TermRoute::Post),
        WsEvent::Terms(TermEvent::PostTerm),
        term,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &post_term_message).await {
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

async fn post_term_handler(user_id: &Uuid, req: PostTermRequest) -> Result<TermState> {
    let organization_id = get_user_organization_id(user_id).await?;

    let mut conn = get_pg_pool().get().await?;

    let term = Term {
        id: Uuid::new_v4(),
        name: req.name,
        definition: Some(req.definition),
        sql_snippet: req.sql_snippet,
        created_by: user_id.clone(),
        updated_by: user_id.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        organization_id,
    };

    let search_term = term.clone();
    let term_id = term.id.clone();

    let insert_term = tokio::spawn(async move {
        match diesel::insert_into(terms::table)
            .values(&term)
            .execute(&mut conn)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error inserting term: {}", e);
                return Err(e);
            }
        }

        if let Some(dataset_ids) = req.dataset_ids {
            for dataset_id in dataset_ids {
                let term_to_dataset = TermToDataset {
                    term_id: term.id,
                    dataset_id,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    deleted_at: None,
                };

                match diesel::insert_into(terms_to_datasets::table)
                    .values(&term_to_dataset)
                    .on_conflict((terms_to_datasets::term_id, terms_to_datasets::dataset_id))
                    .do_update()
                    .set((
                        terms_to_datasets::updated_at.eq(Utc::now()),
                        terms_to_datasets::deleted_at.eq::<Option<chrono::DateTime<Utc>>>(None),
                    ))
                    .execute(&mut conn)
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        tracing::error!("Error inserting term to dataset: {}", e);
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    });

let term_search_insert = tokio::spawn(async move {
        let term_embedding = match embedding_router(vec![search_term.name.clone()], true).await {
            Ok(embedding) => embedding,
            Err(e) => {
                tracing::error!("Error embedding term: {}", e);
                return Err(e);
            }
        };

        let mut conn = match get_pg_pool().get().await {
            Ok(pool) => pool,
            Err(e) => {
                tracing::error!("Error acquiring search pool: {}", e);
                return Err(anyhow!("Error acquiring search pool: {}", e));
            }
        };

        let embedding = serde_json::to_string(&term_embedding[0]).unwrap();

        let query = diesel::sql_query(
            format!("INSERT INTO terms_search (term_id, content, definition, embedding, organization_id)
            VALUES ($1, $2, $3, '{embedding}', $4)
            ON CONFLICT (term_id) 
            DO UPDATE SET
                content = EXCLUDED.content,
                definition = EXCLUDED.definition,
                embedding = EXCLUDED.embedding,
                updated_at = NOW()"
        ))
        .bind::<diesel::sql_types::Uuid, _>(search_term.id)
        .bind::<diesel::sql_types::Text, _>(search_term.name)
        .bind::<diesel::sql_types::Text, _>(search_term.definition.unwrap_or("".to_string()))
        .bind::<diesel::sql_types::Uuid, _>(search_term.organization_id);

        if let Err(e) = query.execute(&mut conn).await {
            tracing::error!("Failed to update asset search: {:?}", e);
            send_sentry_error(&e.to_string(), None);
        }

        Ok(())
    });

    if let Ok(Err(e)) = insert_term.await {
        tracing::error!("Error in term insert: {:?}", e);
        return Err(anyhow!("Error in term insert: {:?}", e));
    };

    if let Ok(Err(e)) = term_search_insert.await {
        tracing::error!("Error in term search insert: {:?}", e);
        return Err(anyhow!("Error in term search insert: {:?}", e));
    };

    let term_state = match get_term_state(user_id, &term_id).await {
        Ok(term_state) => term_state,
        Err(e) => return Err(e),
    };

    Ok(term_state)
}
