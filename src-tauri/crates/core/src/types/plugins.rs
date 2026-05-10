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
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct PluginDetail {
    #[serde(flatten)]
    pub plugin: Plugin,
    pub readme: Option<String>,
}
