use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum GroupBy {
    Day,
    Project,
    Model,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct UsageQuery {
    #[ts(type = "number | null")]
    pub from_ms: Option<i64>,
    #[ts(type = "number | null")]
    pub to_ms: Option<i64>,
    pub project: Option<String>,
    pub group_by: GroupBy,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct UsageTotals {
    #[ts(type = "number")]
    pub input: u64,
    #[ts(type = "number")]
    pub output: u64,
    #[ts(type = "number")]
    pub cache_read: u64,
    #[ts(type = "number")]
    pub cache_write: u64,
    #[ts(type = "number")]
    pub turns: u64,
    pub cost_usd: f64,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct UsageBucket {
    /// Sort key: an ISO date, project name, or model id.
    pub key: String,
    pub label: String,
    pub totals: UsageTotals,
    pub by_model: Vec<ModelTotals>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ModelTotals {
    pub model: String,
    pub totals: UsageTotals,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct UsageReport {
    pub total: UsageTotals,
    pub buckets: Vec<UsageBucket>,
    #[ts(type = "number")]
    pub scanned_files: usize,
    #[ts(type = "number | null")]
    pub indexed_at_ms: Option<i64>,
    /// Model ids whose family could not be identified, so their turns are
    /// counted but not priced. Surfaced so a total is never quietly low.
    pub unpriced_models: Vec<String>,
    #[ts(type = "number")]
    pub unpriced_turns: u64,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ActivityDay {
    /// `YYYY-MM-DD`.
    pub date: String,
    #[ts(type = "number")]
    pub prompts: u32,
    #[ts(type = "number")]
    pub projects: u32,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    #[ts(type = "number")]
    pub total_files: usize,
    #[ts(type = "number")]
    pub rescanned: usize,
    #[ts(type = "number")]
    pub skipped: usize,
    #[ts(type = "number")]
    pub duration_ms: u64,
}
