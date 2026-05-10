use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct OutputStyle {
    pub id: String,
    pub scope: OutputStyleScope,
    pub frontmatter: OutputStyleFrontmatter,
    pub body: String,
    pub file_path: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "lowercase")]
pub enum OutputStyleScope {
    Global,
    Project,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct OutputStyleFrontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub keep_coding_instructions: Option<bool>,
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}
