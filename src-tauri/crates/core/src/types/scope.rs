use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Where a settings value lives. Ordered by the documented precedence, highest
/// first: managed settings override everything, then project-local, then the
/// shared project file, then user settings.
///
/// (`--settings` sits between managed and project-local, but it is a CLI flag
/// with no file this app owns, so it has no variant here.)
#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum SettingsScope {
    Managed,
    Local,
    Project,
    User,
}

impl SettingsScope {
    /// Highest precedence first — the order `effective()` resolves in.
    pub const ALL: [SettingsScope; 4] = [
        SettingsScope::Managed,
        SettingsScope::Local,
        SettingsScope::Project,
        SettingsScope::User,
    ];

    /// Managed settings are deployed by an organization and must never be
    /// written by this app.
    pub fn writable(self) -> bool {
        !matches!(self, SettingsScope::Managed)
    }
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ScopeInfo {
    pub scope: SettingsScope,
    pub path: String,
    pub exists: bool,
    pub writable: bool,
    #[ts(type = "number | null")]
    pub mtime_ms: Option<i64>,
}

/// A settings file as raw text, for the JSON editor.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct RawDoc {
    pub scope: SettingsScope,
    pub path: String,
    pub exists: bool,
    #[ts(type = "number | null")]
    pub mtime_ms: Option<i64>,
    pub content: String,
}

/// One key of the merged view, with the file that supplied it.
///
/// A flat list rather than a merged object: it keeps ts-rs to a single opaque
/// field and makes "which file won" a column rather than a computation.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct EffectiveEntry {
    pub key: String,
    #[ts(type = "unknown")]
    pub value: serde_json::Value,
    pub source: SettingsScope,
    /// Scopes that also set this key but lost.
    pub overridden: Vec<SettingsScope>,
    /// True when the value is the union of several files' lists rather than
    /// one file's value — `permissions.allow` and friends merge.
    pub merged: bool,
}
