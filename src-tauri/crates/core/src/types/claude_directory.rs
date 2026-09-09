use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Which `.claude` tree an entry belongs to.
#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum ClaudeDirScope {
    /// `~/.claude` (plus the sibling `~/.claude.json`).
    Global,
    /// `<project>/.claude` (plus sibling `CLAUDE.md`, `.mcp.json`, …).
    Project,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum ClaudeDirKind {
    File,
    Dir,
}

/// One node of the explorer tree. Documented entries are always returned even
/// when missing from disk (`exists: false`) so the UI can show what *could*
/// live there; anything else found on disk comes back with `known: false`.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ClaudeDirEntry {
    /// Stable id for documented entries (`global-settings`, …); for on-disk
    /// extras it is the absolute path.
    pub id: String,
    pub label: String,
    pub path: String,
    pub kind: ClaudeDirKind,
    pub exists: bool,
    /// `true` when this entry comes from the documented catalog.
    pub known: bool,
    pub size_bytes: Option<u64>,
    /// Immediate child count for directories.
    pub child_count: Option<u32>,
    pub modified_at: Option<String>,
    /// One-line description from the docs. Empty for on-disk extras.
    pub one_liner: String,
    /// `committed`, `gitignored` or `local` — how the file is usually tracked.
    pub badge: Option<String>,
    pub docs_url: Option<String>,
    /// Immediate children, listed from disk for directories.
    pub children: Vec<ClaudeDirEntry>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ClaudeDirTree {
    pub scope: ClaudeDirScope,
    /// Directory the entry paths are rooted at (`~/.claude`, or the project
    /// working dir for the project scope).
    pub root: String,
    pub exists: bool,
    pub entries: Vec<ClaudeDirEntry>,
}
