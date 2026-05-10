use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub session_id: String,
    pub project_name: String,
    pub file_path: String,
    pub started_at: Option<String>,
    pub last_message_at: Option<String>,
    pub message_count: usize,
    pub size_bytes: u64,
    /// First user message content, truncated. For list display.
    pub preview: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Page<T: TS + 'static + Send + Sync> {
    pub items: Vec<T>,
    pub next_after: Option<usize>,
    pub total: Option<usize>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: String,
    pub kind: MessageKind,
    pub role: Option<Role>,
    pub timestamp: Option<String>,
    pub content: Option<String>,
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_result: Option<serde_json::Value>,
    pub thinking: Option<String>,
    #[serde(default)]
    pub is_error: bool,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "kebab-case")]
pub enum MessageKind {
    #[default]
    Text,
    Thinking,
    ToolUse,
    ToolResult,
    Image,
    Status,
    Error,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}
