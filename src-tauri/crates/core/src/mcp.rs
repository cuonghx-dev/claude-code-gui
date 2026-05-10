//! Read+write logic for MCP servers. Mutates the per-scope `.mcp.json`
//! tolerantly: unknown top-level keys are preserved, server entries that
//! we wrote are emitted in canonical (camelCase) form.

use std::path::{Path, PathBuf};

use crate::io;
use crate::types::{McpImportPayload, McpScope, McpServer, McpServerInput, McpTransport};
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

pub fn create(
    claude_dir: &Path,
    input: McpServerInput,
    scope: McpScope,
    working_dir: Option<&Path>,
) -> Result<McpServer, AppError> {
    if input.name.is_empty() {
        return Err(AppError::invalid("mcp server name cannot be empty"));
    }
    let path = file_for(claude_dir, scope, working_dir)?;
    let mut doc = read_doc(&path)?;
    if doc.servers.contains_key(&input.name) {
        return Err(AppError::invalid(format!(
            "mcp server '{}' already exists",
            input.name
        )));
    }
    doc.servers
        .insert(input.name.clone(), transport_to_def(&input.transport));
    write_doc(&path, &doc)?;
    Ok(McpServer {
        name: input.name,
        scope,
        transport: input.transport,
        source_file: path.to_string_lossy().into_owned(),
    })
}

pub fn delete(
    claude_dir: &Path,
    name: &str,
    scope: McpScope,
    working_dir: Option<&Path>,
) -> Result<(), AppError> {
    let path = file_for(claude_dir, scope, working_dir)?;
    let mut doc = read_doc(&path)?;
    if doc.servers.remove(name).is_none() {
        return Err(AppError::not_found(format!(
            "mcp server '{name}' not found"
        )));
    }
    write_doc(&path, &doc)?;
    Ok(())
}

pub fn import(claude_dir: &Path, payload: McpImportPayload) -> Result<Vec<McpServer>, AppError> {
    let wd = payload.working_dir.as_deref().map(Path::new);
    let path = file_for(claude_dir, payload.scope, wd)?;
    let mut doc = if payload.replace {
        Doc::default()
    } else {
        read_doc(&path)?
    };
    for srv in &payload.servers {
        if srv.name.is_empty() {
            return Err(AppError::invalid("mcp server name cannot be empty"));
        }
        doc.servers
            .insert(srv.name.clone(), transport_to_def(&srv.transport));
    }
    write_doc(&path, &doc)?;
    list(claude_dir, payload.scope, wd)
}

fn file_for(
    claude_dir: &Path,
    scope: McpScope,
    working_dir: Option<&Path>,
) -> Result<PathBuf, AppError> {
    Ok(match scope {
        McpScope::Global => claude_dir.join(".mcp.json"),
        McpScope::Project => working_dir
            .ok_or_else(|| AppError::invalid("project-scoped mcp requires workingDir"))?
            .join(".mcp.json"),
    })
}

#[derive(Default)]
struct Doc {
    servers: std::collections::BTreeMap<String, McpDef>,
    extras: std::collections::BTreeMap<String, serde_json::Value>,
}

fn read_doc(path: &Path) -> Result<Doc, AppError> {
    if !path.is_file() {
        return Ok(Doc::default());
    }
    let raw = std::fs::read_to_string(path)?;
    let mut value: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&raw).unwrap_or_default();
    let servers_raw = value.remove("mcpServers").unwrap_or(serde_json::Value::Null);
    let servers: std::collections::BTreeMap<String, McpDef> = match servers_raw {
        serde_json::Value::Null => std::collections::BTreeMap::new(),
        other => serde_json::from_value(other)?,
    };
    Ok(Doc {
        servers,
        extras: value.into_iter().collect(),
    })
}

fn write_doc(path: &Path, doc: &Doc) -> Result<(), AppError> {
    let mut out = serde_json::Map::new();
    for (k, v) in &doc.extras {
        out.insert(k.clone(), v.clone());
    }
    out.insert(
        "mcpServers".into(),
        serde_json::to_value(&doc.servers)?,
    );
    let serialized = serde_json::to_string_pretty(&out)?;
    io::atomic_write(path, serialized.as_bytes())
}

fn transport_to_def(t: &McpTransport) -> McpDef {
    match t {
        McpTransport::Stdio { command, args, env } => McpDef::Stdio {
            command: command.clone(),
            args: args.clone(),
            env: env.clone(),
        },
        McpTransport::HttpSse { url, headers } => McpDef::HttpSse {
            url: url.clone(),
            headers: headers.clone(),
        },
    }
}

#[derive(serde::Deserialize)]
struct McpFile {
    #[serde(default, rename = "mcpServers")]
    mcp_servers: std::collections::BTreeMap<String, McpDef>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
enum McpDef {
    Stdio {
        command: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
        env: std::collections::BTreeMap<String, String>,
    },
    HttpSse {
        url: String,
        #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
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

    #[test]
    fn create_then_delete_global() {
        let td = tempfile::tempdir().unwrap();
        create(
            td.path(),
            McpServerInput {
                name: "alpha".into(),
                transport: McpTransport::Stdio {
                    command: "echo".into(),
                    args: vec!["hi".into()],
                    env: Default::default(),
                },
            },
            McpScope::Global,
            None,
        )
        .unwrap();
        let servers = list(td.path(), McpScope::Global, None).unwrap();
        assert_eq!(servers.len(), 1);
        delete(td.path(), "alpha", McpScope::Global, None).unwrap();
        assert!(list(td.path(), McpScope::Global, None).unwrap().is_empty());
    }

    #[test]
    fn import_replace_overwrites() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join(".mcp.json"),
            r#"{"mcpServers":{"keep":{"command":"x"}}}"#,
        )
        .unwrap();
        let imported = import(
            td.path(),
            McpImportPayload {
                scope: McpScope::Global,
                working_dir: None,
                servers: vec![McpServerInput {
                    name: "fresh".into(),
                    transport: McpTransport::HttpSse {
                        url: "https://x".into(),
                        headers: Default::default(),
                    },
                }],
                replace: true,
            },
        )
        .unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].name, "fresh");
    }
}
