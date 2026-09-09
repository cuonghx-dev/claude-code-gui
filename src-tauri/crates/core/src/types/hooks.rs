use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::SettingsScope;

/// One matcher group for a hook event. Mirrors the `hooks.<event>[]`
/// shape in `~/.claude/settings.json`:
///
/// ```jsonc
/// {
///   "matcher": "Bash|Grep",   // optional; absent => all
///   "hooks": [ { "type": "command", "command": "...", ... } ]
/// }
/// ```
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct HookGroup {
    /// `<scope>:<event>:<index>` — addresses the group for edit and delete.
    pub id: String,
    /// Which settings file this group came from.
    pub scope: SettingsScope,
    /// Position within that file's array for this event.
    #[ts(type = "number")]
    pub index: u32,
    pub file_path: String,
    /// Event name from settings.json (PreToolUse, PostToolUse,
    /// UserPromptSubmit, SessionStart, …). Kept as-is — display labels are
    /// the frontend's job.
    pub event: String,
    #[serde(default)]
    pub matcher: Option<String>,
    #[serde(default)]
    pub entries: Vec<HookEntry>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct HookEntry {
    /// Always "command" today, but kept flexible.
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    #[ts(type = "number | null")]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub status_message: Option<String>,
    /// Entry keys this app does not model, kept so an edit cannot drop them.
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// A hook group to write. `expected_mtime_ms` carries the optimistic-concurrency
/// check through to the settings file.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct HookInput {
    pub scope: SettingsScope,
    pub working_dir: Option<String>,
    pub event: String,
    pub matcher: Option<String>,
    pub entries: Vec<HookEntry>,
    #[ts(type = "number | null")]
    pub expected_mtime_ms: Option<i64>,
}
