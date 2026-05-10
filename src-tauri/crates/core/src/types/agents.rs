use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Markdown agent file under `~/.claude/agents/<dir>/<slug>.md`.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub slug: String,
    pub filename: String,
    /// Subdirectory under `~/.claude/agents/`. Empty string = top level.
    pub directory: String,
    pub frontmatter: AgentFrontmatter,
    pub body: String,
    pub has_memory: bool,
    pub file_path: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct AgentFrontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub model: Option<AgentModel>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub memory: Option<AgentMemory>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    /// Catch-all for unknown frontmatter keys so write-back preserves them
    /// (server-side only; intentionally invisible to the frontend).
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "lowercase")]
pub enum AgentModel {
    Opus,
    Sonnet,
    Haiku,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "lowercase")]
pub enum AgentMemory {
    User,
    Project,
    Local,
    None,
}

/// Payload for create/update. Slug + directory determine the file path on
/// disk: `<claude_dir>/agents/<directory>/<slug>.md` (directory empty = root).
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct AgentInput {
    pub slug: String,
    #[serde(default)]
    pub directory: String,
    pub frontmatter: AgentFrontmatter,
    pub body: String,
}

/// Import payload: serialized agent file content as a string.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct AgentImport {
    pub slug: String,
    #[serde(default)]
    pub directory: String,
    /// Raw markdown source (frontmatter + body) to write verbatim.
    pub content: String,
}
