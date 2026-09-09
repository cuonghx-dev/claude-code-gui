use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Dynamic-workflow script under `~/.claude/workflows/`. Plain JavaScript
/// whose first statement is an `export const meta = { … }` object literal.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub slug: String,
    pub filename: String,
    pub body: String,
    pub file_path: String,
    /// `meta.name` — the slash command the workflow runs as. Falls back to slug.
    pub name: String,
    /// `meta.description` — shown in the permission dialog. Empty when absent.
    pub description: String,
    /// `meta.phases[].title`, in declaration order. Empty when absent.
    pub phases: Vec<String>,
    pub size_bytes: u64,
    pub modified_at: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct WorkflowInput {
    pub slug: String,
    pub body: String,
}
