//! Read-side logic for session JSONL files. Phase 1: list_for_project +
//! paginated messages.
//!
//! Tolerant deserialization: CLI session JSONL schemas drift across
//! versions. Real lines use a top-level `type` ("user"|"assistant"|"summary"|
//! "attachment"|"file-history-snapshot"|"ai-title"|"last-prompt"…) with
//! `message.content` either a string or array of content blocks. Unknown
//! shapes are dropped to avoid polluting the chat view.

use std::path::Path;

use crate::types::{Message, MessageKind, Page, Role, SessionSummary};
use crate::AppError;

const PROJECTS_SUBDIR: &str = "projects";

pub fn list_for_project(claude_dir: &Path, project_name: &str) -> Result<Vec<SessionSummary>, AppError> {
    let dir = claude_dir.join(PROJECTS_SUBDIR).join(project_name);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)?.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        match summarize(&path, project_name) {
            Ok(s) => out.push(s),
            Err(e) => tracing::warn!(error = %e, path = %path.display(), "skipping unparseable session"),
        }
    }
    out.sort_by(|a, b| b.last_message_at.cmp(&a.last_message_at).then(a.session_id.cmp(&b.session_id)));
    Ok(out)
}

pub fn messages(
    claude_dir: &Path,
    project_name: &str,
    session_id: &str,
    after_index: Option<usize>,
    limit: Option<usize>,
) -> Result<Page<Message>, AppError> {
    let path = claude_dir
        .join(PROJECTS_SUBDIR)
        .join(project_name)
        .join(format!("{session_id}.jsonl"));
    if !path.is_file() {
        return Err(AppError::not_found(format!("session '{session_id}' not found")));
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut all = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        all.extend(parse_line(line));
    }
    let total = all.len();
    let start = after_index.unwrap_or(0);
    let limit = limit.unwrap_or(total);
    let end = (start + limit).min(total);
    let items: Vec<Message> = all.into_iter().skip(start).take(limit).collect();
    let next_after = if end < total { Some(end) } else { None };
    Ok(Page {
        items,
        next_after,
        total: Some(total),
    })
}

fn summarize(path: &Path, project_name: &str) -> Result<SessionSummary, AppError> {
    let session_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let raw = std::fs::read_to_string(path)?;
    let mut count = 0;
    let mut started_at: Option<String> = None;
    let mut last_message_at: Option<String> = None;
    let mut first_user_text: Option<String> = None;
    let mut summary_text: Option<String> = None;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let kind = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

        if kind == "summary" {
            if summary_text.is_none() {
                if let Some(s) = v.get("summary").and_then(|s| s.as_str()) {
                    if !s.is_empty() {
                        summary_text = Some(truncate(s, 120));
                    }
                }
            }
            continue;
        }
        if kind != "user" && kind != "assistant" {
            continue;
        }
        count += 1;
        if let Some(ts) = v.get("timestamp").and_then(|t| t.as_str()) {
            if started_at.is_none() {
                started_at = Some(ts.to_string());
            }
            last_message_at = Some(ts.to_string());
        }
        if first_user_text.is_none() && kind == "user" {
            first_user_text = extract_user_text(&v).map(|s| truncate(&s, 120));
        }
    }

    let preview = summary_text.or(first_user_text);
    let meta = std::fs::metadata(path)?;
    Ok(SessionSummary {
        session_id,
        project_name: project_name.to_string(),
        file_path: path.to_string_lossy().into_owned(),
        started_at,
        last_message_at,
        message_count: count,
        size_bytes: meta.len(),
        preview,
    })
}

/// Extract the user-visible text from a `type:"user"` JSONL record.
/// `message.content` is either a plain string or an array of content blocks;
/// for arrays we return the first text-bearing block.
fn extract_user_text(v: &serde_json::Value) -> Option<String> {
    let content = v.get("message")?.get("content")?;
    if let Some(s) = content.as_str() {
        return Some(s.to_string());
    }
    if let Some(arr) = content.as_array() {
        for block in arr {
            if let Some(s) = block.as_str() {
                return Some(s.to_string());
            }
            if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                return Some(t.to_string());
            }
        }
    }
    None
}

/// Parse a JSONL line into zero or more renderable messages.
///
/// Real CLI schema: top-level `type` says what the record is. For user/
/// assistant we extract content blocks (text/tool_use/tool_result/thinking)
/// and emit one Message per block. `summary` becomes a single text Message.
/// Bookkeeping types (attachment, file-history-snapshot, ai-title,
/// last-prompt) are dropped — they're noise in the chat view.
fn parse_line(line: &str) -> Vec<Message> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
        return vec![];
    };
    let record_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let uuid = v
        .get("uuid")
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();
    let timestamp = v
        .get("timestamp")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string());

    match record_type {
        "summary" => {
            let text = v
                .get("summary")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            if text.is_empty() {
                return vec![];
            }
            vec![Message {
                id: uuid,
                kind: MessageKind::Text,
                role: Some(Role::System),
                timestamp,
                content: Some(text),
                ..Default::default()
            }]
        }
        "user" | "assistant" => {
            let role = if record_type == "user" {
                Role::User
            } else {
                Role::Assistant
            };
            extract_message_blocks(&v, &uuid, role, timestamp)
        }
        _ => vec![],
    }
}

