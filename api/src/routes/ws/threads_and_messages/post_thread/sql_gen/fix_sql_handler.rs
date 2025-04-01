use std::collections::HashSet;

use crate::{
    database::{
        lib::{FixingSql, PgPool, StepProgress},
        models::{Dataset, Term, User},
    },
    routes::ws::threads_and_messages::{
        post_thread::post_thread::PostThreadMessage, threads_router::ThreadEvent,
    },
};
use anyhow::{anyhow, Result};
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn fix_sql_handler(
    pg_pool: &PgPool,
    user: &User,
    sql: &String,
    errors: &Vec<String>,
    dataset: &Dataset,
    terms: &HashSet<Term>,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<String> {
    let ddl = match get_dataset_ddl(&pg_pool, dataset).await {
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

    let errors = errors.join("\n");

    let (mut fix_sql_stream, fixed_sql) = match fix_sql_ai_call(
        &ddl,
        &sql,
        &errors,
        &terms_context_string,
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
        if !sql_started && content.contains("sql") {
            sql_started = true;
        }

        if sql_started && !content.contains("```") && !content.contains("sql") {
            let sql_chunk = FixingSql {
                progress: StepProgress::InProgress,
                sql_chunk: Some(content.to_string()),
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
        Ok(fixed_sql) => fixed_sql,
        Err(e) => return Err(e),
    };

    let fixed_sql = fixed_sql.replace("```sql", "");

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
