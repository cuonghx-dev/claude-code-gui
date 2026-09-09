use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One keystroke-to-action binding, flattened out of the file's per-context
/// blocks so the UI can render a single table.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Keybinding {
    /// `Chat`, `Global`, … The file groups by this; the IPC type does not.
    pub context: String,
    /// A chord such as `ctrl+k` or `ctrl+k ctrl+s`.
    pub key: String,
    /// `None` means the JSON value was `null`, which unbinds a default.
    pub action: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct KeybindingsDoc {
    pub path: String,
    /// The file often does not exist until `/keybindings` creates it.
    pub exists: bool,
    #[ts(type = "number | null")]
    pub mtime_ms: Option<i64>,
    pub bindings: Vec<Keybinding>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ChordValidation {
    pub valid: bool,
    pub normalized: Option<String>,
    pub issues: Vec<String>,
}
