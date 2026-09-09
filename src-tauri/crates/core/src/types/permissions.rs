use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::SettingsScope;

/// The `permissions` block of a settings file.
///
/// `default_mode` stays a `String` rather than an enum: the documented set
/// already includes `auto` and `dontAsk`, and it keeps growing. An unknown
/// value should render with a warning, not fail to parse.
#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ask: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_directories: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_bypass_permissions_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_auto_mode: Option<String>,
    /// Anything this app does not model yet — including the sandbox block —
    /// round-trips untouched.
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct PermissionRule {
    pub raw: String,
    pub tool: String,
    pub specifier: Option<String>,
    pub kind: RuleKind,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum RuleKind {
    /// Tool name with no specifier — matches every call.
    Bare,
    Bash,
    PathLike,
    Domain,
    Mcp,
    Other,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct RuleIssue {
    pub rule: String,
    pub severity: Severity,
    pub message: String,
}

/// A rule plus the settings file it came from.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ScopedRule {
    pub rule: String,
    pub scope: SettingsScope,
}

/// Every rule in force, unioned across scopes — permission lists merge rather
/// than override.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct EffectivePermissions {
    pub allow: Vec<ScopedRule>,
    pub ask: Vec<ScopedRule>,
    pub deny: Vec<ScopedRule>,
    pub additional_directories: Vec<ScopedRule>,
    /// A scalar, so it follows precedence instead of merging.
    pub default_mode: Option<String>,
    pub default_mode_source: Option<SettingsScope>,
}
