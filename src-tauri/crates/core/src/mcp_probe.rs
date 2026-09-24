//! Capability probe for MCP servers.
//!
//! Phase 5 implements a minimal JSON-RPC client over stdio that completes
//! the MCP `initialize` handshake and then issues `tools/list`,
//! `resources/list`, and `prompts/list`. Each request is wrapped in a 5s
//! deadline; partial results are merged into `McpCapabilities` and missing
//! sections are returned as empty arrays.
//!
//! HTTP servers go through `probe_http`: Streamable HTTP first (POST, reply
//! as JSON or an SSE stream), falling back to the legacy HTTP+SSE transport
//! (GET an event stream, POST to the `endpoint` it announces) when the POST
//! is rejected with 400/404/405, as the MCP spec's compatibility notes advise.

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
        McpTransport::HttpSse { url, headers } => timeout(PROBE_TIMEOUT, probe_http(url, headers))
            .await
            .map_err(|_| AppError::new(ErrorCode::Mcp, format!("probe '{name}' timed out (5s)")))?,
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
    let init_resp = read_response(&mut reader, 1).await?;
    // The initialize response is where a channel identifies itself; the probe
    // used to discard it.
    let (is_channel, relays_permissions, instructions) = parse_init_result(&init_resp);

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
        is_channel,
        relays_permissions,
        instructions,
    })
}

/// Turn the raw `*/list` envelopes into `McpCapabilities`. A missing or
/// errored section is empty rather than a failure.
fn assemble(
    init: &serde_json::Value,
    tools: Option<serde_json::Value>,
    resources: Option<serde_json::Value>,
    prompts: Option<serde_json::Value>,
) -> McpCapabilities {
    let (is_channel, relays_permissions, instructions) = parse_init_result(init);
    McpCapabilities {
        tools: section::<RawTool>(tools, "tools")
            .into_iter()
            .map(|t| McpTool { name: t.name, description: t.description })
            .collect(),
        resources: section::<RawResource>(resources, "resources")
            .into_iter()
            .map(|r| McpResource { uri: r.uri, name: r.name })
            .collect(),
        prompts: section::<RawPrompt>(prompts, "prompts")
            .into_iter()
            .map(|p| McpPrompt { name: p.name, description: p.description })
            .collect(),
        is_channel,
        relays_permissions,
        instructions,
    }
}

fn section<T: serde::de::DeserializeOwned>(envelope: Option<serde_json::Value>, field: &str) -> Vec<T> {
    match envelope.and_then(|e| e.get("result")?.get(field).cloned()) {
        Some(serde_json::Value::Array(items)) => items
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
        _ => vec![],
    }
}

fn rpc(id: u32, method: &str, params: serde_json::Value) -> serde_json::Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

fn init_params() -> serde_json::Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {},
        "clientInfo": { "name": "claude-code-gui", "version": env!("CARGO_PKG_VERSION") }
    })
}

fn mcp_err(msg: impl std::fmt::Display) -> AppError {
    AppError::new(ErrorCode::Mcp, msg.to_string())
}

async fn probe_http(
    url: &str,
    headers: &std::collections::BTreeMap<String, String>,
) -> Result<McpCapabilities, AppError> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| mcp_err(format!("http client: {e}")))?;
    let mut session = HttpSession {
        client,
        url: url.to_string(),
        headers: headers.clone(),
        session_id: None,
        negotiated: false,
    };

    let init = match session.request(&rpc(1, "initialize", init_params())).await {
        Ok(v) => v,
        Err(HttpFail::Rejected(400 | 404 | 405)) => {
            return probe_legacy_sse(session).await;
        }
        Err(e) => return Err(e.into()),
    };
    session.negotiated = true;
    session.notify("notifications/initialized").await;

    let tools = session.request(&rpc(2, "tools/list", json!({}))).await.ok();
    let resources = session.request(&rpc(3, "resources/list", json!({}))).await.ok();
    let prompts = session.request(&rpc(4, "prompts/list", json!({}))).await.ok();
    session.close().await;
    Ok(assemble(&init, tools, resources, prompts))
}

enum HttpFail {
    Rejected(u16),
    Other(AppError),
}

