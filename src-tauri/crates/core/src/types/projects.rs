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

/// A git worktree of a project.
///
/// libgit2's `worktrees()` lists only *linked* worktrees, so the main checkout
/// is synthesized and flagged — a list that silently omits the directory you
/// are standing in would be worse than no list.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub name: String,
    pub path: String,
    pub branch: Option<String>,
    /// Short commit id at HEAD.
    pub head: Option<String>,
    pub is_main: bool,
    /// The worktree the caller asked about.
    pub is_current: bool,
    pub is_locked: bool,
    pub lock_reason: Option<String>,
    /// The working directory is gone but the administrative files remain.
    pub prunable: bool,
}

/// `.worktreeinclude`: the gitignored files a new worktree should still get.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInclude {
    pub path: String,
    pub exists: bool,
    /// Patterns as written, comments and blanks dropped.
    pub patterns: Vec<String>,
    /// Files in the project that match, so the effect is visible.
    pub matched_files: Vec<String>,
    pub truncated: bool,
}
