use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
    pub timeout: Option<u64>,
    #[serde(default)]
    pub status_message: Option<String>,
}