impl From<HttpFail> for AppError {
    fn from(f: HttpFail) -> Self {
        match f {
            // This app never holds MCP OAuth tokens; the CLI owns that flow.
            HttpFail::Rejected(status @ (401 | 403)) => mcp_err(format!(
                "server requires sign-in (HTTP {status}); authenticate it with /mcp in claude"
            )),
            HttpFail::Rejected(status) => mcp_err(format!("server answered HTTP {status}")),
            HttpFail::Other(e) => e,
        }
    }
}

impl From<AppError> for HttpFail {
    fn from(e: AppError) -> Self {
        HttpFail::Other(e)
    }
}

/// Streamable HTTP client state: one POST per JSON-RPC message.
struct HttpSession {
    client: reqwest::Client,
    url: String,
    headers: std::collections::BTreeMap<String, String>,
    session_id: Option<String>,
    negotiated: bool,
}

impl HttpSession {
    fn post(&self, body: &serde_json::Value) -> reqwest::RequestBuilder {
        let mut req = self
            .client
            .post(&self.url)
            .header("Accept", "application/json, text/event-stream")
            .json(body);
        for (k, v) in &self.headers {
            req = req.header(k, v);
        }
        if let Some(sid) = &self.session_id {
            req = req.header("Mcp-Session-Id", sid);
        }
        if self.negotiated {
            req = req.header("MCP-Protocol-Version", PROTOCOL_VERSION);
        }
        req
    }

    async fn request(&mut self, body: &serde_json::Value) -> Result<serde_json::Value, HttpFail> {
        let expect_id = body.get("id").and_then(|i| i.as_u64()).unwrap_or_default();
        let resp = self
            .post(body)
            .send()
            .await
            .map_err(|e| mcp_err(format!("POST {}: {e}", self.url)))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(HttpFail::Rejected(status.as_u16()));
        }
        if let Some(sid) = resp.headers().get("mcp-session-id").and_then(|v| v.to_str().ok()) {
            self.session_id = Some(sid.to_string());
        }
        let is_sse = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.starts_with("text/event-stream"));
        if is_sse {
            let mut events = SseReader::new(resp);
            while let Some(ev) = events.next().await? {
                if let Some(v) = response_with_id(&ev.data, expect_id) {
                    return Ok(v);
                }
            }
            Err(mcp_err(format!("stream ended before response to id {expect_id}")).into())
        } else {
            let text = resp.text().await.map_err(|e| mcp_err(format!("read body: {e}")))?;
            response_with_id(&text, expect_id)
                .ok_or_else(|| mcp_err(format!("no JSON-RPC response to id {expect_id}")).into())
        }
    }

    async fn notify(&self, method: &str) {
        let body = json!({ "jsonrpc": "2.0", "method": method });
        let _ = self.post(&body).send().await;
    }

    /// Best-effort session teardown so probes do not leak server sessions.
    async fn close(&self) {
        if let Some(sid) = &self.session_id {
            let mut req = self.client.delete(&self.url).header("Mcp-Session-Id", sid);
            for (k, v) in &self.headers {
                req = req.header(k, v);
            }
            let _ = req.send().await;
        }
    }
}

/// Find the response to `id` in a JSON-RPC message or batch.
fn response_with_id(raw: &str, id: u64) -> Option<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_str(raw.trim()).ok()?;
    let matches = |m: &serde_json::Value| m.get("id").and_then(|i| i.as_u64()) == Some(id);
    match v {
        serde_json::Value::Array(items) => items.into_iter().find(matches),
        m if matches(&m) => Some(m),
        _ => None,
    }
}

