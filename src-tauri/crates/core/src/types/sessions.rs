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

    /// Chains a subagent's messages back to the turn that spawned them.
    #[serde(default)]
    pub parent_uuid: Option<String>,
    /// True for messages inside a subagent (Task tool) conversation.
    #[serde(default)]
    pub is_sidechain: bool,
    #[serde(default)]
    pub git_branch: Option<String>,
    /// Raw top-level record `type`, so the UI can label the records that are
    /// not chat turns (`system`, `mode`, `permission-mode`).
    #[serde(default)]
    pub record_type: Option<String>,
    /// `tool_use.id` on a call, `tool_result.tool_use_id` on its result — the
    /// key the UI pairs them on.
    #[serde(default)]
    pub tool_use_id: Option<String>,

    /// Model and usage belong to the *record*, not the block. One assistant
    /// record can expand into five messages, so they are attached only to the
    /// first — otherwise a five-block turn counts its tokens five times.
    #[serde(default)]
    pub is_turn_head: bool,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub usage: Option<TokenUsage>,
    #[serde(default)]
    pub cost_usd: Option<f64>,

    /// Tool results are capped before they cross the IPC boundary: Bash and
    /// Read output runs to hundreds of KB and routinely contains secrets from
    /// files the agent read. Both a performance and a privacy control.
    #[serde(default)]
    pub tool_result_truncated: bool,
    #[serde(default)]
    #[ts(type = "number")]
    pub tool_result_bytes: usize,
}

/// A subagent (Task tool) conversation, rolled up for the collapsed card that
/// stands in for it in the main transcript.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub root_uuid: String,
    /// The `tool_use` this thread answers, when it could be resolved.
    pub parent_tool_use_id: Option<String>,
    pub agent_name: Option<String>,
    #[ts(type = "number")]
    pub message_count: usize,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub usage: TokenUsage,
    pub cost_usd: Option<f64>,
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
    /// A `system` record: CLI notices, not conversation.
    System,
    /// Session bookkeeping the CLI writes inline — `mode`, `permission-mode`.
    Meta,
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
