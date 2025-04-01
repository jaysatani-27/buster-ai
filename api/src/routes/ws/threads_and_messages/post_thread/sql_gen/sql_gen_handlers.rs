use std::{collections::HashSet, sync::Arc};

use anyhow::{anyhow, Result};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    database::{
        enums::DataSourceType,
        lib::{
            ContextJsonBody, FixingSql, GeneratingSql, IdentifiedDataset, IdentifiedTerm,
            IdentifyingDataset, IdentifyingTerms, PgPool, Step, StepProgress,
        },
        models::{Dataset, DatasetColumn, Term, User},
        schema::{
            data_sources, dataset_columns, datasets, datasets_to_permission_groups,
            permission_groups, permission_groups_to_identities, teams_to_users, terms,
            terms_to_datasets,
        },
    },
    routes::ws::threads_and_messages::{
        post_thread::{
            ai::ai_calls::{
                fix_sql_ai_call, generate_sql_ai_call, select_dataset_ai_call, select_term_ai_call,
            },
            post_thread::PostThreadMessage,
        },
        thread_utils::ThreadState,
        threads_router::ThreadEvent,
    },
    utils::{
        clients::typesense::{self, CollectionName, SearchRequestObject, StoredValueDocument},
        user::user_info::get_user_organization_id,
    },
};

pub async fn generate_sql_handler(
    pg_pool: &PgPool,
    user: &User,
    thread: &ThreadState,
    dataset: &Dataset,
    terms: &HashSet<RelevantTerm>,
    stored_values: &Vec<StoredValueDocument>,
    message_id: &Uuid,
    data_source_type: &DataSourceType,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<Option<String>> {
    let ddl = match get_dataset_ddl(&pg_pool, &dataset, stored_values).await {
        Ok(ddl) => ddl,
        Err(e) => {
            tracing::error!("Unable to get dataset DDL: {:?}", e);
            return Err(e);
        }
    };

    let terms_context_string = match get_terms_context_string(terms).await {
        Ok(terms_context_string) => terms_context_string,
        Err(e) => return Err(e),
    };

    let (mut sql_stream, sql) = match generate_sql_ai_call(
        &ddl,
        &thread,
        &terms_context_string,
        &data_source_type,
        &thread.thread.id,
        &user.id,
    )
    .await
    {
        Ok(sql_stream) => sql_stream,
        Err(e) => return Err(e),
    };

    let mut sql_started = false;
    let mut temp_sql = String::new();

    while let Some(content) = sql_stream.recv().await {
        temp_sql.push_str(&content);

        if temp_sql.contains("[NO SQL]") {
            return Ok(None);
        }

        if !sql_started && content.contains("sql") {
            sql_started = true;
        }

        if sql_started && !content.contains("```") && !content.contains("sql") {
            let sql_chunk = GeneratingSql {
                progress: StepProgress::InProgress,
                sql_chunk: Some(content.to_string()),
                sql: None,
                thread_id: thread.thread.id.clone(),
                message_id: message_id.clone(),
            };

            thread_tx
                .send(PostThreadMessage::new(
                    ThreadEvent::GeneratingSql,
                    Some(sql_chunk),
                ))
                .await?;
        }
    }

    let sql = match sql.await? {
        Ok(sql) => {
            if sql == "[NO SQL]" {
                return Ok(None);
            } else {
                sql
            }
        }
        Err(e) => return Err(e),
    };

    let sql = sql.replace("```sql", "");

    let completed_sql = GeneratingSql {
        progress: StepProgress::Completed,
        sql: Some(sql.clone()),
        sql_chunk: None,
        thread_id: thread.thread.id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::GeneratingSql,
            Some(completed_sql),
        ))
        .await?;

    Ok(Some(sql))
}