/// Legacy HTTP+SSE (MCP 2024-11-05): responses arrive on a GET event stream,
/// requests are POSTed to the URL the stream's first `endpoint` event names.
async fn probe_legacy_sse(session: HttpSession) -> Result<McpCapabilities, AppError> {
    let mut get = session.client.get(&session.url).header("Accept", "text/event-stream");
    for (k, v) in &session.headers {
        get = get.header(k, v);
    }
    let resp = get
        .send()
        .await
        .map_err(|e| mcp_err(format!("GET {}: {e}", session.url)))?;
    if !resp.status().is_success() {
        return Err(mcp_err(format!("server answered HTTP {} to GET", resp.status().as_u16())));
    }
    let base = resp.url().clone();
    let mut events = SseReader::new(resp);

    let endpoint = loop {
        match events.next().await.map_err(AppError::from)? {
            Some(ev) if ev.event == "endpoint" => {
                break base
                    .join(ev.data.trim())
                    .map_err(|e| mcp_err(format!("bad endpoint '{}': {e}", ev.data.trim())))?;
            }
            Some(_) => continue,
            None => return Err(mcp_err("event stream closed before announcing an endpoint")),
        }
    };

    let send = |body: serde_json::Value| {
        let mut req = session.client.post(endpoint.clone()).json(&body);
        for (k, v) in &session.headers {
            req = req.header(k, v);
        }
        req.send()
    };
    let mut call = async |id: u32, method: &str, params: serde_json::Value| -> Result<serde_json::Value, AppError> {
        send(rpc(id, method, params))
            .await
            .map_err(|e| mcp_err(format!("POST {endpoint}: {e}")))?;
        while let Some(ev) = events.next().await.map_err(AppError::from)? {
            if let Some(v) = response_with_id(&ev.data, id as u64) {
                return Ok(v);
            }
        }
        Err(mcp_err(format!("stream ended before response to id {id}")))
    };

    let init = call(1, "initialize", init_params()).await?;
    let _ = send(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).await;
    let tools = call(2, "tools/list", json!({})).await.ok();
    let resources = call(3, "resources/list", json!({})).await.ok();
    let prompts = call(4, "prompts/list", json!({})).await.ok();
    Ok(assemble(&init, tools, resources, prompts))
}

#[derive(Debug, Default)]
struct SseEvent {
    event: String,
    data: String,
}

/// Incremental `text/event-stream` parser over a reqwest body.
struct SseReader {
    resp: reqwest::Response,
    buf: String,
}

impl SseReader {
    fn new(resp: reqwest::Response) -> Self {
        Self { resp, buf: String::new() }
    }

    async fn next(&mut self) -> Result<Option<SseEvent>, HttpFail> {
        loop {
            if let Some(ev) = take_event(&mut self.buf) {
                return Ok(Some(ev));
            }
            match self.resp.chunk().await {
                Ok(Some(bytes)) => self.buf.push_str(&String::from_utf8_lossy(&bytes).replace("\r\n", "\n")),
                Ok(None) => return Ok(None),
                Err(e) => return Err(mcp_err(format!("event stream: {e}")).into()),
            }
        }
    }
}

