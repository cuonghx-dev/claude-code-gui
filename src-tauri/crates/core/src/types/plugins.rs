use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub dir: String,
    /// Skill slugs the plugin contributes.
    pub skills: Vec<String>,
    /// Marketplace name (the `<mkt>` half of `<plugin>@<mkt>`).
    pub marketplace: Option<String>,
    /// ISO-8601 timestamp from the registry entry (`lastUpdated` falling back
    /// to `installedAt`). `None` if neither is present.
    pub installed_at: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct PluginDetail {
    #[serde(flatten)]
    pub plugin: Plugin,
    pub readme: Option<String>,
}

/// Configured marketplace source from `~/.claude/.marketplaces.json`.
/// Phase 3 supports `Github` (git clone) and `Http` (JSON manifest fetch).
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSource {
    pub name: String,
    #[serde(rename = "sourceType")]
    pub source_type: String, // "github" | "http"
    pub url: String,
    /// Cached plugin manifest fetched on `marketplace_sources_update`.
    #[serde(default)]
    pub plugins: Vec<AvailablePlugin>,
    #[serde(default)]
    pub last_updated: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct MarketplaceSourceInput {
    pub name: String,
    pub source_type: String, // "github" | "http"
    pub url: String,
}

/// A plugin discoverable from a marketplace source — not yet installed.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct AvailablePlugin {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    /// Source identifier (`<source-name>`) — frontend uses this when calling
    /// `marketplace_install` so the backend knows which source to fetch from.
    pub source: String,
    /// Direct git/http URL to fetch the plugin from.
    pub install_url: String,
}
