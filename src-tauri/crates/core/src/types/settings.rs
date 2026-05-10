use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// `~/.claude/settings.json` shape. Tolerant: unknown keys preserved.
#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub default_permission_mode: Option<String>,
    #[serde(default)]
    pub onboarding_completed: Option<bool>,
    /// Catch-all so writes preserve unknown keys.
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

/// App-level config persisted via `tauri-plugin-store`. Distinct from
/// `Settings` (which is the Claude CLI's `~/.claude/settings.json`).
#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default)]
    pub claude_dir_override: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub experimental_hooks_metrics: bool,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

/// First-run wizard result. Persisted to `settings.json` (the public Claude
/// CLI surface) and to `AppConfig` (private to claude-code-gui).
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct SetupPayload {
    #[serde(default)]
    pub claude_dir_override: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub default_permission_mode: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
}
