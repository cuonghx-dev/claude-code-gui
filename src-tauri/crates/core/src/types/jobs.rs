use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A background job as written by `~/.claude/jobs/<id>/state.json`.
///
/// The on-disk record carries internal bookkeeping this app has no use for
/// (`linkScanOffset`, `respawnFlags`, `interactiveLineage`, …) and one field it
/// must never surface: `providerEnv`, which holds provider credentials. Only
/// its key names cross the IPC boundary — the webview is inspectable, so
/// hiding values in the UI would not be enough.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// Directory name, an 8-hex id.
    pub id: String,
    pub name: String,
    /// `auto` when the CLI named the job, `user` when a person did.
    pub name_source: Option<String>,
    /// `running`, `blocked`, `working`, `done`, `failed`, … not an enum: the
    /// CLI adds states faster than this app can follow, and an unknown one
    /// should render, not break the listing.
    pub state: String,
    /// One-line status, usually the job's most recent summary.
    pub detail: Option<String>,
    /// The prompt that started the job.
    pub intent: Option<String>,
    /// What the job is waiting on, when `state` is blocked.
    pub needs: Option<String>,
    pub backend: Option<String>,
    pub template: Option<String>,
    pub tempo: Option<String>,
    pub cwd: Option<String>,
    /// RFC3339, as written by the CLI.
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub first_terminal_at: Option<String>,
    pub reaped_mid_work_at: Option<String>,
    pub session_id: Option<String>,
    pub resume_session_id: Option<String>,
    /// Artifacts the job produced — PRs, issues, branches.
    pub children: Vec<JobLink>,
    /// Free-form result payload, rendered as JSON.
    pub output: Option<String>,
    #[ts(type = "number | null")]
    pub tokens: Option<i64>,
    pub pinned: bool,
    /// Sorted key names of `providerEnv`. **Never the values.**
    pub provider_env_keys: Vec<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct JobLink {
    pub id: String,
    pub href: Option<String>,
    pub kind: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct TimelineEvent {
    /// RFC3339.
    pub at: Option<String>,
    pub state: Option<String>,
    pub detail: Option<String>,
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct JobDetail {
    pub job: Job,
    pub timeline: Vec<TimelineEvent>,
    /// Lines of `timeline.jsonl` that failed to parse. Surfaced as a footnote
    /// rather than failing the read: the file is appended to live and torn
    /// lines happen.
    #[ts(type = "number")]
    pub malformed_lines: usize,
}
