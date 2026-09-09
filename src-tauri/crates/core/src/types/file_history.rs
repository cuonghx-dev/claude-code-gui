use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A point in a session where the CLI snapshotted the files it was about to
/// change.
///
/// The index lives in the transcript (`file-history-snapshot` and
/// `file-history-delta` records); the file contents live as raw blobs under
/// `~/.claude/file-history/<session-id>/<16-hex>@v<N>`. A checkpoint is the
/// join of the two.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Checkpoint {
    /// The assistant message this snapshot belongs to.
    pub message_id: String,
    pub timestamp: Option<String>,
    pub files: Vec<CheckpointFile>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct CheckpointFile {
    /// Absolute path of the file this version was taken from.
    pub tracking_path: String,
    /// Blob name, e.g. `0b642a3eaf179c2b@v1`.
    pub backup_file_name: String,
    #[ts(type = "number")]
    pub version: u32,
    pub backup_time: Option<String>,
    /// False when the transcript references a blob that is no longer on disk —
    /// the interesting case, so it is surfaced rather than dropped.
    pub exists: bool,
    #[ts(type = "number")]
    pub size_bytes: u64,
}

/// One side of a diff: a stored blob, or the file as it stands now.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "kind")]
pub enum DiffSide {
    Blob { backup_file_name: String },
    WorkingTree { path: String },
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct DiffResult {
    pub left_label: String,
    pub right_label: String,
    pub hunks: Vec<DiffHunk>,
    /// The diff exceeded the line cap and was cut short.
    pub truncated: bool,
    /// One side looks like binary content, so no line diff was attempted.
    pub binary: bool,
    pub left_missing: bool,
    pub right_missing: bool,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    #[ts(type = "number")]
    pub left_start: u32,
    #[ts(type = "number")]
    pub right_start: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub tag: DiffTag,
    #[ts(type = "number | null")]
    pub left_no: Option<u32>,
    #[ts(type = "number | null")]
    pub right_no: Option<u32>,
    pub text: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum DiffTag {
    Equal,
    Insert,
    Delete,
}
