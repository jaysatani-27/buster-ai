use std::sync::Arc;

use crate::{
    database::{lib::MessageResponses, models::User},
    routes::ws::threads_and_messages::thread_utils::DataObject,
};

use super::{
    ai::ai_handlers::{
        generate_data_explanation_ai_call_handler, generate_data_summary_ai_call_handler,
        generate_metric_title_ai_call_handler, generate_no_data_returned_response_ai_call_handler,
        generate_summary_question_ai_call_handler, generate_time_frame_ai_call_handler,
    },
    post_thread::PostThreadMessage,
};
use anyhow::Result;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct ResponsePipelineOutput {
    pub message_responses: MessageResponses,
    pub metric_title: String,
    pub time_frame: String,
    pub summary_question: String,
}

pub async fn response_pipeline(
    user: Arc<User>,
    data_and_data_metadata: Arc<DataObject>,
    sql: Arc<String>,
    prompt: Arc<String>,
    full_prompt: Arc<String>,
    thread_id: Arc<Uuid>,
    message_id: Arc<Uuid>,
    thread_tx: Arc<mpsc::Sender<PostThreadMessage>>,
) -> Result<ResponsePipelineOutput> {
    let metric_title_handle = {
        let title_user = Arc::clone(&user);
        let title_prompt = Arc::clone(&full_prompt);
        let title_sql = Arc::clone(&sql);
        let title_thread_id = Arc::clone(&thread_id);
        let title_message_id = Arc::clone(&message_id);
        let title_thread_tx = Arc::clone(&thread_tx);
        tokio::spawn(async move {
            generate_metric_title_ai_call_handler(
                &title_user,
                &title_prompt,
                &title_sql,
                &title_thread_id,
                &title_message_id,
                &title_thread_tx,
            )
            .await
        })
    };

    let summary_question_handle = {
        let summary_question_user = Arc::clone(&user);
        let summary_question_prompt = Arc::clone(&full_prompt);
        let summary_question_sql = Arc::clone(&sql);
        let summary_question_thread_id = Arc::clone(&thread_id);
        let summary_question_message_id = Arc::clone(&message_id);
        let summary_question_thread_tx = Arc::clone(&thread_tx);
        tokio::spawn(async move {
            generate_summary_question_ai_call_handler(
                &summary_question_user,
                &summary_question_prompt,
                &summary_question_sql,
                &summary_question_thread_id,
                &summary_question_message_id,
                &summary_question_thread_tx,
            )
            .await
        })
    };

    let time_frame_handle = {
        let time_frame_user = Arc::clone(&user);
        let time_frame_prompt = Arc::clone(&prompt);
        let time_frame_sql = Arc::clone(&sql);
        let time_frame_thread_id = Arc::clone(&thread_id);
        let time_frame_message_id = Arc::clone(&message_id);
        let time_frame_thread_tx = Arc::clone(&thread_tx);
        tokio::spawn(async move {
            generate_time_frame_ai_call_handler(
                &time_frame_user,
                &time_frame_prompt,
                &time_frame_sql,
                &time_frame_thread_id,
                &time_frame_message_id,
                &time_frame_thread_tx,
            )
            .await
        })
    };

    let mut ai_responses = Vec::new();

    if data_and_data_metadata.data.len() > 0 {
        let data_summary = match generate_data_summary_ai_call_handler(
            &user,
            &prompt,
            &sql,
            &data_and_data_metadata.data,
            &data_and_data_metadata.data_metadata,
            &thread_id,
            &message_id,
            &thread_tx,
        )
        .await
        {
            Ok(data_summary) => data_summary,
            Err(e) => {
                tracing::error!("Unable to generate data summary: {:?}", e);
                return Err(e);
            }
        };

        let data_explanation = match generate_data_explanation_ai_call_handler(
            &user,
            &prompt,
            &sql,
            &thread_id,
            &message_id,
            &thread_tx,
        )
        .await
        {
            Ok(title) => title,
            Err(e) => {
                tracing::error!("Unable to generate data explanation: {:?}", e);
                return Err(e);
            }
        };

        ai_responses.push(data_summary);
        ai_responses.push(data_explanation);
    } else {
        let no_data_response = match generate_no_data_returned_response_ai_call_handler(
            &user,
            &prompt,
            &sql,
            &thread_id,
            &message_id,
            &thread_tx,
        )
        .await
        {
            Ok(no_data_response) => no_data_response,
            Err(e) => {
                tracing::error!("Unable to generate no data response: {:?}", e);
                return Err(e);
            }
        };

        ai_responses.push(no_data_response);
    }

    let metric_title = match metric_title_handle.await.unwrap() {
        Ok(title) => title,
        Err(e) => {
            tracing::error!("Unable to generate metric title: {:?}", e);
            return Err(e);
        }
    };

    let time_frame = match time_frame_handle.await.unwrap() {
        Ok(time_frame) => time_frame,
        Err(e) => {
            tracing::error!("Unable to generate time frame: {:?}", e);
            return Err(e);
        }
    };

    let summary_question = match summary_question_handle.await.unwrap() {
        Ok(summary_question) => summary_question,
        Err(e) => {
            tracing::error!("Unable to generate summary question: {:?}", e);
            return Err(e);
        }
    };

    let message_responses = MessageResponses {
        messages: ai_responses,
    };

    Ok(ResponsePipelineOutput {
        message_responses,
        metric_title,
        time_frame,
        summary_question,
    })
}