pub async fn fix_sql_handler(
    pg_pool: &PgPool,
    user: &User,
    sql: &String,
    errors: &Vec<String>,
    dataset: &Dataset,
    stored_values: &Vec<StoredValueDocument>,
    terms: &HashSet<RelevantTerm>,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: Arc<mpsc::Sender<PostThreadMessage>>,
) -> Result<String> {
    let ddl = match get_dataset_ddl(&pg_pool, dataset, stored_values).await {
        Ok(ddl) => ddl,
        Err(e) => {
            tracing::error!("Unable to get dataset DDL: {:?}", e);
            return Err(e);
        }
    };

    let terms_context_string = match get_terms_context_string(terms).await {
        Ok(terms_context_string) => terms_context_string,
        Err(e) => {
            tracing::error!("Unable to get terms context string: {:?}", e);
            return Err(e);
        }
    };

    let data_source_type = match get_data_source_type(&pg_pool, &dataset.id).await {
        Ok(data_source_type) => data_source_type,
        Err(e) => return Err(anyhow!("Failed to get data source type: {}", e)),
    };

    let errors = errors.join("\n");

    let (mut fix_sql_stream, fixed_sql) = match fix_sql_ai_call(
        &ddl,
        &sql,
        &errors,
        &terms_context_string,
        &data_source_type,
        thread_id,
        &user.id,
    )
    .await
    {
        Ok(sql_stream) => sql_stream,
        Err(e) => {
            tracing::error!("Unable to fix SQL: {:?}", e);
            return Err(anyhow!("Unable to fix SQL: {:?}", e));
        }
    };

    let mut sql_started = false;

    while let Some(content) = fix_sql_stream.recv().await {
        if !sql_started {
            if content.contains("```sql") {
                sql_started = true;
                continue; // Skip the line containing ```sql
            }
        } else {
            if content.contains("```") {
                // End of SQL content
                break;
            }

            let sql_chunk = FixingSql {
                progress: StepProgress::InProgress,
                sql_chunk: Some(content.clone()),
                sql: None,
                thread_id: thread_id.clone(),
                message_id: message_id.clone(),
            };

            thread_tx
                .send(PostThreadMessage::new(
                    ThreadEvent::FixingSql,
                    Some(sql_chunk),
                ))
                .await?;
        }
    }

    let fixed_sql = match fixed_sql.await? {
        Ok(fixed_sql) => {
            let re = regex::Regex::new(r"(?s)```sql(.*?)```")
                .map_err(|e| anyhow!("Regex error: {}", e))?;
            let mut extracted_sql = String::new();
            for cap in re.captures_iter(&fixed_sql) {
                if let Some(sql) = cap.get(1) {
                    extracted_sql.push_str(sql.as_str());
                }
            }
            extracted_sql
        }
        Err(e) => return Err(e),
    };

    let completed_fixed_sql = FixingSql {
        progress: StepProgress::Completed,
        sql: Some(sql.clone()),
        sql_chunk: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::FixingSql,
            Some(completed_fixed_sql),
        ))
        .await?;

    Ok(fixed_sql)
}

pub async fn identifying_terms_handler(
    pg_pool: &PgPool,
    user: &User,
    thread: &ThreadState,
    prompt: &String,
    dataset_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<HashSet<RelevantTerm>> {
    let last_message = match thread.messages.last() {
        Some(last_message) => last_message,
        None => return Err(anyhow!("Thread has no messages")),
    };

    let thread = Arc::new(thread.clone());
    let pg_pool = Arc::new(pg_pool.clone());

    let previously_used_terms_handler = {
        let thread = Arc::clone(&thread);
        let pg_pool = Arc::clone(&pg_pool);
        tokio::spawn(async move { get_previously_used_terms(pg_pool, thread).await })
    };

    let search_for_relevant_terms_handler = {
        let pg_pool = Arc::clone(&pg_pool);
        let last_message = last_message.clone();
        let dataset_id = *dataset_id;
        let user = user.clone();
        tokio::spawn(async move {
            search_for_relevant_terms(
                &pg_pool,
                &user.id,
                &last_message.message.message,
                &dataset_id,
            )
            .await
        })
    };

    let (previously_used_terms_res, search_for_relevant_terms_res) = match tokio::try_join!(
        previously_used_terms_handler,
        search_for_relevant_terms_handler
    ) {
        Ok((previously_used_terms_res, search_for_relevant_terms_res)) => {
            (previously_used_terms_res, search_for_relevant_terms_res)
        }
        Err(e) => {
            return Err(anyhow!(
                "Error joining previously used terms and search for relevant terms: {e}"
            ))
        }
    };

    let previously_used_terms = match previously_used_terms_res {
        Ok(previously_used_terms) => previously_used_terms,
        Err(e) => return Err(e),
    };

    let terms = match search_for_relevant_terms_res {
        Ok(terms) => terms,
        Err(e) => return Err(e),
    };

    let select_term_response =
        match select_term_ai_call(&prompt, &terms, &thread.thread.id, &user.id).await {
            Ok(select_term_response) => select_term_response,
            Err(e) => return Err(e),
        };

    let filtered_terms = terms
        .iter()
        .filter(|t| select_term_response.terms.contains(&t.name))
        .cloned()
        .collect::<Vec<RelevantTerm>>();

    let identified_terms = filtered_terms
        .iter()
        .map(|t| IdentifiedTerm {
            name: t.name.clone(),
            id: t.id.clone(),
        })
        .collect::<Vec<IdentifiedTerm>>();

    let merged_terms = filtered_terms
        .into_iter()
        .chain(previously_used_terms.into_iter().map(|term| {
            RelevantTerm {
                id: term.id,
                name: term.name,
                definition: term
                    .definition
                    .unwrap_or(String::from("No definition found")),
                sql_snippet: term.sql_snippet,
            }
        }))
        .collect::<HashSet<RelevantTerm>>();

    let identify_terms_body = IdentifyingTerms {
        progress: StepProgress::Completed,
        terms: Some(identified_terms.clone()),
        thread_id: thread.thread.id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::IdentifyingTerms,
            Some(identify_terms_body),
        ))
        .await?;

    Ok(merged_terms)
}

