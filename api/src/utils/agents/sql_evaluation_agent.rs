use serde_json::Value;
use std::fmt;
use tokio::sync::mpsc;

use crate::utils::{
    agent_builder::nodes::{
        error_node::ErrorNode,
        prompt_node::{prompt_node, PromptNodeMessage, PromptNodeSettings},
    },
    prompts::sql_evaluator_prompts::{
        sql_evaluation_summary_prompts::{
            sql_evaluation_summary_system_prompt, sql_evaluation_summary_user_prompt,
        },
        sql_evaluator_prompts::{
            sql_evaluation_json_schema, sql_evaluation_system_prompt, sql_evaluation_user_prompt,
        },
    },
};

pub enum SqlEvaluationAgentError {
    MissingKey,
    ObjectNotJson,
    PromptNodeError,
    GenericError,
}

impl fmt::Display for SqlEvaluationAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey => write!(f, "missing_key"),
            Self::ObjectNotJson => write!(f, "object_not_json"),
            Self::PromptNodeError => write!(f, "prompt_node_error"),
            Self::GenericError => write!(f, "generic_error"),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SqlEvaluationResponse {
    pub answerable: EvaluationField<bool>,
    pub accuracy_of_sql_semantics: EvaluationField<bool>,
    pub level_of_assumptions: EvaluationField<String>,
    pub avoidance_of_hallucinations: EvaluationField<bool>,
    pub not_doing_extra: EvaluationField<bool>,
    pub appropriate_use_of_available_data: EvaluationField<bool>,
    pub user_satisfaction: EvaluationField<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EvaluationField<T> {
    pub value: T,
    pub explanation: String,
}

pub struct SqlEvaluationAgentOptions {
    pub request: String,
    pub sql: String,
    pub datasets: String,
    pub output_sender: mpsc::Sender<Value>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SqlEvaluationResult {
    pub evaluation_obj: SqlEvaluationResponse,
    pub evaluation_summary: String,
    pub score: String,
    pub request: String,
    pub sql: String,
}

pub async fn sql_evaluation_agent(
    options: SqlEvaluationAgentOptions,
) -> Result<SqlEvaluationResult, ErrorNode> {
    let sql_evaluation_prompt_settings = PromptNodeSettings {
        messages: create_sql_evaluation_messages(&options.request, &options.sql, &options.datasets),
        prompt_name: "sql_evaluation".to_string(),
        json_schema: Some(sql_evaluation_json_schema()),
        ..Default::default()
    };

    let evaluation_response = match prompt_node(sql_evaluation_prompt_settings).await {
        Ok(response) => response,
        Ok(_) => {
            return Err(ErrorNode::new(
                SqlEvaluationAgentError::ObjectNotJson.to_string(),
                "SQL evaluation response is not a string".to_string(),
            ));
        }
        Err(e) => {
            return Err(e);
        }
    };

    let evaluation_obj: SqlEvaluationResponse = match serde_json::from_value(evaluation_response) {
        Ok(obj) => obj,
        Err(e) => {
            return Err(ErrorNode::new(
                SqlEvaluationAgentError::ObjectNotJson.to_string(),
                e.to_string(),
            ));
        }
    };

    let score = score_sql_evaluation(&evaluation_obj);

    let evaluation_summary_options = PromptNodeSettings {
        messages: create_sql_evaluation_summary_messages(&score, &evaluation_obj),
        prompt_name: "sql_evaluation_summary".to_string(),
        ..Default::default()
    };

    let evaluation_summary_response = match prompt_node(evaluation_summary_options).await {
        Ok(Value::String(response)) => response,
        Ok(_) => {
            return Err(ErrorNode::new(
                SqlEvaluationAgentError::ObjectNotJson.to_string(),
                "SQL evaluation summary response is not a string".to_string(),
            ));
        }
        Err(e) => {
            return Err(e);
        }
    };

    Ok(SqlEvaluationResult {
        evaluation_obj,
        evaluation_summary: evaluation_summary_response,
        score,
        request: options.request,
        sql: options.sql,
    })
}

fn create_sql_evaluation_messages(
    input: &String,
    sql: &String,
    datasets: &String,
) -> Vec<PromptNodeMessage> {
    vec![
        PromptNodeMessage {
            role: "system".to_string(),
            content: sql_evaluation_system_prompt(),
        },
        PromptNodeMessage {
            role: "user".to_string(),
            content: sql_evaluation_user_prompt(input, sql, datasets),
        },
    ]
}

fn create_sql_evaluation_summary_messages(
    score: &String,
    sql_confidence_score: &SqlEvaluationResponse,
) -> Vec<PromptNodeMessage> {
    let sql_confidence_score_json =
        serde_json::to_string(sql_confidence_score).unwrap_or("".to_string());

    vec![
        PromptNodeMessage {
            role: "system".to_string(),
            content: sql_evaluation_summary_system_prompt(),
        },
        PromptNodeMessage {
            role: "user".to_string(),
            content: sql_evaluation_summary_user_prompt(score, &sql_confidence_score_json),
        },
    ]
}

fn score_sql_evaluation(evaluation_obj: &SqlEvaluationResponse) -> String {
    let mut lowest_score = "High".to_string();

    // Helper to update lowest_score
    let mut update_score = |new_score: &str| {
        if (lowest_score == "High" && (new_score == "Moderate" || new_score == "Low"))
            || (lowest_score == "Moderate" && new_score == "Low")
        {
            lowest_score = new_score.to_string();
        }
    };

    // Check critical boolean checks
    if !evaluation_obj.answerable.value
        || !evaluation_obj.accuracy_of_sql_semantics.value
        || !evaluation_obj.avoidance_of_hallucinations.value
        || !evaluation_obj.not_doing_extra.value
        || !evaluation_obj.appropriate_use_of_available_data.value
    {
        update_score("Low");
    }

    // Check level of assumptions
    match evaluation_obj.level_of_assumptions.value.as_str() {
        "straightforward with no assumptions" => update_score("High"),
        "straightforward with minor assumptions" => update_score("High"),
        "not straightforward and major assumptions" => update_score("Low"),
        _ => (),
    }

    lowest_score
}

// SAVING FOR LATER

// fn score_sql_evaluation(evaluation_obj: &SqlEvaluationResponse) -> String {
//     let mut score = 0;

//     // Boolean fields (true = high, false = low)
//     if evaluation_obj.answerable.value { score += 1; }
//     if evaluation_obj.accuracy_of_sql_semantics.value { score += 1; }
//     if evaluation_obj.avoidance_of_hallucinations.value { score += 1; }
//     if evaluation_obj.not_doing_extra.value { score += 1; }
//     if evaluation_obj.appropriate_use_of_available_data.value { score += 1; }

//     // String fields with levels
//     score += match evaluation_obj.level_of_assumptions.value.as_str() {
//         "Completely" => 0,
//         "Mostly" => 1,
//         "Partially" => 2,
//         "Not at all" => 3,
//         _ => 0,
//     };

//     score += match evaluation_obj.alignment_with_user_request.value.as_str() {
//         "High" => 2,
//         "Moderate" => 1,
//         "Low" => 0,
//         _ => 0,
//     };

//     score += match evaluation_obj.user_satisfaction.value.as_str() {
//         "straightforward with no assumptions" => 2,
//         "straightforward with minor assumptions" => 1,
//         "not straightforward and major assumptions" => 0,
//         _ => 0,
//     };

//     // Max possible score is 11
//     match score {
//         0..=3 => "Low".to_string(),
//         4..=7 => "Moderate".to_string(),
//         _ => "High".to_string(),
//     }
// }
