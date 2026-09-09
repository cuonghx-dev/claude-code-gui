use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum MemoryKind {
    ClaudeMd,
    /// Deprecated by Claude Code, still read when present.
    ClaudeLocalMd,
    Rule,
    /// Written by an agent for itself. Read-only here.
    AgentMemory,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum MemoryScope {
    User,
    Project,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct MemoryFile {
    /// `<scope>:<kind>:<relative path>`.
    pub id: String,
    pub kind: MemoryKind,
    pub scope: MemoryScope,
    pub path: String,
    pub rel_path: String,
    /// False for a file Claude Code would read if it existed — the UI offers
    /// to create it.
    pub exists: bool,
    #[ts(type = "number | null")]
    pub size_bytes: Option<u64>,
    #[ts(type = "number | null")]
    pub mtime_ms: Option<i64>,
    /// First markdown heading, for the list.
    pub title: Option<String>,
    #[ts(type = "number")]
    pub import_count: u32,
}

/// One `@path` import, resolved.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct MemoryImport {
    /// The path as written after `@`.
    pub raw: String,
    pub resolved_path: String,
    pub exists: bool,
    #[ts(type = "number")]
    pub depth: u32,
    /// The import loops back to a file already on the chain. Marked rather
    /// than treated as an error: the file itself still reads fine.
    pub cyclic: bool,
    pub parent_path: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct MemoryDoc {
    pub file: MemoryFile,
    pub content: String,
    pub imports: Vec<MemoryImport>,
}

/// The file with its imports inlined, as Claude Code would assemble it.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct MemoryPreview {
    pub flattened: String,
    pub sources: Vec<String>,
    pub truncated: bool,
}
