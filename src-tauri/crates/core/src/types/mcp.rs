use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub name: String,
    pub scope: McpScope,
    pub transport: McpTransport,
    /// Source file path: `~/.claude/.mcp.json` or `<project>/.mcp.json`.
    pub source_file: String,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy, PartialEq, Eq)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub enum McpScope {
    Global,
    Project,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum McpTransport {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: std::collections::BTreeMap<String, String>,
    },
    HttpSse {
        url: String,
        #[serde(default)]
        headers: std::collections::BTreeMap<String, String>,
    },
}

/// Capability probe result.
#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct McpCapabilities {
    pub tools: Vec<McpTool>,
    pub resources: Vec<McpResource>,
    pub prompts: Vec<McpPrompt>,
    /// The server declared `experimental['claude/channel']`, so Claude Code
    /// registers a notification listener and it can push events into a
    /// session. Channels have no configuration of their own — they are MCP
    /// servers, and this is the only thing that marks one.
    pub is_channel: bool,
    /// `experimental['claude/channel/permission']` — the channel can receive
    /// relayed tool-approval prompts, so approving a tool call can happen
    /// outside this machine. Worth showing prominently.
    pub relays_permissions: bool,
    /// The server's own `instructions` string, delivered to Claude as context
    /// when it connects.
    pub instructions: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct McpResource {
    pub uri: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
}

/// Create payload for an MCP server.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct McpServerInput {
    pub name: String,
    pub transport: McpTransport,
}

/// Bulk import: replace or merge servers in `.mcp.json` for the given scope.
#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct McpImportPayload {
    pub scope: McpScope,
    #[serde(default)]
    pub working_dir: Option<String>,
    pub servers: Vec<McpServerInput>,
    /// `true` replaces the entire file; `false` merges (later wins).
    #[serde(default)]
    pub replace: bool,
}
