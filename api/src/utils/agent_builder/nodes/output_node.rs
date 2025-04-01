use serde_json::Value;
use tokio::sync::mpsc;

use super::error_node::ErrorNode;

pub struct OutputNodeSettings {
    pub sender: mpsc::Sender<Value>,
}

pub async fn output_node(settings: OutputNodeSettings, value: Value) -> Result<(), ErrorNode> {
    // Send the value through the channel
    settings
        .sender
        .send(value)
        .await
        .map_err(|e| ErrorNode::new("OutputNodeError".to_string(), e.to_string()))?;

    Ok(())
}
