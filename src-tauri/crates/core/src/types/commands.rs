use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub slug: String,
    pub filename: String,
    pub directory: String,
    pub frontmatter: CommandFrontmatter,
    pub body: String,
    pub file_path: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct CommandFrontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// `alias` lets us read Claude CLI's kebab-case YAML keys.
    /// Phase 2 write-back emits camelCase per the IPC wire format.
    #[serde(default, alias = "argument-hint")]
    pub argument_hint: Option<String>,
    #[serde(default, alias = "allowed-tools")]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}
