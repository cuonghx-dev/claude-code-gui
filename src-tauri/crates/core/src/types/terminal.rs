use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Options for `terminal_session_create`. The terminal is a strict Claude
/// wrapper — at least one of `agent_slug`, `resume_session_id`, or
/// `command_template` must be present (see compose_command).
#[derive(Deserialize, Serialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct TerminalOpts {
    #[serde(default)]
    pub agent_slug: Option<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    pub cols: u16,
    pub rows: u16,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<PermissionMode>,
    #[serde(default)]
    pub output_style_id: Option<String>,
    #[serde(default)]
    pub resume_session_id: Option<String>,
    /// When set, the terminal is bound to a slash command — its body is
    /// piped to claude as the initial prompt and `command_template` itself
    /// is just an opaque identifier the UI uses for display.
    #[serde(default)]
    pub command_template: Option<String>,
}

#[derive(Deserialize, Serialize, TS, Debug, Clone, Copy)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
pub enum PermissionMode {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "acceptEdits")]
    AcceptEdits,
    #[serde(rename = "bypassPermissions")]
    BypassPermissions,
    #[serde(rename = "plan")]
    Plan,
}

impl PermissionMode {
    pub fn as_cli_flag(&self) -> &'static str {
        match self {
            PermissionMode::Default => "default",
            PermissionMode::AcceptEdits => "acceptEdits",
            PermissionMode::BypassPermissions => "bypassPermissions",
            PermissionMode::Plan => "plan",
        }
    }
}

/// Metadata surfaced to the UI for the sidebar / sessions list.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct TerminalSession {
    pub id: String,
    pub agent_slug: Option<String>,
    pub working_dir: Option<String>,
    pub cols: u16,
    pub rows: u16,
    pub model: Option<String>,
    /// ISO 8601 of session creation.
    pub started_at: String,
    /// ISO 8601 of last input/output activity.
    pub last_activity: String,
    pub alive: bool,
}

/// Live tool-call event surfaced to the context panel.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    pub name: String,
    pub state: String, // "started" | "completed" | "failed"
    #[serde(default)]
    pub duration_ms: Option<u32>,
    pub timestamp: String,
}
