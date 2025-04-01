use std::sync::Arc;

use anyhow::{anyhow, Result};
use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{
        enums::UserOrganizationRole,
        lib::get_pg_pool,
        models::Term,
        schema::{datasets, terms, terms_to_datasets, users, users_to_organizations},
    },
    utils::user::user_info::get_user_organization_id,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermDataset {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermCreator {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TermState {
    #[serde(flatten)]
    pub term: Term,
    pub permission: UserOrganizationRole,
    pub datasets: Vec<TermDataset>,
    pub created_by: TermCreator,
}

pub async fn get_term_state(user_id: &Uuid, term_id: &Uuid) -> Result<TermState> {
    let user_id = Arc::new(user_id.clone());
    let term_id = Arc::new(term_id.clone());

    let organization_role = {
        let user_id = user_id.clone();
        let term_id = term_id.clone();
        tokio::spawn(async move { get_organization_role(user_id, term_id).await })
    };

    let term_state = {
        let user_id = user_id.clone();
        let term_id = term_id.clone();
        tokio::spawn(async move { get_term_datasets_and_creator(user_id, term_id).await })
    };

    let (organization_role, term_state) = match tokio::try_join!(organization_role, term_state) {
        Ok((role, state)) => (role, state),
        Err(e) => {
            tracing::error!("Error getting organization role or term state: {}", e);
            return Err(anyhow!(
                "Error getting organization role or term state: {}",
                e
            ));
        }
    };

    let permission = match organization_role {
        Ok(role) => role,
        Err(e) => {
            tracing::error!("Error getting organization role: {}", e);
            return Err(anyhow!("Error getting organization role: {}", e));
        }
    };

    let (term, datasets, created_by) = match term_state {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Error getting term state: {}", e);
            return Err(anyhow!("Error getting term state: {}", e));
        }
    };

    Ok(TermState {
        term,
        permission,
        datasets,
        created_by,
    })
}

async fn get_term_datasets_and_creator(
    user_id: Arc<Uuid>,
    term_id: Arc<Uuid>,
) -> Result<(Term, Vec<TermDataset>, TermCreator)> {
    let organization_id = get_user_organization_id(user_id.as_ref()).await?;

    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let term_records = match terms::table
        .inner_join(terms_to_datasets::table.on(terms::id.eq(terms_to_datasets::term_id)))
        .inner_join(datasets::table.on(terms_to_datasets::dataset_id.eq(datasets::id)))
        .inner_join(users::table.on(terms::created_by.eq(users::id)))
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
            datasets::id,
            datasets::name,
            users::id,
            users::name.nullable(),
            users::email,
        ))
        .filter(terms::id.eq(term_id.as_ref()))
        .filter(terms::organization_id.eq(organization_id))
        .filter(terms_to_datasets::deleted_at.is_null())
        .load::<(Term, Uuid, String, Uuid, Option<String>, String)>(&mut conn)
        .await
    {
        Ok(terms) => terms,
        Err(e) => {
            tracing::error!("Error getting term records: {}", e);
            return Err(anyhow!("Error getting term records: {}", e));
        }
    };

    if term_records.is_empty() {
        return Err(anyhow!("Term not found"));
    }

    let term = term_records[0].0.clone();
    let mut datasets = Vec::new();
    let mut creator = None;

    for (_, dataset_id, dataset_name, user_id, user_name, user_email) in term_records {
        datasets.push(TermDataset {
            id: dataset_id,
            name: dataset_name,
        });

        if creator.is_none() {
            creator = Some(TermCreator {
                id: user_id,
                name: user_name.unwrap_or_else(|| user_email),
            });
        }
    }

    let creator = creator.ok_or_else(|| anyhow!("Creator not found"))?;

    let term_state = (term, datasets, creator);

    Ok(term_state)
}

async fn get_organization_role(
    user_id: Arc<Uuid>,
    term_id: Arc<Uuid>,
) -> Result<UserOrganizationRole> {
    let mut conn = match get_pg_pool().get().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Error getting pg connection: {}", e);
            return Err(anyhow!("Error getting pg connection: {}", e));
        }
    };

    let organization_role = match users_to_organizations::table
        .inner_join(
            terms::table.on(users_to_organizations::organization_id.eq(terms::organization_id)),
        )
        .select(users_to_organizations::role)
        .filter(users_to_organizations::user_id.eq(user_id.as_ref()))
        .filter(terms::id.eq(term_id.as_ref()))
        .first::<UserOrganizationRole>(&mut conn)
        .await
    {
        Ok(role) => role,
        Err(diesel::NotFound) => return Err(anyhow!("User not found")),
        Err(e) => {
            tracing::error!("Error getting user organization role: {}", e);
            return Err(anyhow!("Error getting user organization role: {}", e));
        }
    };

    Ok(organization_role)
}
