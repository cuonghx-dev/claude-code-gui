use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A finished terminal session, snapshotted by this app's PTY manager to
/// `~/.claude/cli-history/<id>.json` when the session ends.
///
/// The list view carries metadata only — `lastLines` can hold 10k lines of raw
/// terminal output per session and is loaded on demand.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct CliHistoryEntry {
    pub id: String,
    pub agent_slug: Option<String>,
    pub model: Option<String>,
    pub working_dir: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub last_activity: Option<String>,
    #[ts(type = "number | null")]
    pub exit_code: Option<i64>,
    #[ts(type = "number | null")]
    pub cols: Option<i64>,
    #[ts(type = "number | null")]
    pub rows: Option<i64>,
    #[ts(type = "number")]
    pub line_count: usize,
    /// First non-empty output line with ANSI escapes stripped, truncated.
    pub preview: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct CliHistoryDetail {
    pub entry: CliHistoryEntry,
    /// Raw scrollback, ANSI escapes intact — the frontend replays it through
    /// xterm. Local only: this is whatever was on screen, including anything
    /// the user typed.
    pub lines: Vec<String>,
}
