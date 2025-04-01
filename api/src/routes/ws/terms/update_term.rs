use anyhow::{anyhow, Result};
use diesel::{AsChangeset, ExpressionMethods};
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        lib::{get_pg_pool, get_sqlx_pool},
        models::{TermToDataset, User},
        schema::{terms, terms_to_datasets},
    },
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::clients::{ai::embedding_router::embedding_router, sentry_utils::send_sentry_error},
};

use super::{
    terms_router::{TermEvent, TermRoute},
    terms_utils::{get_term_state, TermState},
};

#[derive(Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = terms)]
pub struct TermUpdateBody {
    pub name: Option<String>,
    pub definition: Option<String>,
    pub sql_snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTermRequest {
    pub id: Uuid,
    #[serde(flatten)]
    pub term_update_body: Option<TermUpdateBody>,
    pub add_to_dataset: Option<Vec<Uuid>>,
    pub remove_from_dataset: Option<Vec<Uuid>>,
}

pub async fn update_term(user: &User, req: UpdateTermRequest) -> Result<()> {
    let term_state = match update_term_handler(&user.id, req).await {
        Ok(term_state) => term_state,
        Err(e) => {
            tracing::error!("Error updating term: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Terms(TermRoute::Update),
                WsEvent::Terms(TermEvent::UpdateTerm),
                WsErrorCode::InternalServerError,
                e.to_string(),
                user,
            )
            .await?;
            return Err(e);
        }
    };

    let update_term_message = WsResponseMessage::new(
        WsRoutes::Terms(TermRoute::Update),
        WsEvent::Terms(TermEvent::UpdateTerm),
        term_state,
        None,
        user,
        WsSendMethod::All,
    );

    match send_ws_message(&user.id.to_string(), &update_term_message).await {
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

async fn update_term_handler(user_id: &Uuid, req: UpdateTermRequest) -> Result<TermState> {
    let user_id = Arc::new(user_id.clone());
    let term_id = Arc::new(req.id);

    let update_term = {
        let term_id = term_id.clone();
        let update = req.term_update_body.clone();
        tokio::spawn(async move {
            if let Err(e) = update_term_data(term_id, update).await {
                tracing::error!("Error in update_term_data: {}", e);
            }
        })
    };

    let update_datasets = {
        let term_id = term_id.clone();
        let add_to_dataset = req.add_to_dataset.clone();
        let remove_from_dataset = req.remove_from_dataset.clone();
        tokio::spawn(async move {
            if let Err(e) = update_term_datasets(term_id, add_to_dataset, remove_from_dataset).await
            {
                tracing::error!("Error in update_term_datasets: {}", e);
            }
        })
    };

    let (_, _) = tokio::try_join!(update_term, update_datasets)?;

    get_term_state(&user_id, &term_id).await
}

async fn update_term_data(
    term_id: Arc<Uuid>,
    term_changeset: Option<TermUpdateBody>,
) -> Result<()> {
    if let Some(term_changeset) = term_changeset {
        if term_changeset.name.is_none()
            && term_changeset.definition.is_none()
            && term_changeset.sql_snippet.is_none()
        {
            return Ok(());
        }

        let update_term = {
            let term_id = term_id.clone();
            let term_changeset = term_changeset.clone();
            tokio::spawn(async move {
                let mut conn = get_pg_pool().get().await?;
                diesel::update(terms::table)
                    .filter(terms::id.eq(*term_id))
                    .set(&term_changeset)
                    .execute(&mut conn)
                    .await
                    .map_err(|e| anyhow!("Error updating term: {}", e))
            })
        };

        let update_search = {
            if term_changeset.name.is_some() || term_changeset.definition.is_some() {
                let term_id = term_id.clone();
                let name = term_changeset.name.clone();
                let definition = term_changeset.definition.clone();
                Some(tokio::spawn(async move {
                    let term_embedding = match embedding_router(vec![name.clone().unwrap_or("".to_string())], true).await {
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
                        format!("UPDATE terms_search 
                            SET content = $1,
                                definition = $2,
                                embedding = '{embedding}',
                                updated_at = NOW()
                            WHERE term_id = $3"
                        ))
                        .bind::<diesel::sql_types::Text, _>(name.unwrap_or("".to_string()))
                        .bind::<diesel::sql_types::Text, _>(definition.unwrap_or("".to_string()))
                        .bind::<diesel::sql_types::Uuid, _>(*term_id);

                    if let Err(e) = query.execute(&mut conn).await {
                        tracing::error!("Failed to update term search: {:?}", e);
                        send_sentry_error(&e.to_string(), None);
                        return Err(anyhow!("Error updating term search: {}", e));
                    }

                    Ok::<_, anyhow::Error>(())
                }))
            } else {
                None
            }
        };

        if let Some(update_search) = update_search {
            let (term_result, search_result) = tokio::try_join!(update_term, update_search)?;
            term_result?;
            search_result?;
        } else {
            update_term.await??;
        }
    }
    Ok(())
}

async fn update_term_datasets(
    term_id: Arc<Uuid>,
    add_to_dataset: Option<Vec<Uuid>>,
    remove_from_dataset: Option<Vec<Uuid>>,
) -> Result<()> {
    let add_datasets = {
        let term_id = term_id.clone();
        tokio::spawn(async move { add_term_datasets(term_id, add_to_dataset).await })
    };

    let remove_datasets = {
        let term_id = term_id.clone();
        tokio::spawn(async move { remove_term_datasets(term_id, remove_from_dataset).await })
    };

    let (add_result, remove_result) = tokio::try_join!(add_datasets, remove_datasets)?;

    add_result?;
    remove_result?;

    Ok(())
}

async fn add_term_datasets(term_id: Arc<Uuid>, add_to_dataset: Option<Vec<Uuid>>) -> Result<()> {
    if let Some(add_datasets) = add_to_dataset {
        let mut conn = get_pg_pool().get().await?;

        for dataset_id in add_datasets {
            let term_to_dataset = TermToDataset {
                term_id: *term_id,
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
                Err(e) => return Err(anyhow!("Error adding term to dataset: {}", e)),
            }
        }
    }
    Ok(())
}

async fn remove_term_datasets(
    term_id: Arc<Uuid>,
    remove_from_dataset: Option<Vec<Uuid>>,
) -> Result<()> {
    if let Some(remove_datasets) = remove_from_dataset {
        let mut conn = get_pg_pool().get().await?;

        diesel::update(terms_to_datasets::table)
            .filter(terms_to_datasets::term_id.eq(*term_id))
            .filter(terms_to_datasets::dataset_id.eq_any(remove_datasets))
            .set(terms_to_datasets::deleted_at.eq(Some(Utc::now())))
            .execute(&mut conn)
            .await?;
    }
    Ok(())
}
