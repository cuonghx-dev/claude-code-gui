use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// `<encoded>` directory name under `~/.claude/projects/`. `/` → `-`.
    pub name: String,
    /// Decoded absolute path of the working directory.
    pub working_dir: String,
    /// Number of session JSONLs found.
    pub session_count: usize,
    /// Most recent session timestamp (ISO 8601). `None` if no sessions.
    pub last_active: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub working_dir: String,
    pub exists_on_disk: bool,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub files: Vec<GitFileStatus>,
    pub clean: bool,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct GitFileStatus {
    pub path: String,
    pub status: String, // "M", "A", "D", "??", "R", "C", "U"
    pub staged: bool,
}
