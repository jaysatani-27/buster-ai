use anyhow::Result;
use indexmap::IndexMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    database::{
        lib::{
            DataMetadataJsonBody, GeneratingMetricTitle, GeneratingResponse,
            GeneratingSummaryQuestion, GeneratingTimeFrame, StepProgress,
        },
        models::User,
    },
    routes::ws::threads_and_messages::{
        post_thread::post_thread::PostThreadMessage, threads_router::ThreadEvent,
    },
    utils::query_engine::data_types::DataType,
};

use super::ai_calls::{
    generate_data_explanation_ai_call, generate_data_summary_ai_call,
    generate_metric_title_ai_call, generate_no_data_returned_response_ai_call,
    generate_summary_question_ai_call, generate_time_frame_ai_call,
};

pub async fn generate_data_summary_ai_call_handler(
    user: &User,
    prompt: &String,
    sql: &String,
    data: &Vec<IndexMap<String, DataType>>,
    data_metadata: &DataMetadataJsonBody,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<String> {
    let (mut stream, data_summary) =
        match generate_data_summary_ai_call(prompt, sql, data, data_metadata, thread_id, &user.id)
            .await
        {
            Ok(data_summary_stream) => data_summary_stream,
            Err(e) => return Err(e),
        };

    while let Some(content) = stream.recv().await {
        let data_summary_chunk = GeneratingResponse {
            progress: StepProgress::InProgress,
            text_chunk: Some(content.to_string()),
            text: None,
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
        };

        thread_tx
            .send(PostThreadMessage::new(
                ThreadEvent::GeneratingResponse,
                data_summary_chunk,
            ))
            .await?;
    }

    let data_summary = match data_summary.await? {
        Ok(data_summary) => data_summary,
        Err(e) => return Err(e),
    };

    let completed_data_summary_response = GeneratingResponse {
        progress: StepProgress::Completed,
        text: Some(data_summary.clone()),
        text_chunk: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::GeneratingResponse,
            completed_data_summary_response,
        ))
        .await?;

    Ok(data_summary)
}

pub async fn generate_no_data_returned_response_ai_call_handler(
    user: &User,
    prompt: &String,
    sql: &String,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<String> {
    let (mut stream, no_data_response) =
        match generate_no_data_returned_response_ai_call(prompt, sql, thread_id, &user.id).await {
            Ok(no_data_response_stream) => no_data_response_stream,
            Err(e) => return Err(e),
        };

    while let Some(content) = stream.recv().await {
        let no_data_response_chunk = GeneratingResponse {
            progress: StepProgress::InProgress,
            text_chunk: Some(content.to_string()),
            text: None,
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
        };

        thread_tx
            .send(PostThreadMessage::new(
                ThreadEvent::GeneratingResponse,
                no_data_response_chunk,
            ))
            .await?;
    }

    let no_data_response = match no_data_response.await? {
        Ok(no_data_response) => no_data_response,
        Err(e) => return Err(e),
    };

    let no_data_complete_response = GeneratingResponse {
        progress: StepProgress::Completed,
        text: Some(no_data_response.clone()),
        text_chunk: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::GeneratingResponse,
            no_data_complete_response,
        ))
        .await?;

    Ok(no_data_response)
}

pub async fn generate_data_explanation_ai_call_handler(
    user: &User,
    prompt: &String,
    sql: &String,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<String> {
    let (mut stream, data_explanation) =
        match generate_data_explanation_ai_call(prompt, sql, thread_id, &user.id).await {
            Ok(data_explanation_stream) => data_explanation_stream,
            Err(e) => return Err(e),
        };

    while let Some(content) = stream.recv().await {
        let data_explanation_chunk = GeneratingResponse {
            progress: StepProgress::InProgress,
            text_chunk: Some(content.to_string()),
            text: None,
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
        };

        thread_tx
            .send(PostThreadMessage::new(
                ThreadEvent::GeneratingResponse,
                data_explanation_chunk,
            ))
            .await?;
    }

    let data_explanation = match data_explanation.await? {
        Ok(data_explanation) => data_explanation,
        Err(e) => return Err(e),
    };

    let completed_data_explanation_response = GeneratingResponse {
        progress: StepProgress::Completed,
        text: Some(data_explanation.clone()),
        text_chunk: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::GeneratingResponse,
            completed_data_explanation_response,
        ))
        .await?;

    Ok(data_explanation)
}

