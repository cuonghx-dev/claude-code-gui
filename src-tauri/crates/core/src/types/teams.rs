use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An agent team as written by `~/.claude/teams/<session-id>/config.json`.
///
/// Timestamps are epoch milliseconds and stay `i64` all the way to the
/// frontend, which formats them with `Intl.DateTimeFormat`. (`SessionSummary`
/// converts to RFC3339 strings in Rust; new epoch-ms sources do not — pick one
/// per type and say which in the doc comment.)
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Team {
    /// Directory name, e.g. `session-3a5d9134`. Stable identifier.
    pub id: String,
    pub name: String,
    /// ts-rs maps `i64` to `bigint`, but Tauri hands JS a plain JSON number —
    /// and epoch ms is well inside f64's exact range.
    #[ts(type = "number")]
    pub created_at_ms: i64,
    pub lead_agent_id: Option<String>,
    pub lead_session_id: Option<String>,
    pub members: Vec<TeamMember>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub agent_id: String,
    pub name: Option<String>,
    pub agent_type: Option<String>,
    #[ts(type = "number | null")]
    pub joined_at_ms: Option<i64>,
    pub tmux_pane_id: Option<String>,
    pub cwd: Option<String>,
    pub subscriptions: Vec<String>,
    pub backend_type: Option<String>,
}
