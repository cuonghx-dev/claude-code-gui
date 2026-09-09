use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The `statusLine` settings key.
#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct StatusLine {
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<u32>,
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Result of running the statusline command once, on request.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct StatusLinePreview {
    pub stdout: String,
    pub stderr: String,
    #[ts(type = "number | null")]
    pub exit_code: Option<i32>,
    #[ts(type = "number")]
    pub duration_ms: u64,
    pub timed_out: bool,
    pub truncated: bool,
}
