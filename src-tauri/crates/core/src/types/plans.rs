use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Plain markdown document under `~/.claude/plans/`. No frontmatter required.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub slug: String,
    pub filename: String,
    pub body: String,
    pub file_path: String,
    /// First heading in the body (for list display). Falls back to slug.
    pub title: String,
    pub size_bytes: u64,
    pub modified_at: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct PlanInput {
    pub slug: String,
    pub body: String,
}