fn extract_message_blocks(
    v: &serde_json::Value,
    uuid: &str,
    role: Role,
    timestamp: Option<String>,
) -> Vec<Message> {
    let Some(content) = v.get("message").and_then(|m| m.get("content")) else {
        return vec![];
    };
    if let Some(s) = content.as_str() {
        if s.is_empty() {
            return vec![];
        }
        return vec![Message {
            id: uuid.to_string(),
            kind: MessageKind::Text,
            role: Some(role),
            timestamp,
            content: Some(s.to_string()),
            ..Default::default()
        }];
    }
    let Some(arr) = content.as_array() else {
        return vec![];
    };
    let mut out = Vec::new();
    for (i, block) in arr.iter().enumerate() {
        let id = format!("{uuid}#{i}");
        let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
        match block_type {
            "text" => {
                let text = block
                    .get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                if text.is_empty() {
                    continue;
                }
                out.push(Message {
                    id,
                    kind: MessageKind::Text,
                    role: Some(role),
                    timestamp: timestamp.clone(),
                    content: Some(text),
                    ..Default::default()
                });
            }
            "thinking" => {
                let text = block
                    .get("thinking")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                out.push(Message {
                    id,
                    kind: MessageKind::Thinking,
                    role: Some(role),
                    timestamp: timestamp.clone(),
                    thinking: Some(text),
                    ..Default::default()
                });
            }
            "tool_use" => {
                out.push(Message {
                    id,
                    kind: MessageKind::ToolUse,
                    role: Some(role),
                    timestamp: timestamp.clone(),
                    tool_name: block
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string()),
                    tool_input: block.get("input").cloned(),
                    ..Default::default()
                });
            }
            "tool_result" => {
                let is_error = block
                    .get("is_error")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(false);
                out.push(Message {
                    id,
                    kind: MessageKind::ToolResult,
                    role: Some(role),
                    timestamp: timestamp.clone(),
                    tool_result: block.get("content").cloned(),
                    is_error,
                    ..Default::default()
                });
            }
            "image" => {
                out.push(Message {
                    id,
                    kind: MessageKind::Image,
                    role: Some(role),
                    timestamp: timestamp.clone(),
                    ..Default::default()
                });
            }
            _ => {}
        }
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    let mut out: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_and_paginate() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path().join("projects/-tmp-x");
        std::fs::create_dir_all(&dir).unwrap();
        let lines: Vec<String> = (0..5)
            .map(|i| format!(
                r#"{{"type":"user","timestamp":"2026-01-0{}T00:00:00Z","message":{{"role":"user","content":"hi {i}"}}}}"#,
                i + 1,
            ))
            .collect();
        std::fs::write(dir.join("01h.jsonl"), lines.join("\n")).unwrap();

        let summaries = list_for_project(td.path(), "-tmp-x").unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].message_count, 5);
        assert_eq!(summaries[0].preview.as_deref(), Some("hi 0"));
        assert!(summaries[0].started_at.is_some());
        assert!(summaries[0].last_message_at.is_some());

        let page = messages(td.path(), "-tmp-x", "01h", Some(2), Some(2)).unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.next_after, Some(4));
    }

    #[test]
    fn summary_record_overrides_first_user_message() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path().join("projects/-tmp-s");
        std::fs::create_dir_all(&dir).unwrap();
        let body = r#"{"type":"summary","summary":"Refactor sessions list"}
{"type":"user","timestamp":"2026-01-01T00:00:00Z","message":{"role":"user","content":"long original prompt that should be hidden"}}
"#;
        std::fs::write(dir.join("01.jsonl"), body).unwrap();
        let summaries = list_for_project(td.path(), "-tmp-s").unwrap();
        assert_eq!(summaries[0].preview.as_deref(), Some("Refactor sessions list"));
    }

    #[test]
    fn tolerant_to_garbage_lines() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path().join("projects/-tmp-y");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("a.jsonl"),
            "not-json\n{}\n{\"type\":\"file-history-snapshot\"}\n",
        )
        .unwrap();
        let page = messages(td.path(), "-tmp-y", "a", None, None).unwrap();
        assert!(page.items.is_empty());
    }

    #[test]
    fn assistant_array_content_emits_blocks() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path().join("projects/-tmp-z");
        std::fs::create_dir_all(&dir).unwrap();
        let body = r#"{"type":"assistant","uuid":"u1","timestamp":"2026-01-01T00:00:00Z","message":{"role":"assistant","content":[{"type":"text","text":"hello"},{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"ls"}}]}}"#;
        std::fs::write(dir.join("b.jsonl"), body).unwrap();
        let page = messages(td.path(), "-tmp-z", "b", None, None).unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].kind, MessageKind::Text);
        assert_eq!(page.items[0].content.as_deref(), Some("hello"));
        assert_eq!(page.items[1].kind, MessageKind::ToolUse);
        assert_eq!(page.items[1].tool_name.as_deref(), Some("Bash"));
    }
}
