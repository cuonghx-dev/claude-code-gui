//! Read-side logic for MCP servers. Phase 1: list + get from .mcp.json.

use std::path::Path;

use crate::types::{McpScope, McpServer, McpTransport};
use crate::AppError;

pub fn list(
    claude_dir: &Path,
    scope: McpScope,
    working_dir: Option<&Path>,
) -> Result<Vec<McpServer>, AppError> {
    let path = match scope {
        McpScope::Global => claude_dir.join(".mcp.json"),
        McpScope::Project => match working_dir {
            Some(wd) => wd.join(".mcp.json"),
            None => return Ok(vec![]),
        },
    };
    if !path.is_file() {
        return Ok(vec![]);
    }
    let raw = std::fs::read_to_string(&path)?;
    let parsed: McpFile = serde_json::from_str(&raw)?;
    Ok(parsed
        .mcp_servers
        .into_iter()
        .map(|(name, def)| McpServer {
            name,
            scope,
            transport: def.into_transport(),
            source_file: path.to_string_lossy().into_owned(),
        })
        .collect())
}

pub fn get(
    claude_dir: &Path,
    name: &str,
    scope: McpScope,
    working_dir: Option<&Path>,
) -> Result<McpServer, AppError> {
    list(claude_dir, scope, working_dir)?
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| AppError::not_found(format!("mcp server '{name}' not found")))
}

#[derive(serde::Deserialize)]
struct McpFile {
    #[serde(default, rename = "mcpServers")]
    mcp_servers: std::collections::BTreeMap<String, McpDef>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum McpDef {
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

impl McpDef {
    fn into_transport(self) -> McpTransport {
        match self {
            McpDef::Stdio { command, args, env } => McpTransport::Stdio { command, args, env },
            McpDef::HttpSse { url, headers } => McpTransport::HttpSse { url, headers },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stdio_and_http() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join(".mcp.json"),
            r#"{
  "mcpServers": {
    "stdio-server": { "command": "/usr/bin/foo", "args": ["a", "b"] },
    "http-server":  { "url": "https://example.com/sse" }
  }
}"#,
        )
        .unwrap();
        let mut servers = list(td.path(), McpScope::Global, None).unwrap();
        servers.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(servers.len(), 2);
        assert!(matches!(&servers[0].transport, McpTransport::HttpSse { url, .. } if url.starts_with("https")));
        assert!(matches!(&servers[1].transport, McpTransport::Stdio { command, .. } if command == "/usr/bin/foo"));
    }
}