/// Pop the first complete (blank-line terminated) event off `buf`.
fn take_event(buf: &mut String) -> Option<SseEvent> {
    loop {
        let end = buf.find("\n\n")?;
        let block: String = buf.drain(..end + 2).collect();
        let mut ev = SseEvent { event: "message".into(), data: String::new() };
        let mut has_data = false;
        for line in block.lines() {
            let (field, value) = line.split_once(':').unwrap_or((line, ""));
            let value = value.strip_prefix(' ').unwrap_or(value);
            match field {
                "event" => ev.event = value.to_string(),
                "data" => {
                    if has_data {
                        ev.data.push('\n');
                    }
                    ev.data.push_str(value);
                    has_data = true;
                }
                _ => {}
            }
        }
        if has_data || ev.event != "message" {
            return Some(ev);
        }
        // Comment/keep-alive only: keep scanning.
    }
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

/// Pull the channel declaration out of an `initialize` response envelope.
///
/// A channel is just an MCP server that declares
/// `capabilities.experimental['claude/channel']`; there is no channel config
/// file anywhere, so this response is the only place one can be recognized.
fn parse_init_result(envelope: &serde_json::Value) -> (bool, bool, Option<String>) {
    let Some(result) = envelope.get("result") else {
        return (false, false, None);
    };
    let experimental = result
        .get("capabilities")
        .and_then(|c| c.get("experimental"));
    // The docs treat an explicit `false` as opting out, anything else as
    // declared.
    let declared = |key: &str| {
        experimental
            .and_then(|e| e.get(key))
            .is_some_and(|v| v != &json!(false))
    };
    (
        declared("claude/channel"),
        declared("claude/channel/permission"),
        result
            .get("instructions")
            .and_then(|i| i.as_str())
            .map(str::to_string),
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(experimental: serde_json::Value) -> serde_json::Value {
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": { "tools": {}, "experimental": experimental },
                "instructions": "Forwards webhooks. Reply with send_message.",
                "serverInfo": { "name": "webhook", "version": "0.1.0" }
            }
        })
    }

    #[test]
    fn recognizes_a_channel_from_the_initialize_response() {
        let (is_channel, relays, instructions) =
            parse_init_result(&envelope(json!({ "claude/channel": {} })));
        assert!(is_channel);
        assert!(!relays);
        assert!(instructions.unwrap().starts_with("Forwards webhooks"));
    }

    #[test]
    fn permission_relay_is_declared_separately() {
        let (_, relays, _) = parse_init_result(&envelope(
            json!({ "claude/channel": {}, "claude/channel/permission": {} }),
        ));
        assert!(relays, "this channel can approve tool calls remotely");

        // An explicit `false` opts out.
        let (_, relays, _) = parse_init_result(&envelope(
            json!({ "claude/channel": {}, "claude/channel/permission": false }),
        ));
        assert!(!relays);
    }

    #[test]
    fn an_ordinary_mcp_server_is_not_a_channel() {
        let (is_channel, relays, _) = parse_init_result(&envelope(json!({})));
        assert!(!is_channel && !relays);

        // No experimental block at all, and a malformed envelope.
        let bare = json!({"jsonrpc":"2.0","id":1,"result":{"capabilities":{"tools":{}}}});
        assert_eq!(parse_init_result(&bare), (false, false, None));
        assert_eq!(parse_init_result(&json!({"error": {}})), (false, false, None));
    }

    #[test]
    fn sse_parser_handles_multiline_data_comments_and_split_chunks() {
        let mut buf = String::from(": keep-alive\n\nevent: endpoint\ndata: /m?s=1\n\ndata: {\"a\":\ndata: 1}\n\ndata: tail");
        let ev = take_event(&mut buf).unwrap();
        assert_eq!((ev.event.as_str(), ev.data.as_str()), ("endpoint", "/m?s=1"));
        let ev = take_event(&mut buf).unwrap();
        assert_eq!(ev.data, "{\"a\":\n1}");
        assert!(take_event(&mut buf).is_none(), "incomplete event must wait for more bytes");
        assert_eq!(buf, "data: tail");
    }

    #[test]
    fn finds_response_in_batches() {
        let raw = r#"[{"jsonrpc":"2.0","method":"note"},{"jsonrpc":"2.0","id":3,"result":{}}]"#;
        assert!(response_with_id(raw, 3).is_some());
        assert!(response_with_id(raw, 4).is_none());
    }

    mod http {
        use super::super::*;
        use std::sync::{Arc, Mutex};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::{TcpListener, TcpStream};
        use tokio::sync::mpsc;

        struct Req {
            method: String,
            path: String,
            headers: Vec<(String, String)>,
            body: String,
        }

        async fn read_req(sock: &mut TcpStream) -> Option<Req> {
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            let head_end = loop {
                let n = sock.read(&mut tmp).await.ok()?;
                if n == 0 {
                    return None;
                }
                buf.extend_from_slice(&tmp[..n]);
                if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break i;
                }
            };
            let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
            let mut lines = head.lines();
            let mut first = lines.next()?.split(' ');
            let (method, path) = (first.next()?.to_string(), first.next()?.to_string());
            let headers: Vec<(String, String)> = lines
                .filter_map(|l| l.split_once(':'))
                .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
                .collect();
            let len: usize = headers
                .iter()
                .find(|(k, _)| k == "content-length")
                .and_then(|(_, v)| v.parse().ok())
                .unwrap_or(0);
            let mut body = buf[head_end + 4..].to_vec();
            while body.len() < len {
                let n = sock.read(&mut tmp).await.ok()?;
                body.extend_from_slice(&tmp[..n]);
            }
            Some(Req { method, path, headers, body: String::from_utf8_lossy(&body).to_string() })
        }

        fn reply_for(body: &str) -> Option<serde_json::Value> {
            let v: serde_json::Value = serde_json::from_str(body).ok()?;
            let id = v.get("id")?.clone();
            let result = match v["method"].as_str()? {
                "initialize" => json!({
                    "protocolVersion": PROTOCOL_VERSION,
                    "capabilities": { "tools": {} },
                    "instructions": "hi",
                    "serverInfo": { "name": "t", "version": "1" }
                }),
                "tools/list" => json!({ "tools": [{ "name": "echo", "description": "Echo" }] }),
                "resources/list" => json!({ "resources": [{ "uri": "file:///a", "name": "a" }] }),
                _ => return Some(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "nope" } })),
            };
            Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
        }

        async fn write(sock: &mut TcpStream, status: &str, extra: &str, body: &str) {
            let msg = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n{body}",
                body.len()
            );
            let _ = sock.write_all(msg.as_bytes()).await;
        }

        /// Streamable HTTP server; `sse` picks the reply encoding.
        async fn streamable(sse: bool, seen: Arc<Mutex<Vec<String>>>) -> String {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::spawn(async move {
                while let Ok((mut sock, _)) = listener.accept().await {
                    let seen = seen.clone();
                    tokio::spawn(async move {
                        let Some(req) = read_req(&mut sock).await else { return };
                        let sid = req.headers.iter().find(|(k, _)| k == "mcp-session-id");
                        seen.lock().unwrap().push(format!(
                            "{} sid={}",
                            req.method,
                            sid.map(|(_, v)| v.as_str()).unwrap_or("-")
                        ));
                        match reply_for(&req.body) {
                            None => write(&mut sock, "202 Accepted", "", "").await,
                            Some(r) if sse => {
                                let body = format!(": ping\n\nevent: message\ndata: {r}\n\n");
                                write(&mut sock, "200 OK", "Content-Type: text/event-stream\r\nMcp-Session-Id: abc\r\n", &body).await
                            }
                            Some(r) => write(&mut sock, "200 OK", "Content-Type: application/json\r\nMcp-Session-Id: abc\r\n", &r.to_string()).await,
                        }
                    });
                }
            });
            format!("http://{addr}/mcp")
        }

        #[tokio::test]
        async fn probes_streamable_http_with_json_and_sse_replies() {
            for sse in [false, true] {
                let seen = Arc::new(Mutex::new(Vec::new()));
                let url = streamable(sse, seen.clone()).await;
                let caps = probe_http(&url, &Default::default()).await.unwrap();
                assert_eq!(caps.tools.len(), 1, "sse={sse}");
                assert_eq!(caps.tools[0].name, "echo");
                assert_eq!(caps.resources[0].uri, "file:///a");
                assert!(caps.prompts.is_empty(), "errored section is empty, not a failure");
                assert_eq!(caps.instructions.as_deref(), Some("hi"));
                let seen = seen.lock().unwrap();
                assert_eq!(seen[0], "POST sid=-");
                assert!(seen[1..].iter().all(|l| l.ends_with("sid=abc")), "{seen:?}");
                assert_eq!(seen.last().unwrap(), "DELETE sid=abc");
            }
        }

        #[tokio::test]
        async fn falls_back_to_legacy_sse_transport() {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let (tx, rx) = mpsc::unbounded_channel::<String>();
            let rx = Arc::new(tokio::sync::Mutex::new(Some(rx)));
            tokio::spawn(async move {
                while let Ok((mut sock, _)) = listener.accept().await {
                    let (tx, rx) = (tx.clone(), rx.clone());
                    tokio::spawn(async move {
                        let Some(req) = read_req(&mut sock).await else { return };
                        match (req.method.as_str(), req.path.as_str()) {
                            ("POST", "/sse") => write(&mut sock, "405 Method Not Allowed", "", "").await,
                            ("GET", "/sse") => {
                                let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\n\r\n";
                                let _ = sock.write_all(head.as_bytes()).await;
                                let _ = sock.write_all(b"event: endpoint\ndata: /messages?session=1\n\n").await;
                                let mut rx = rx.lock().await.take().unwrap();
                                while let Some(msg) = rx.recv().await {
                                    let _ = sock.write_all(format!("event: message\ndata: {msg}\n\n").as_bytes()).await;
                                }
                            }
                            ("POST", p) if p.starts_with("/messages?session=1") => {
                                if let Some(r) = reply_for(&req.body) {
                                    let _ = tx.send(r.to_string());
                                }
                                write(&mut sock, "202 Accepted", "", "").await;
                            }
                            _ => write(&mut sock, "404 Not Found", "", "").await,
                        }
                    });
                }
            });

            let caps = timeout(PROBE_TIMEOUT, probe_http(&format!("http://{addr}/sse"), &Default::default()))
                .await
                .expect("legacy probe hung")
                .unwrap();
            assert_eq!(caps.tools[0].name, "echo");
            assert_eq!(caps.resources.len(), 1);
        }
    }

}
