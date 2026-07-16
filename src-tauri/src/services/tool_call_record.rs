use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Persistent tool-call record shared by Rig MCP loop and Sidecar SSE path.
/// Field names stay aligned with the frontend `ToolCall` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: String,
    pub server_id: String,
    pub server_name: String,
    pub tool_name: String,
    pub arguments: Value,
    pub result: Option<Value>,
    pub status: String,
    pub error: Option<String>,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
}
