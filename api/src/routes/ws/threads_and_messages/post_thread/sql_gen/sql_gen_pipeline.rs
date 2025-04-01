use std::{collections::HashSet, sync::Arc};

use anyhow::{anyhow, Result};
use tokio::sync::mpsc;

use uuid::Uuid;

use crate::{
    database::{
        lib::{
            GeneratingSql, IdentifiedDataset, IdentifyingDataset, IdentifyingTerms, PgPool,
            StepProgress,
        },
        models::{Dataset, User},
    },
    routes::ws::threads_and_messages::{
        post_thread::post_thread::PostThreadMessage, thread_utils::ThreadState,
        threads_router::ThreadEvent,
    },
    utils::clients::typesense::StoredValueDocument,
};

use super::sql_gen_handlers::{
    generate_sql_handler, get_data_source_type, get_relevant_values, identifying_terms_handler,
    select_dataset_ai_handler, RelevantTerm,
};

pub async fn sql_gen_pipeline(
    pg_pool: Arc<PgPool>,
    user: Arc<User>,
    prompt: Arc<String>,
    dataset_id: Arc<Option<Uuid>>,
    thread: Arc<ThreadState>,
    thread_tx: Arc<mpsc::Sender<PostThreadMessage>>,
) -> Result<(
    Option<String>,
    Dataset,
    HashSet<RelevantTerm>,
    Vec<StoredValueDocument>,
)> {
    let last_message = match thread.messages.last() {
        Some(last_message) => last_message,
        None => {
            return Err(anyhow::anyhow!(
                "Thread has no messages [UUID: f47ac10b-58cc-4372-a567-0e02b2c3d479]"
            ))
        }
    };

    let full_prompt = thread.messages.iter().fold(String::new(), |acc, m| {
        acc + &format!("- {}\n", m.message.message)
    });

    match send_identifying_dataset_in_progress_to_sub(
        &thread.thread.id,
        &last_message.message.id,
        thread_tx.as_ref(),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => return Err(e),
    };

    let dataset = match select_dataset_ai_handler(
        pg_pool.as_ref(),
        user.as_ref(),
        &full_prompt,
        dataset_id.as_ref(),
        &thread.thread.id,
        &last_message.message.id,
        thread_tx.as_ref(),
    )
    .await
    {
        Ok(dataset) => dataset,
        Err(e) => {
            tracing::error!(
                "Unable to select dataset: {:?} [UUID: 550e8400-e29b-41d4-a716-446655440000]",
                e
            );
            return Err(e);
        }
    };

    let data_source_type = match get_data_source_type(&pg_pool, &dataset.id).await {
        Ok(data_source_type) => data_source_type,
        Err(e) => return Err(anyhow!("Failed to get data source type: {}", e)),
    };

    match send_identifying_dataset_success_to_sub(
        &dataset,
        &thread.thread.id,
        &last_message.message.id,
        thread_tx.as_ref(),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => return Err(e),
    };

    match send_identifying_terms_in_progress_to_sub(
        &thread.thread.id,
        &last_message.message.id,
        thread_tx.as_ref(),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => return Err(e),
    };

    let terms = match identifying_terms_handler(
        pg_pool.as_ref(),
        user.as_ref(),
        &thread,
        prompt.as_ref(),
        &dataset.id,
        &last_message.message.id,
        thread_tx.as_ref(),
    )
    .await
    {
        Ok(terms) => terms,
        Err(e) => {
            tracing::error!("Unable to generate identifying terms: {:?} [UUID: 7c9e6679-7425-40de-944b-e07fc1f90ae7]", e);
            return Err(e);
        }
    };

    match send_generating_sql_in_progress_to_sub(
        &thread.thread.id,
        &last_message.message.id,
        thread_tx.as_ref(),
    )
    .await
    {
        Ok(_) => (),
        Err(e) => return Err(e),
    };

    let stored_values = match get_relevant_values(&dataset.id, prompt.as_ref()).await {
        Ok(stored_values) => stored_values,
        Err(e) => {
            tracing::error!(
                "Unable to get relevant values: {:?} [UUID: 8c9e6679-7425-40de-944b-e07fc1f90ae8]",
                e
            );
            return Err(e);
        }
    };

    let sql = match generate_sql_handler(
        pg_pool.as_ref(),
        user.as_ref(),
        &thread,
        &dataset,
        &terms,
        &stored_values,
        &last_message.message.id,
        &data_source_type,
        thread_tx.as_ref(),
    )
    .await
    {
        Ok(sql) => sql,
        Err(e) => {
            tracing::error!(
                "Unable to generate SQL: {:?} [UUID: 8c9e6679-7425-40de-944b-e07fc1f90ae8]",
                e
            );
            return Err(e);
        }
    };

    Ok((sql, dataset, terms, stored_values))
}

async fn send_identifying_dataset_in_progress_to_sub(
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<()> {
    let identify_dataset_body = IdentifyingDataset {
        progress: StepProgress::InProgress,
        dataset: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    match thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::IdentifyingDataset,
            identify_dataset_body,
        ))
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!(
                "Failed to send identifying dataset in progress to subscription: {} [UUID: 9c9e6679-7425-40de-944b-e07fc1f90ae9]",
                e
            ))
        }
    }

    Ok(())
}

async fn send_identifying_dataset_success_to_sub(
    dataset: &Dataset,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<()> {
    let identify_dataset_body = IdentifyingDataset {
        progress: StepProgress::Completed,
        dataset: Some(IdentifiedDataset {
            id: dataset.id,
            name: dataset.name.clone(),
        }),
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    match thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::IdentifyingDataset,
            identify_dataset_body,
        ))
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!(
                "Failed to send identifying dataset success to subscription: {} [UUID: ac9e6679-7425-40de-944b-e07fc1f90aea]",
                e
            ))
        }
    }

    Ok(())
}

async fn send_identifying_terms_in_progress_to_sub(
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<()> {
    let identify_terms_body = IdentifyingTerms {
        progress: StepProgress::InProgress,
        terms: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    match thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::IdentifyingTerms,
            identify_terms_body,
        ))
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!(
                "Failed to send identifying terms in progress to subscription: {} [UUID: bc9e6679-7425-40de-944b-e07fc1f90aeb]",
                e
            ))
        }
    }

    Ok(())
}

async fn send_generating_sql_in_progress_to_sub(
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<()> {
    let generating_sql_body = GeneratingSql {
        progress: StepProgress::InProgress,
        sql_chunk: None,
        sql: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    match thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::GeneratingSql,
            generating_sql_body,
        ))
        .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!(
                "Failed to send generating sql in progress to subscription: {} [UUID: cc9e6679-7425-40de-944b-e07fc1f90aec]",
                e
            ))
        }
    }

    Ok(())
}