pub async fn get_data_source_type(pool: &PgPool, dataset_id: &Uuid) -> Result<DataSourceType> {
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Unable to get connection from pool: {}", e)),
    };

    let data_source_type: DataSourceType = match data_sources::table
        .select(data_sources::type_)
        .inner_join(datasets::table.on(datasets::data_source_id.eq(data_sources::id)))
        .filter(datasets::id.eq(&dataset_id))
        .filter(data_sources::deleted_at.is_null())
        .first::<DataSourceType>(&mut conn)
        .await
    {
        Ok(sources) => sources,
        Err(e) => return Err(anyhow!("Unable to load data sources: {}", e)),
    };

    Ok(data_source_type)
}

pub async fn select_dataset_ai_handler(
    pg_pool: &PgPool,
    user: &User,
    prompt: &String,
    dataset_id: &Option<Uuid>,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<Dataset> {
    let dataset = match dataset_id {
        Some(dataset_id) => {
            let dataset = match get_user_specified_dataset(&pg_pool, &user.id, &dataset_id).await {
                Ok(dataset) => dataset,
                Err(e) => return Err(e),
            };

            dataset
        }
        None => {
            let datasets = match get_permissioned_datasets(&pg_pool, &user.id).await {
                Ok(datasets) => datasets,
                Err(e) => return Err(e),
            };

            let dataset = match select_dataset_ai_call(&user, &prompt, &datasets, thread_id).await {
                Ok(dataset_name) => datasets
                    .iter()
                    .find(|d| d.database_name == dataset_name)
                    .unwrap()
                    .clone(),
                Err(e) => return Err(e),
            };

            dataset
        }
    };

    let select_dataset_body = IdentifyingDataset {
        progress: StepProgress::Completed,
        dataset: Some(IdentifiedDataset {
            id: dataset.id,
            name: dataset.name.clone(),
        }),
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::IdentifyingDataset,
            Some(select_dataset_body),
        ))
        .await?;

    Ok(dataset)
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RelevantTerm {
    pub id: Uuid,
    pub name: String,
    pub definition: String,
    pub sql_snippet: Option<String>,
}

async fn search_for_relevant_terms(
    pool: &PgPool,
    user_id: &Uuid,
    prompt: &String,
    dataset_id: &Uuid,
) -> Result<Vec<RelevantTerm>> {
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Error getting connection: {e}")),
    };

    let organization_id = match get_user_organization_id(user_id).await {
        Ok(organization_id) => organization_id,
        Err(e) => return Err(anyhow!("Error getting organization ID: {e}")),
    };

    let search_req = SearchRequestObject {
        collection: CollectionName::Terms,
        q: prompt.to_string(),
        query_by: "name,name_embedding".to_string(),
        prefix: true,
        exclude_fields: "name_embedding".to_string(),
        highlight_fields: "name".to_string(),
        use_cache: true,
        filter_by: format!("organization_id:={}", organization_id),
        vector_query: String::from("name_embedding:([], alpha: 0.3)"),
        limit: Some(10),
    };

    let terms_search_results = match typesense::search_documents(vec![search_req]).await {
        Ok(terms_search_results) => terms_search_results,
        Err(e) => return Err(anyhow!("Error searching for relevant terms: {e}")),
    };

    let ids = terms_search_results
        .results
        .iter()
        .map(|result| {
            result
                .hits
                .iter()
                .map(|hit| hit.document.id())
                .collect::<Vec<Uuid>>()
        })
        .flatten()
        .collect::<Vec<Uuid>>();

    if ids.is_empty() {
        return Ok(vec![]);
    }

    let terms = match terms::table
        .inner_join(terms_to_datasets::table.on(terms::id.eq(terms_to_datasets::term_id)))
        .select((
            terms::id,
            terms::name,
            terms::definition.assume_not_null(),
            terms::sql_snippet.nullable(),
        ))
        .filter(terms::organization_id.eq(&organization_id))
        .filter(terms_to_datasets::dataset_id.eq(&dataset_id))
        .filter(terms::id.eq_any(ids))
        .filter(terms::definition.is_not_null())
        .filter(terms_to_datasets::deleted_at.is_null())
        .filter(terms::deleted_at.is_null())
        .load::<(Uuid, String, String, Option<String>)>(&mut conn)
        .await
    {
        Ok(terms) => terms,
        Err(e) => return Err(anyhow!("Error loading terms: {e}")),
    };

    let terms = terms
        .iter()
        .map(|(id, name, definition, sql_snippet)| RelevantTerm {
            id: *id,
            name: name.clone(),
            definition: definition.clone(),
            sql_snippet: sql_snippet.clone(),
        })
        .collect::<Vec<RelevantTerm>>();

    Ok(terms)
}

