//! `claude -p` subprocess driver for non-conversational, one-shot use.
//!
//! Implements `improve_instructions`: spawns `claude -p
//! --output-format stream-json --input-format stream-json
//! --append-system-prompt <system>`, writes a single `user` message with the
//! agent's prompt to stdin, then reads stdout line-by-line and emits
//! `claude:improve:{request_id}` events with `kind: 'delta' | 'done' | 'error'`.
//!
//! The crate is tauri-free: callers pass an `Emit` callback (the Tauri app
//! wires `app.emit`). This keeps the crate testable without spinning up the
//! Tauri runtime.

use std::process::Stdio;
use std::sync::Arc;

use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use app_core::types::ImproveRequest;
use app_core::{AppError, ErrorCode, RequestId};

/// Callback supplied by the caller. Wired in main.rs to `app.emit(name, payload)`.
pub type Emit = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync + 'static>;

/// Run `claude -p` and stream deltas. Spawns its own tokio task; returns the
/// `RequestId` immediately so the IPC handler can hand it to the frontend.
pub fn improve_instructions(
    emit: Emit,
    request_id: RequestId,
    input: ImproveRequest,
) -> Result<(), AppError> {
    let claude = app_core::claude_cli::path()?;
    let event = format!("claude:improve:{request_id}");

    tokio::spawn(async move {
        if let Err(e) = run(&claude, &emit, &event, input).await {
            emit(
                &event,
                serde_json::json!({
                    "kind": "error",
                    "error": e.to_string(),
                }),
            );
        }
    });
    Ok(())
}

async fn run(
    claude: &std::path::Path,
    emit: &Emit,
    event: &str,
    input: ImproveRequest,
) -> Result<(), AppError> {
    let mut cmd = Command::new(claude);
    cmd.arg("-p")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--input-format")
        .arg("stream-json")
        .arg("--append-system-prompt")
        .arg(&input.system);
    if let Some(model) = &input.model {
        cmd.arg("--model").arg(model);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::new(ErrorCode::ClaudeCli, format!("spawn failed: {e}")))?;

    // Write a single user-message envelope. The CLI's stream-json input
    // shape is `{"type":"user","message":{"role":"user","content":"…"}}`.
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AppError::new(ErrorCode::ClaudeCli, "no stdin pipe"))?;
    {
        let mut stdin = stdin;
        let line = serde_json::json!({
            "type": "user",
            "message": { "role": "user", "content": input.prompt }
        })
        .to_string();
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| AppError::new(ErrorCode::ClaudeCli, format!("stdin write: {e}")))?;
        stdin.write_all(b"\n").await.map_err(|e| {
            AppError::new(ErrorCode::ClaudeCli, format!("stdin newline: {e}"))
        })?;
        // Drop closes stdin so the CLI knows the input stream is done.
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::new(ErrorCode::ClaudeCli, "no stdout pipe"))?;
    let mut lines = BufReader::new(stdout).lines();

    while let Some(line) = lines
        .next_line()
        .await
        .map_err(|e| AppError::new(ErrorCode::ClaudeCli, format!("read line: {e}")))?
    {
        if line.trim().is_empty() {
            continue;
        }
        // Tolerant parse: unknown shapes get skipped silently.
        let parsed: StreamEvent = match serde_json::from_str(&line) {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!(error = %e, line, "stream-json line ignored");
                continue;
            }
        };
        match parsed {
            StreamEvent::TextDelta { text } => {
                emit(event, serde_json::json!({ "kind": "delta", "text": text }));
            }
            StreamEvent::Result { .. } => {
                emit(event, serde_json::json!({ "kind": "done" }));
                break;
            }
            StreamEvent::Error { error } => {
                emit(event, serde_json::json!({ "kind": "error", "error": error }));
                break;
            }
            StreamEvent::Other => {}
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| AppError::new(ErrorCode::ClaudeCli, format!("wait: {e}")))?;
    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut s) = child.stderr.take() {
            use tokio::io::AsyncReadExt;
            let _ = s.read_to_string(&mut stderr).await;
        }
        emit(
            event,
            serde_json::json!({
                "kind": "error",
                "error": format!("claude -p exited {} ({})", status, stderr.trim()),
            }),
        );
    }
    Ok(())
}

/// Tolerant subset of stream-json events. We only care about text deltas
/// and the result/error sentinel; anything else (system, tool_use,
/// thinking) lands in `Other`.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum StreamEvent {
    /// `{"type":"text_delta","text":"…"}` — partial assistant text.
    TextDelta {
        #[serde(default)]
        text: String,
    },
    /// `{"type":"result", …}` — final marker.
    Result {
        #[serde(default)]
        #[allow(dead_code)]
        message: Option<serde_json::Value>,
    },
    /// `{"type":"error","error":"…"}` — emitted on CLI-side error.
    Error {
        #[serde(default)]
        error: String,
    },
    #[serde(other)]
    Other,
}
