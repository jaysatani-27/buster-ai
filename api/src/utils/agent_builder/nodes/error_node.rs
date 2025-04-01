use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorNode {
    pub error_type: String,
    pub error_message: String,
}

impl ErrorNode {
    pub fn new(error_type: String, error_message: String) -> Self {
        Self {
            error_type,
            error_message,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}: {}", self.error_type, self.error_message)
    }
}