async fn get_terms_context_string(terms: &HashSet<RelevantTerm>) -> Result<String> {
    if terms.is_empty() {
        return Ok(String::new());
    }

    let mut context_string = "### RELEVANT DOMAIN SPECIFIC LANGUAGE/DEFINITIONS\n".to_string();

    for term in terms {
        if let Some(sql_snippet) = &term.sql_snippet {
            let definition = &term.definition; // definition is always Some
            context_string.push_str(&format!(
                "- `{}`: {}\n   sql_snippet: {}\n",
                term.name, definition, sql_snippet
            ));
        } else {
            context_string.push_str(&format!("- `{}`: {}\n", term.name, term.definition));
        }
    }

    Ok(context_string)
}

async fn get_dataset_ddl(
    pool: &PgPool,
    dataset: &Dataset,
    values: &Vec<StoredValueDocument>,
) -> Result<String> {
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Unable to get connection from pool: {}", e)),
    };

    let dataset_columns: Vec<DatasetColumn> = match dataset_columns::table
        .select(dataset_columns::all_columns)
        .filter(dataset_columns::dataset_id.eq(&dataset.id))
        .filter(dataset_columns::deleted_at.is_null())
        .load::<DatasetColumn>(&mut conn)
        .await
    {
        Ok(columns) => columns,
        Err(e) => {
            return Err(anyhow!(
                "Unable to get dataset columns from database: {}",
                e
            ))
        }
    };

    let ddl = format!(
        "CREATE TABLE {}.{} (
{}
)",
        dataset.schema,
        dataset.database_name,
        dataset_columns
            .iter()
            .map(|c| {
                let relevant_values = values
                    .iter()
                    .filter(|v| v.dataset_column_id == c.id)
                    .collect::<Vec<&StoredValueDocument>>();

                let column_ddl = format!(
                    "{} {} {} {} {}",
                    c.name.clone(),
                    c.type_.clone(),
                    if c.nullable {
                        "NOT NULL".to_string()
                    } else {
                        "".to_string()
                    },
                    if let Some(description) = &c.description {
                        format!(" -- {}", description)
                    } else {
                        "".to_string()
                    },
                    if !relevant_values.is_empty() {
                        format!(
                            " Relevant values: [{}]",
                            relevant_values
                                .iter()
                                .map(|v| v.value.clone())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                    } else {
                        "".to_string()
                    }
                );

                column_ddl
            })
            .collect::<Vec<String>>()
            .join(",\n")
    );

    Ok(ddl)
}

pub async fn get_relevant_values(
    dataset_id: &Uuid,
    prompt: &String,
) -> Result<Vec<StoredValueDocument>> {
    let search_req = SearchRequestObject {
        collection: CollectionName::StoredValues(format!("dataset_index_{}", dataset_id)),
        q: prompt.to_string(),
        query_by: "value,value_embedding".to_string(),
        prefix: true,
        exclude_fields: "value_embedding".to_string(),
        highlight_fields: "value".to_string(),
        use_cache: true,
        filter_by: "".to_string(),
        vector_query: String::from("value_embedding:([], alpha: 0.7)"),
        limit: Some(25),
    };

    let search_results = match typesense::search_documents(vec![search_req]).await {
        Ok(search_results) => search_results,
        Err(e) => return Err(anyhow!("Error searching for relevant values: {e}")),
    };

    let search_results = search_results
        .results
        .iter()
        .flat_map(|result| {
            result
                .hits
                .iter()
                .map(|hit| hit.document.into_stored_value_document().clone())
                .collect::<Vec<StoredValueDocument>>()
        })
        .collect::<Vec<StoredValueDocument>>();

    Ok(search_results)
}

