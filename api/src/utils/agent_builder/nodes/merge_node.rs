use futures::future::join_all;
use serde_json::Value;
use std::fmt;
use tokio::task::JoinHandle;

use super::error_node::ErrorNode;

pub struct MergeNodeSettings {
    futures: Vec<JoinHandle<Result<Value, ErrorNode>>>,
}

impl MergeNodeSettings {
    pub fn new(futures: Vec<JoinHandle<Result<Value, ErrorNode>>>) -> Self {
        Self { futures }
    }
}

pub enum MergeNodeError {
    TaskError,
    JoinError,
}

impl fmt::Display for MergeNodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskError => write!(f, "task_error"),
            Self::JoinError => write!(f, "join_error"),
        }
    }
}

pub async fn merge_node(settings: MergeNodeSettings) -> Result<Vec<Value>, ErrorNode> {
    let results = join_all(settings.futures)
        .await
        .into_iter()
        .map(|join_result| {
            join_result
                .map_err(|e| ErrorNode::new(MergeNodeError::JoinError.to_string(), e.to_string()))?
                .map_err(|e| ErrorNode::new(MergeNodeError::TaskError.to_string(), e.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}