pub async fn generate_metric_title_ai_call_handler(
    user: &User,
    prompt: &String,
    sql: &String,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<String> {
    let (mut stream, metric_title) =
        match generate_metric_title_ai_call(prompt, sql, thread_id, &user.id).await {
            Ok(metric_title_stream) => metric_title_stream,
            Err(e) => return Err(e),
        };

    while let Some(content) = stream.recv().await {
        let metric_title_chunk = GeneratingMetricTitle {
            progress: StepProgress::InProgress,
            metric_title_chunk: Some(content.to_string()),
            metric_title: None,
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
        };

        thread_tx
            .send(PostThreadMessage::new(
                ThreadEvent::GeneratingMetricTitle,
                metric_title_chunk,
            ))
            .await?;
    }

    let metric_title = match metric_title.await? {
        Ok(metric_title) => metric_title,
        Err(e) => return Err(e),
    };

    let completed_metric_title_response = GeneratingMetricTitle {
        progress: StepProgress::Completed,
        metric_title: Some(metric_title.clone()),
        metric_title_chunk: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::GeneratingMetricTitle,
            completed_metric_title_response,
        ))
        .await?;

    Ok(metric_title)
}

pub async fn generate_summary_question_ai_call_handler(
    user: &User,
    prompt: &String,
    sql: &String,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<String> {
    let (mut stream, summary_question) =
        match generate_summary_question_ai_call(prompt, sql, thread_id, &user.id).await {
            Ok(metric_title_stream) => metric_title_stream,
            Err(e) => return Err(e),
        };

    while let Some(content) = stream.recv().await {
        let summary_question_chunk = GeneratingSummaryQuestion {
            progress: StepProgress::InProgress,
            summary_question_chunk: Some(content.to_string()),
            summary_question: None,
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
        };

        thread_tx
            .send(PostThreadMessage::new(
                ThreadEvent::GeneratingSummaryQuestion,
                summary_question_chunk,
            ))
            .await?;
    }

    let summary_question = match summary_question.await? {
        Ok(summary_question) => summary_question,
        Err(e) => return Err(e),
    };

    let completed_summary_question_response = GeneratingSummaryQuestion {
        progress: StepProgress::Completed,
        summary_question: Some(summary_question.clone()),
        summary_question_chunk: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::GeneratingSummaryQuestion,
            completed_summary_question_response,
        ))
        .await?;

    Ok(summary_question)
}

pub async fn generate_time_frame_ai_call_handler(
    user: &User,
    prompt: &String,
    sql: &String,
    thread_id: &Uuid,
    message_id: &Uuid,
    thread_tx: &mpsc::Sender<PostThreadMessage>,
) -> Result<String> {
    let (mut stream, time_frame) =
        match generate_time_frame_ai_call(prompt, sql, thread_id, &user.id).await {
            Ok(time_frame_stream) => time_frame_stream,
            Err(e) => return Err(e),
        };

    while let Some(content) = stream.recv().await {
        let time_frame_chunk = GeneratingTimeFrame {
            progress: StepProgress::InProgress,
            time_frame_chunk: Some(content.to_string()),
            time_frame: None,
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
        };

        thread_tx
            .send(PostThreadMessage::new(
                ThreadEvent::GeneratingTimeFrame,
                time_frame_chunk,
            ))
            .await?;
    }

    let time_frame = match time_frame.await? {
        Ok(time_frame) => time_frame,
        Err(e) => return Err(e),
    };

    let completed_time_frame_response = GeneratingTimeFrame {
        progress: StepProgress::Completed,
        time_frame: Some(time_frame.clone()),
        time_frame_chunk: None,
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
    };

    thread_tx
        .send(PostThreadMessage::new(
            ThreadEvent::GeneratingTimeFrame,
            completed_time_frame_response,
        ))
        .await?;

    Ok(time_frame)
}
