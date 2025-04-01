use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

use crate::{
    database::{
        lib::get_pg_pool,
        models::{Term, User},
        schema::{terms, terms_to_datasets, users},
    },
    routes::ws::{
        ws::{WsErrorCode, WsEvent, WsResponseMessage, WsSendMethod},
        ws_router::WsRoutes,
        ws_utils::{send_error_message, send_ws_message},
    },
    utils::{clients::sentry_utils::send_sentry_error, user::user_info::get_user_organization_id},
};

use super::{
    terms_router::{TermEvent, TermRoute},
    terms_utils::TermCreator,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListTermsRequest {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TermListItem {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_edited: DateTime<Utc>,
    pub dataset_count: i64,
    pub created_by: TermCreator,
}

pub async fn list_terms(user: &User, req: ListTermsRequest) -> Result<()> {
    let list_terms_res = match list_terms_handler(&user.id, req).await {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Error getting terms: {}", e);
            let err = anyhow!("Error getting terms: {}", e);
            send_sentry_error(&e.to_string(), Some(&user.id));
            send_error_message(
                &user.id.to_string(),
                WsRoutes::Terms(TermRoute::List),
                WsEvent::Terms(TermEvent::ListTerms),
                WsErrorCode::InternalServerError,
                "Failed to list terms.".to_string(),
                user,
            )
            .await?;
            return Err(err);
        }
    };

    let list_terms_message = WsResponseMessage::new(
        WsRoutes::Terms(TermRoute::List),
        WsEvent::Terms(TermEvent::ListTerms),
        list_terms_res,
        None,
        user,
        WsSendMethod::SenderOnly,
    );

    match send_ws_message(&user.id.to_string(), &list_terms_message).await {
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

async fn list_terms_handler(user_id: &Uuid, req: ListTermsRequest) -> Result<Vec<TermListItem>> {
    let page = req.page.unwrap_or(0);
    let page_size = req.page_size.unwrap_or(25);

    let organization_id = get_user_organization_id(user_id).await?;

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            return Err(anyhow!("Error getting connection: {}", e));
        }
    };

    let terms_with_info_but_no_datasets: Vec<(Term, Option<String>, String, Uuid)> =
        match terms::table
            .inner_join(users::table.on(terms::created_by.eq(users::id)))
            .filter(terms::organization_id.eq(organization_id))
            .filter(terms::deleted_at.is_null())
            .select((
                (
                    terms::id,
                    terms::name,
                    terms::definition,
                    terms::sql_snippet,
                    terms::organization_id,
                    terms::created_by,
                    terms::updated_by,
                    terms::created_at,
                    terms::updated_at,
                    terms::deleted_at,
                ),
                users::name.nullable(),
                users::email,
                users::id,
            ))
            .order(terms::updated_at.desc())
            .limit(page_size)
            .offset(page * page_size)
            .load::<(Term, Option<String>, String, Uuid)>(&mut conn)
            .await
        {
            Ok(terms) => terms,
            Err(diesel::NotFound) => vec![],
            Err(e) => {
                return Err(anyhow!("Error loading terms: {}", e));
            }
        };

    let term_ids: Vec<Uuid> = terms_with_info_but_no_datasets
        .iter()
        .map(|(term, _, _, _)| term.id)
        .collect();

    let dataset_records = match terms_to_datasets::table
        .select((terms_to_datasets::term_id, terms_to_datasets::dataset_id))
        .filter(terms_to_datasets::term_id.eq_any(&term_ids))
        .filter(terms_to_datasets::deleted_at.is_null())
        .load::<(Uuid, Uuid)>(&mut conn)
        .await
    {
        Ok(counts) => counts,
        Err(e) => return Err(anyhow!("Error loading dataset counts: {}", e)),
    };

    let dataset_counts: std::collections::HashMap<Uuid, usize> = dataset_records.into_iter().fold(
        std::collections::HashMap::new(),
        |mut acc, (term_id, _)| {
            *acc.entry(term_id).or_insert(0) += 1;
            acc
        },
    );

    let term_list_items: Vec<TermListItem> = terms_with_info_but_no_datasets
        .into_iter()
        .map(|(term, user_name, user_email, user_id)| TermListItem {
            id: term.id,
            name: term.name,
            last_edited: term.updated_at,
            created_at: term.created_at,
            created_by: TermCreator {
                id: user_id,
                name: user_name.unwrap_or(user_email),
            },
            dataset_count: dataset_counts.get(&term.id).cloned().unwrap_or(0) as i64,
        })
        .collect();

    Ok(term_list_items)
}
