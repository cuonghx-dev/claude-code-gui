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
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub skills: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
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

/// Accept either a YAML sequence (`[a, b]`) or a comma-separated string
/// (`"a, b"`). Claude Code's published agent templates use the CSV form for
/// `tools`; the GitHub-rendered docs likewise show CSV. We canonicalize to
/// `Vec<String>` on read.
fn deserialize_string_or_vec<'de, D>(d: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, SeqAccess, Visitor};
    use std::fmt;

    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a sequence of strings or a comma-separated string")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }
        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            self.visit_str(&v)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                out.push(s);
            }
            Ok(out)
        }
        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(Vec::new())
        }
    }
    d.deserialize_any(V)
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