async fn get_user_specified_dataset(
    pool: &PgPool,
    user_id: &Uuid,
    dataset_id: &Uuid,
) -> Result<Dataset> {
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Unable to get connection from pool: {}", e)),
    };

    let dataset: Dataset = match datasets::table
        .select(datasets::all_columns)
        .inner_join(
            datasets_to_permission_groups::table
                .on(datasets::id.eq(datasets_to_permission_groups::dataset_id)),
        )
        .inner_join(
            permission_groups::table
                .on(datasets_to_permission_groups::permission_group_id.eq(permission_groups::id)),
        )
        .inner_join(
            permission_groups_to_identities::table
                .on(permission_groups::id.eq(permission_groups_to_identities::permission_group_id)),
        )
        .inner_join(
            teams_to_users::table
                .on(teams_to_users::team_id.eq(permission_groups_to_identities::identity_id)),
        )
        .filter(
            permission_groups_to_identities::identity_id
                .eq(&user_id)
                .or(teams_to_users::user_id.eq(&user_id)),
        )
        .filter(datasets::deleted_at.is_null())
        .filter(datasets::id.eq(&dataset_id))
        .filter(teams_to_users::deleted_at.is_null())
        .filter(datasets_to_permission_groups::deleted_at.is_null())
        .filter(permission_groups::deleted_at.is_null())
        .first::<Dataset>(&mut conn)
        .await
    {
        Ok(dataset) => dataset,
        Err(diesel::result::Error::NotFound) => {
            tracing::error!("Dataset not found");
            return Err(anyhow!("Dataset not found"));
        }
        Err(e) => {
            tracing::error!("Error querying dataset by ID: {}", e);
            return Err(anyhow!("Error querying dataset by ID: {}", e));
        }
    };

    Ok(dataset)
}

async fn get_permissioned_datasets(pool: &PgPool, user_id: &Uuid) -> Result<Vec<Dataset>> {
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => return Err(anyhow!("Unable to get connection from pool: {}", e)),
    };

    let datasets = match datasets::table
        .select(datasets::all_columns)
        .inner_join(
            datasets_to_permission_groups::table.on(datasets::id
                .eq(datasets_to_permission_groups::dataset_id)
                .and(datasets_to_permission_groups::deleted_at.is_null())),
        )
        .inner_join(
            permission_groups::table.on(datasets_to_permission_groups::permission_group_id
                .eq(permission_groups::id)
                .and(permission_groups::deleted_at.is_null())),
        )
        .inner_join(
            permission_groups_to_identities::table.on(permission_groups::id
                .eq(permission_groups_to_identities::permission_group_id)
                .and(permission_groups_to_identities::deleted_at.is_null())),
        )
        .left_join(
            teams_to_users::table.on(teams_to_users::team_id
                .eq(permission_groups_to_identities::identity_id)
                .and(teams_to_users::deleted_at.is_null())),
        )
        .filter(
            permission_groups_to_identities::identity_id
                .eq(&user_id)
                .or(teams_to_users::user_id.eq(&user_id)),
        )
        .distinct()
        .load::<Dataset>(&mut conn)
        .await
    {
        Ok(datasets) => datasets,
        Err(diesel::result::Error::NotFound) => {
            tracing::error!("Datasets not found");
            return Err(anyhow!("Datasets not found"));
        }
        Err(e) => return Err(anyhow!("Unable to get datasets from database: {}", e)),
    };

    Ok(datasets)
}

async fn get_previously_used_terms(
    pool: Arc<PgPool>,
    thread: Arc<ThreadState>,
) -> Result<HashSet<Term>> {
    let mut previously_used_terms = HashSet::new();

    for message in &thread.messages {
        if let Some(context) = &message.message.context {
            let context_body: ContextJsonBody = match serde_json::from_value(context.clone()) {
                Ok(context_body) => context_body,
                Err(e) => return Err(anyhow!("Error deserializing context: {e}")),
            };

            if let Some(Step::IdentifyingTerms(identifying_terms_body)) = context_body
                .steps
                .iter()
                .find(|step| matches!(step, Step::IdentifyingTerms(_)))
            {
                if let Some(terms) = &identifying_terms_body.terms {
                    for term in terms {
                        if let Ok(Some(term)) = Term::find_by_id(&pool, &term.id).await {
                            previously_used_terms.insert(term);
                        }
                    }
                }
            }
        }
    }

    Ok(previously_used_terms)
}
