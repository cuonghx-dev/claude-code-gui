use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub slug: String,
    /// Skill directory under `~/.claude/skills/<slug>/`.
    pub directory: String,
    pub frontmatter: SkillFrontmatter,
    pub body: String,
    pub file_path: String,
    /// `Local` for `~/.claude/skills/`, `Plugin(<id>)` for plugin-bundled.
    pub source: SkillSource,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SkillSource {
    Local,
    Plugin { id: String },
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct SkillFrontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub context: Option<SkillContext>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(flatten)]
    #[ts(skip)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "lowercase")]
pub enum SkillContext {
    /// Loaded only when triggers match.
    When,
    /// Always loaded into the agent.
    Always,
}

/// Payload for skill create/update. Writes to
/// `<claude_dir>/skills/<slug>/SKILL.md`.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct SkillInput {
    pub slug: String,
    pub frontmatter: SkillFrontmatter,
    pub body: String,
}

/// Import a skill from external source. Phase 2: only `Local { path }`
/// (a directory containing SKILL.md) is supported. `Github { url }` lands
/// in Phase 3.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SkillImportSource {
    Local { path: String },
    Github { url: String },
}
