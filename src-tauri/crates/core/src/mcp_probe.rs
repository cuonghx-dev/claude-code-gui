//! Capability probe for MCP servers.
//!
//! Phase 5 implements a minimal JSON-RPC client over stdio that completes
//! the MCP `initialize` handshake and then issues `tools/list`,
//! `resources/list`, and `prompts/list`. Each request is wrapped in a 5s
//! deadline; partial results are merged into `McpCapabilities` and missing
//! sections are returned as empty arrays.
//!
//! HTTP+SSE servers are recognized but not probed (Phase 6 fills that in
//! once we have a streaming reqwest client) — they return an empty
//! capability list so the UI surfaces the "configured but not probed"
//! affordance without erroring.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

use crate::types::{McpCapabilities, McpPrompt, McpResource, McpScope, McpServer, McpTool, McpTransport};
use crate::{AppError, ErrorCode};

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PROTOCOL_VERSION: &str = "2024-11-05";

pub async fn probe(
    claude_dir: &Path,
    name: &str,
    scope: McpScope,
    working_dir: Option<&Path>,
) -> Result<McpCapabilities, AppError> {
    let server: McpServer = crate::mcp::get(claude_dir, name, scope, working_dir)?;
    match &server.transport {
        McpTransport::Stdio { command, args, env } => {
            timeout(PROBE_TIMEOUT, probe_stdio(command, args, env, working_dir))
                .await
                .map_err(|_| {
                    AppError::new(ErrorCode::Mcp, format!("probe '{name}' timed out (5s)"))
                })?
        }
        McpTransport::HttpSse { .. } => {
            tracing::info!(name, "http+sse mcp probe not yet implemented; returning empty");
            Ok(McpCapabilities::default())
        }
    }
}

async fn probe_stdio(
    command: &str,
    args: &[String],
    env: &std::collections::BTreeMap<String, String>,
    cwd: Option<&Path>,
) -> Result<McpCapabilities, AppError> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    if let Some(wd) = cwd {
        cmd.current_dir(wd);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::new(ErrorCode::Mcp, format!("spawn '{command}': {e}")))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| AppError::new(ErrorCode::Mcp, "no stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::new(ErrorCode::Mcp, "no stdout"))?;
    let mut reader = BufReader::new(stdout).lines();

    // 1. initialize
    let init = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "claude-code-gui", "version": "0.1.0" }
        }
    });
    write_msg(&mut stdin, &init).await?;
    let _init_resp = read_response(&mut reader, 1).await?;

    // 2. notifications/initialized (no response expected)
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    write_msg(&mut stdin, &initialized).await?;

    // 3. list each capability section. If a method is unsupported, the
    //    server returns an error — we treat that as empty.
    let tools = list_section::<RawTool>(&mut stdin, &mut reader, 2, "tools/list", "tools")
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|t| McpTool {
            name: t.name,
            description: t.description,
        })
        .collect();
    let resources =
        list_section::<RawResource>(&mut stdin, &mut reader, 3, "resources/list", "resources")
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| McpResource {
                uri: r.uri,
                name: r.name,
            })
            .collect();
    let prompts =
        list_section::<RawPrompt>(&mut stdin, &mut reader, 4, "prompts/list", "prompts")
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|p| McpPrompt {
                name: p.name,
                description: p.description,
            })
            .collect();

    let _ = child.kill().await;

    Ok(McpCapabilities {
        tools,
        resources,
        prompts,
    })
}

#[derive(serde::Deserialize)]
struct RawTool {
    name: String,
    #[serde(default)]
    description: Option<String>,
}
#[derive(serde::Deserialize)]
struct RawResource {
    uri: String,
    #[serde(default)]
    name: Option<String>,
}
#[derive(serde::Deserialize)]
struct RawPrompt {
    name: String,
    #[serde(default)]
    description: Option<String>,
}

async fn list_section<T: serde::de::DeserializeOwned>(
    stdin: &mut tokio::process::ChildStdin,
    reader: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    id: u32,
    method: &str,
    field: &str,
) -> Result<Vec<T>, AppError> {
    let req = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {}
    });
    write_msg(stdin, &req).await?;
    let resp = read_response(reader, id).await?;
    let result = resp
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let arr = match result.get(field).cloned() {
        Some(serde_json::Value::Array(items)) => items,
        _ => return Ok(vec![]),
    };
    Ok(arr
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect())
}

async fn write_msg(
    stdin: &mut tokio::process::ChildStdin,
    msg: &serde_json::Value,
) -> Result<(), AppError> {
    let line = serde_json::to_string(msg)?;
    stdin
        .write_all(line.as_bytes())
        .await
        .map_err(|e| AppError::new(ErrorCode::Mcp, format!("stdin write: {e}")))?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(|e| AppError::new(ErrorCode::Mcp, format!("stdin newline: {e}")))?;
    stdin
        .flush()
        .await
        .map_err(|e| AppError::new(ErrorCode::Mcp, format!("stdin flush: {e}")))?;
    Ok(())
}

async fn read_response(
    reader: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    expect_id: u32,
) -> Result<serde_json::Value, AppError> {
    while let Some(line) = reader
        .next_line()
        .await
        .map_err(|e| AppError::new(ErrorCode::Mcp, format!("stdout read: {e}")))?
    {
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue, // ignore non-JSON noise
        };
        // notifications have no id; skip them.
        match v.get("id").and_then(|i| i.as_u64()) {
            Some(id) if id as u32 == expect_id => return Ok(v),
            _ => continue,
        }
    }
    Err(AppError::new(
        ErrorCode::Mcp,
        format!("server closed before responding to id {expect_id}"),
    ))
}
