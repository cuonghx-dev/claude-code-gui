//! Read-side logic for session JSONL files: list, paginated messages, and
//! subagent thread rollups.
//!
//! Tolerant deserialization: CLI session JSONL schemas drift across versions.
//! Real lines use a top-level `type` (`user`, `assistant`, `summary`,
//! `system`, `mode`, `permission-mode`, `attachment`, `file-history-*`,
//! `ai-title`, `last-prompt`, `atis-latch`…) with `message.content` either a
//! string or an array of content blocks.
//!
//! Records that are not part of the conversation are dropped from the message
//! stream: `attachment`, `ai-title`, `last-prompt`, `atis-latch`, and the
//! `file-history-*` pair (the checkpoints feature reads those directly).
//!
//! Pagination goes through `transcript_scan`'s byte index rather than reading
//! the whole file per page — transcripts reach tens of MB.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::transcript_scan::{self, IndexCache};
use crate::types::{Message, MessageKind, Page, Role, SessionSummary, Thread, TokenUsage};
use crate::AppError;

const PROJECTS_SUBDIR: &str = "projects";

/// Tool results above this size are truncated before crossing the IPC
/// boundary. Bash and Read output runs to hundreds of KB and routinely
/// contains secrets from files the agent read.
const TOOL_RESULT_CAP: usize = 16 * 1024;

pub fn list_for_project(
    claude_dir: &Path,
    project_name: &str,
) -> Result<Vec<SessionSummary>, AppError> {
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
            Err(e) => {
                tracing::warn!(error = %e, path = %path.display(), "skipping unparseable session")
            }
        }
    }
    out.sort_by(|a, b| {
        b.last_message_at
            .cmp(&a.last_message_at)
            .then(a.session_id.cmp(&b.session_id))
    });
    Ok(out)
}

/// One page of messages, located through the cached byte index.
///
/// The index maps message offsets to the line that starts them, so a page deep
/// into a large transcript seeks straight there instead of re-parsing
/// everything before it.
pub fn messages(
    claude_dir: &Path,
    cache: &IndexCache,
    project_name: &str,
    session_id: &str,
    after_index: Option<usize>,
    limit: Option<usize>,
) -> Result<Page<Message>, AppError> {
    let path = session_path(claude_dir, project_name, session_id)?;
    let index = cache.get_or_build(&path, |line| parse_line(line).len())?;
    let total = index.total_messages;
    let start = after_index.unwrap_or(0);
    let limit = limit.unwrap_or(total);

    if start >= total || limit == 0 {
        return Ok(Page {
            items: vec![],
            next_after: None,
            total: Some(total),
        });
    }

    let Some((byte_offset, skip_within_line)) = index.locate(start) else {
        return Ok(Page {
            items: vec![],
            next_after: None,
            total: Some(total),
        });
    };

    // Read forward a line at a time until the page is full. A line yields at
    // most a handful of messages, so the overshoot is bounded.
    let mut items: Vec<Message> = Vec::with_capacity(limit.min(512));
    let mut offset = byte_offset;
    let mut skip = skip_within_line;
    loop {
        let lines = transcript_scan::read_lines_from(&path, offset, 64)?;
        if lines.is_empty() {
            break;
        }
        let mut consumed_bytes = 0u64;
        for line in &lines {
            consumed_bytes += line.len() as u64 + 1; // + newline
            let mut parsed = parse_line(line);
            if skip > 0 {
                let drop = skip.min(parsed.len());
                parsed.drain(..drop);
                skip -= drop;
            }
            for m in parsed {
                if items.len() == limit {
                    break;
                }
                items.push(m);
            }
            if items.len() == limit {
                break;
            }
        }
        if items.len() == limit {
            break;
        }
        offset += consumed_bytes;
    }

    let end = start + items.len();
    Ok(Page {
        items,
        next_after: if end < total { Some(end) } else { None },
        total: Some(total),
    })
}

/// Roll up the subagent (Task tool) conversations in a session.
///
/// Sidechain records carry `isSidechain: true` and chain back through
/// `parentUuid`. Grouping happens here rather than in the frontend so the main
/// transcript can render one collapsed card per subagent without walking the
/// parent chain in JavaScript.
pub fn threads(
    claude_dir: &Path,
    project_name: &str,
    session_id: &str,
) -> Result<Vec<Thread>, AppError> {
    let path = session_path(claude_dir, project_name, session_id)?;

    // uuid -> parent uuid, for every sidechain record.
    let mut parent_of: HashMap<String, Option<String>> = HashMap::new();
    // Accumulated per record, keyed by uuid; resolved to roots in a second pass.
    struct Rec {
        timestamp: Option<String>,
        usage: TokenUsage,
        cost: Option<f64>,
        agent_name: Option<String>,
    }
    let mut recs: Vec<(String, Rec)> = Vec::new();
    // tool_use.id -> the Task call's own uuid, so a thread can name its caller.
    let mut task_calls: HashMap<String, String> = HashMap::new();

    transcript_scan::for_each_line(&path, |_, line| {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            return true;
        };
        let uuid = str_field(&v, "uuid").unwrap_or_default();

        // Remember Task tool calls from the *main* thread: their tool_use id is
        // what a sidechain hangs off.
        if !v.get("isSidechain").and_then(|s| s.as_bool()).unwrap_or(false) {
            if let Some(blocks) = v.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_array()) {
                for b in blocks {
                    if b.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                        && b.get("name").and_then(|n| n.as_str()) == Some("Task")
                    {
                        if let Some(id) = str_field(b, "id") {
                            let agent = b
                                .get("input")
                                .and_then(|i| i.get("subagent_type").or_else(|| i.get("description")))
                                .and_then(|x| x.as_str())
                                .map(str::to_string);
                            task_calls.insert(id, agent.unwrap_or_default());
                        }
                    }
                }
            }
            return true;
        }

        if uuid.is_empty() {
            return true;
        }
        parent_of.insert(uuid.clone(), str_field(&v, "parentUuid"));

        let (usage, model) = usage_of(&v);
        recs.push((
            uuid,
            Rec {
                timestamp: str_field(&v, "timestamp"),
                cost: model.as_deref().and_then(|m| {
                    crate::models::cost_usd(
                        m,
                        usage.input,
                        usage.output,
                        usage.cache_read,
                        usage.cache_write,
                    )
                }),
                usage,
                agent_name: None,
            },
        ));
        true
    })?;

    // Walk each record up to the topmost sidechain ancestor.
    let root_of = |uuid: &str| -> String {
        let mut cur = uuid.to_string();
        let mut guard = 0;
        while let Some(Some(parent)) = parent_of.get(&cur) {
            // Stop at the first parent outside the sidechain: that is the main
            // thread, and `cur` is the thread root.
            if !parent_of.contains_key(parent) {
                break;
            }
            cur = parent.clone();
            guard += 1;
            if guard > 10_000 {
                break; // cycles in a corrupt transcript must not hang the read
            }
        }
        cur
    };

    let mut grouped: HashMap<String, Thread> = HashMap::new();
    for (uuid, rec) in &recs {
        let root = root_of(uuid);
        let t = grouped.entry(root.clone()).or_insert_with(|| Thread {
            root_uuid: root.clone(),
            parent_tool_use_id: None,
            agent_name: rec.agent_name.clone(),
            message_count: 0,
            started_at: None,
            ended_at: None,
            usage: TokenUsage::default(),
            cost_usd: None,
        });
        t.message_count += 1;
        t.usage.input += rec.usage.input;
        t.usage.output += rec.usage.output;
        t.usage.cache_read += rec.usage.cache_read;
        t.usage.cache_write += rec.usage.cache_write;
        if let Some(c) = rec.cost {
            t.cost_usd = Some(t.cost_usd.unwrap_or(0.0) + c);
        }
        if let Some(ts) = &rec.timestamp {
            if t.started_at.as_ref().is_none_or(|s| ts < s) {
                t.started_at = Some(ts.clone());
            }
            if t.ended_at.as_ref().is_none_or(|e| ts > e) {
                t.ended_at = Some(ts.clone());
            }
        }
    }

    // A thread's root points at the Task call that spawned it.
    for t in grouped.values_mut() {
        if let Some(Some(parent)) = parent_of.get(&t.root_uuid) {
            if let Some(agent) = task_calls.get(parent) {
                t.parent_tool_use_id = Some(parent.clone());
                if !agent.is_empty() {
                    t.agent_name = Some(agent.clone());
                }
            }
        }
    }

    let mut out: Vec<Thread> = grouped.into_values().collect();
    out.sort_by(|a, b| a.started_at.cmp(&b.started_at).then(a.root_uuid.cmp(&b.root_uuid)));
    Ok(out)
}

fn session_path(
    claude_dir: &Path,
    project_name: &str,
    session_id: &str,
) -> Result<PathBuf, AppError> {
    let path = claude_dir
        .join(PROJECTS_SUBDIR)
        .join(project_name)
        .join(format!("{session_id}.jsonl"));
    if !path.is_file() {
        return Err(AppError::not_found(format!(
            "session '{session_id}' not found"
        )));
    }
    Ok(path)
}

fn summarize(path: &Path, project_name: &str) -> Result<SessionSummary, AppError> {
    let session_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let mut count = 0;
    let mut started_at: Option<String> = None;
    let mut last_message_at: Option<String> = None;
    let mut first_user_text: Option<String> = None;
    let mut summary_text: Option<String> = None;

    transcript_scan::for_each_line(path, |_, line| {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            return true;
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
            return true;
        }
        if kind != "user" && kind != "assistant" {
            return true;
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
        true
    })?;

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

/// Token counts and model id from an assistant record.
fn usage_of(v: &serde_json::Value) -> (TokenUsage, Option<String>) {
    let Some(msg) = v.get("message") else {
        return (TokenUsage::default(), None);
    };
    let model = msg.get("model").and_then(|m| m.as_str()).map(str::to_string);
    let Some(u) = msg.get("usage") else {
        return (TokenUsage::default(), model);
    };
    let n = |key: &str| u.get(key).and_then(|x| x.as_u64()).unwrap_or(0);
    (
        TokenUsage {
            input: n("input_tokens"),
            output: n("output_tokens"),
            cache_read: n("cache_read_input_tokens"),
            cache_write: n("cache_creation_input_tokens"),
        },
        model,
    )
}

/// Parse a JSONL line into zero or more renderable messages.
fn parse_line(line: &str) -> Vec<Message> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
        return vec![];
    };
    let record_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let uuid = str_field(&v, "uuid").unwrap_or_default();
    let timestamp = str_field(&v, "timestamp");

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
                record_type: Some(record_type.to_string()),
                is_turn_head: true,
                ..Default::default()
            }]
        }
        "user" | "assistant" => {
            let role = if record_type == "user" {
                Role::User
            } else {
                Role::Assistant
            };
            extract_message_blocks(&v, &uuid, role, timestamp, record_type)
        }
        // Session bookkeeping the CLI writes inline. Small, and useful for
        // explaining why the agent's behavior changed mid-session.
        "mode" | "permission-mode" => {
            let text = str_field(&v, "mode")
                .or_else(|| str_field(&v, "permissionMode"))
                .map(|m| format!("{record_type}: {m}"))
                .unwrap_or_else(|| record_type.to_string());
            vec![Message {
                id: uuid,
                kind: MessageKind::Meta,
                role: Some(Role::System),
                timestamp,
                content: Some(text),
                record_type: Some(record_type.to_string()),
                ..Default::default()
            }]
        }
        "system" => {
            let text = str_field(&v, "content").or_else(|| str_field(&v, "text"));
            match text {
                Some(t) if !t.is_empty() => vec![Message {
                    id: uuid,
                    kind: MessageKind::System,
                    role: Some(Role::System),
                    timestamp,
                    content: Some(t),
                    record_type: Some(record_type.to_string()),
                    ..Default::default()
                }],
                _ => vec![],
            }
        }
        _ => vec![],
    }
}

fn extract_message_blocks(
    v: &serde_json::Value,
    uuid: &str,
    role: Role,
    timestamp: Option<String>,
    record_type: &str,
) -> Vec<Message> {
    let Some(content) = v.get("message").and_then(|m| m.get("content")) else {
        return vec![];
    };

    let parent_uuid = str_field(v, "parentUuid");
    let is_sidechain = v
        .get("isSidechain")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);
    let git_branch = str_field(v, "gitBranch");
    let (usage, model) = usage_of(v);
    let has_usage = usage.input + usage.output + usage.cache_read + usage.cache_write > 0;
    let cost = model.as_deref().and_then(|m| {
        crate::models::cost_usd(m, usage.input, usage.output, usage.cache_read, usage.cache_write)
    });

    let base = |id: String, kind: MessageKind| Message {
        id,
        kind,
        role: Some(role),
        timestamp: timestamp.clone(),
        parent_uuid: parent_uuid.clone(),
        is_sidechain,
        git_branch: git_branch.clone(),
        record_type: Some(record_type.to_string()),
        ..Default::default()
    };

    if let Some(s) = content.as_str() {
        if s.is_empty() {
            return vec![];
        }
        let mut m = base(uuid.to_string(), MessageKind::Text);
        m.content = Some(s.to_string());
        attach_turn_meta(&mut m, has_usage, usage, model, cost);
        return vec![m];
    }
    let Some(arr) = content.as_array() else {
        return vec![];
    };

    let mut out: Vec<Message> = Vec::new();
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
                let mut m = base(id, MessageKind::Text);
                m.content = Some(text);
                out.push(m);
            }
            "thinking" => {
                let mut m = base(id, MessageKind::Thinking);
                m.thinking = block
                    .get("thinking")
                    .and_then(|t| t.as_str())
                    .map(str::to_string);
                out.push(m);
            }
            "tool_use" => {
                let mut m = base(id, MessageKind::ToolUse);
                m.tool_name = block.get("name").and_then(|n| n.as_str()).map(str::to_string);
                m.tool_input = block.get("input").cloned();
                m.tool_use_id = str_field(block, "id");
                out.push(m);
            }
            "tool_result" => {
                let mut m = base(id, MessageKind::ToolResult);
                m.is_error = block
                    .get("is_error")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(false);
                m.tool_use_id = str_field(block, "tool_use_id");
                let (value, bytes, truncated) = cap_tool_result(block.get("content"));
                m.tool_result = value;
                m.tool_result_bytes = bytes;
                m.tool_result_truncated = truncated;
                out.push(m);
            }
            "image" => out.push(base(id, MessageKind::Image)),
            _ => {}
        }
    }

    // Usage belongs to the record, so only the first emitted message carries it.
    if let Some(first) = out.first_mut() {
        attach_turn_meta(first, has_usage, usage, model, cost);
    }
    out
}

fn attach_turn_meta(
    m: &mut Message,
    has_usage: bool,
    usage: TokenUsage,
    model: Option<String>,
    cost: Option<f64>,
) {
    m.is_turn_head = true;
    m.model = model;
    if has_usage {
        m.usage = Some(usage);
        m.cost_usd = cost;
    }
}

/// Cap a tool result's serialized size. Returns the (possibly replaced) value,
/// its original byte size, and whether it was truncated.
fn cap_tool_result(content: Option<&serde_json::Value>) -> (Option<serde_json::Value>, usize, bool) {
    let Some(v) = content else {
        return (None, 0, false);
    };
    let serialized = serde_json::to_string(v).unwrap_or_default();
    let bytes = serialized.len();
    if bytes <= TOOL_RESULT_CAP {
        return (Some(v.clone()), bytes, false);
    }
    // Keep a readable prefix rather than dropping the result entirely — the
    // first lines are usually the informative ones.
    let head: String = serialized.chars().take(TOOL_RESULT_CAP).collect();
    (Some(serde_json::Value::String(head)), bytes, true)
}

fn str_field(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(str::to_string)
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

    fn cache() -> IndexCache {
        IndexCache::new()
    }

    fn project(td: &Path, name: &str, session: &str, body: &str) {
        let dir = td.join(PROJECTS_SUBDIR).join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{session}.jsonl")), body).unwrap();
    }

    #[test]
    fn list_and_paginate() {
        let td = tempfile::tempdir().unwrap();
        let lines: Vec<String> = (0..5)
            .map(|i| format!(
                r#"{{"type":"user","timestamp":"2026-01-0{}T00:00:00Z","message":{{"role":"user","content":"hi {i}"}}}}"#,
                i + 1,
            ))
            .collect();
        project(td.path(), "-tmp-x", "01h", &lines.join("\n"));

        let summaries = list_for_project(td.path(), "-tmp-x").unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].message_count, 5);
        assert_eq!(summaries[0].preview.as_deref(), Some("hi 0"));

        let page = messages(td.path(), &cache(), "-tmp-x", "01h", Some(2), Some(2)).unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].content.as_deref(), Some("hi 2"));
        assert_eq!(page.next_after, Some(4));
    }

    #[test]
    fn seeking_returns_the_same_page_as_a_full_read() {
        let td = tempfile::tempdir().unwrap();
        let lines: Vec<String> = (0..200)
            .map(|i| format!(
                r#"{{"type":"assistant","uuid":"u{i}","message":{{"role":"assistant","content":[{{"type":"text","text":"m{i}"}}]}}}}"#
            ))
            .collect();
        project(td.path(), "-tmp-big", "s", &lines.join("\n"));

        let c = cache();
        let all = messages(td.path(), &c, "-tmp-big", "s", None, None).unwrap();
        assert_eq!(all.items.len(), 200);

        let page = messages(td.path(), &c, "-tmp-big", "s", Some(150), Some(10)).unwrap();
        let expected: Vec<_> = all.items[150..160].iter().map(|m| m.id.clone()).collect();
        let got: Vec<_> = page.items.iter().map(|m| m.id.clone()).collect();
        assert_eq!(got, expected);
        assert_eq!(page.total, Some(200));

        // Past the end is empty, not an error.
        let past = messages(td.path(), &c, "-tmp-big", "s", Some(500), Some(10)).unwrap();
        assert!(past.items.is_empty());
        assert!(past.next_after.is_none());
    }

    #[test]
    fn usage_attaches_only_to_the_turn_head() {
        let td = tempfile::tempdir().unwrap();
        let body = r#"{"type":"assistant","uuid":"u1","message":{"role":"assistant","model":"claude-sonnet-4-6","content":[{"type":"text","text":"a"},{"type":"text","text":"b"},{"type":"tool_use","id":"t1","name":"Bash","input":{}}],"usage":{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":10,"cache_creation_input_tokens":5}}}"#;
        project(td.path(), "-tmp-u", "s", body);

        let page = messages(td.path(), &cache(), "-tmp-u", "s", None, None).unwrap();
        assert_eq!(page.items.len(), 3);
        // A three-block turn must not count its tokens three times.
        assert!(page.items[0].is_turn_head);
        assert_eq!(page.items[0].usage.unwrap().input, 100);
        assert!(page.items[0].cost_usd.unwrap() > 0.0);
        assert!(page.items[1].usage.is_none());
        assert!(page.items[2].usage.is_none());
        assert_eq!(page.items[0].model.as_deref(), Some("claude-sonnet-4-6"));
    }

    #[test]
    fn tool_use_and_result_share_an_id() {
        let td = tempfile::tempdir().unwrap();
        let body = concat!(
            r#"{"type":"assistant","uuid":"u1","message":{"role":"assistant","content":[{"type":"tool_use","id":"toolu_1","name":"Bash","input":{"command":"ls"}}]}}"#,
            "\n",
            r#"{"type":"user","uuid":"u2","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_1","content":"a.txt"}]}}"#,
        );
        project(td.path(), "-tmp-t", "s", body);

        let page = messages(td.path(), &cache(), "-tmp-t", "s", None, None).unwrap();
        assert_eq!(page.items[0].tool_use_id.as_deref(), Some("toolu_1"));
        assert_eq!(page.items[1].tool_use_id.as_deref(), Some("toolu_1"));
        assert!(!page.items[1].tool_result_truncated);
    }

    #[test]
    fn oversize_tool_result_is_capped() {
        let td = tempfile::tempdir().unwrap();
        let huge = "x".repeat(40 * 1024);
        let body = format!(
            r#"{{"type":"user","uuid":"u1","message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"t","content":"{huge}"}}]}}}}"#
        );
        project(td.path(), "-tmp-c", "s", &body);

        let page = messages(td.path(), &cache(), "-tmp-c", "s", None, None).unwrap();
        let m = &page.items[0];
        assert!(m.tool_result_truncated);
        assert!(m.tool_result_bytes > TOOL_RESULT_CAP);
        let wire = serde_json::to_string(m).unwrap();
        assert!(wire.len() < 40 * 1024, "capped result still crossed IPC in full");
    }

    #[test]
    fn sidechain_and_parent_are_parsed() {
        let td = tempfile::tempdir().unwrap();
        let body = r#"{"type":"assistant","uuid":"u2","parentUuid":"u1","isSidechain":true,"gitBranch":"main","message":{"role":"assistant","content":[{"type":"text","text":"sub"}]}}"#;
        project(td.path(), "-tmp-sc", "s", body);
        let m = &messages(td.path(), &cache(), "-tmp-sc", "s", None, None).unwrap().items[0];
        assert!(m.is_sidechain);
        assert_eq!(m.parent_uuid.as_deref(), Some("u1"));
        assert_eq!(m.git_branch.as_deref(), Some("main"));
        assert_eq!(m.record_type.as_deref(), Some("assistant"));
    }

    #[test]
    fn mode_records_render_as_meta() {
        let td = tempfile::tempdir().unwrap();
        let body = concat!(
            r#"{"type":"mode","uuid":"m1","mode":"plan"}"#,
            "\n",
            r#"{"type":"permission-mode","uuid":"m2","permissionMode":"acceptEdits"}"#,
            "\n",
            r#"{"type":"ai-title","leafUuid":"x"}"#,
        );
        project(td.path(), "-tmp-m", "s", body);
        let page = messages(td.path(), &cache(), "-tmp-m", "s", None, None).unwrap();
        // mode + permission-mode render; ai-title stays dropped.
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].kind, MessageKind::Meta);
        assert!(page.items[0].content.as_deref().unwrap().contains("plan"));
        assert!(page.items[1].content.as_deref().unwrap().contains("acceptEdits"));
    }

    #[test]
    fn threads_group_sidechains_by_parent() {
        let td = tempfile::tempdir().unwrap();
        let body = concat!(
            // Main thread spawns two subagents.
            r#"{"type":"assistant","uuid":"main1","message":{"role":"assistant","content":[{"type":"tool_use","id":"task_a","name":"Task","input":{"subagent_type":"Explore"}}]}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a1","parentUuid":"task_a","isSidechain":true,"timestamp":"2026-01-01T00:00:01Z","message":{"role":"assistant","model":"claude-sonnet-4-6","content":[{"type":"text","text":"looking"}],"usage":{"input_tokens":10,"output_tokens":5}}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"a2","parentUuid":"a1","isSidechain":true,"timestamp":"2026-01-01T00:00:02Z","message":{"role":"assistant","model":"claude-sonnet-4-6","content":[{"type":"text","text":"found"}],"usage":{"input_tokens":20,"output_tokens":7}}}"#,
            "\n",
            r#"{"type":"assistant","uuid":"b1","parentUuid":"other","isSidechain":true,"timestamp":"2026-01-01T00:00:03Z","message":{"role":"assistant","content":[{"type":"text","text":"second agent"}]}}"#,
        );
        project(td.path(), "-tmp-th", "s", body);

        let threads = threads(td.path(), "-tmp-th", "s").unwrap();
        assert_eq!(threads.len(), 2);
        let a = threads.iter().find(|t| t.root_uuid == "a1").unwrap();
        assert_eq!(a.message_count, 2); // a1 + its child a2
        assert_eq!(a.usage.input, 30);
        assert_eq!(a.parent_tool_use_id.as_deref(), Some("task_a"));
        assert_eq!(a.agent_name.as_deref(), Some("Explore"));
        assert!(a.cost_usd.unwrap() > 0.0);
        assert_eq!(a.started_at.as_deref(), Some("2026-01-01T00:00:01Z"));
        assert_eq!(a.ended_at.as_deref(), Some("2026-01-01T00:00:02Z"));
    }

    #[test]
    fn tolerant_to_garbage_lines() {
        let td = tempfile::tempdir().unwrap();
        project(
            td.path(),
            "-tmp-y",
            "a",
            "not-json\n{}\n{\"type\":\"file-history-snapshot\"}\n",
        );
        let page = messages(td.path(), &cache(), "-tmp-y", "a", None, None).unwrap();
        assert!(page.items.is_empty());
        assert!(threads(td.path(), "-tmp-y", "a").unwrap().is_empty());
    }

    #[test]
    fn summary_record_overrides_first_user_message() {
        let td = tempfile::tempdir().unwrap();
        let body = concat!(
            r#"{"type":"summary","summary":"Refactor sessions list"}"#,
            "\n",
            r#"{"type":"user","timestamp":"2026-01-01T00:00:00Z","message":{"role":"user","content":"long original prompt"}}"#,
        );
        project(td.path(), "-tmp-s", "01", body);
        let summaries = list_for_project(td.path(), "-tmp-s").unwrap();
        assert_eq!(summaries[0].preview.as_deref(), Some("Refactor sessions list"));
    }

    #[test]
    fn missing_session_is_not_found() {
        let td = tempfile::tempdir().unwrap();
        assert!(messages(td.path(), &cache(), "-tmp-none", "nope", None, None).is_err());
        assert!(threads(td.path(), "-tmp-none", "nope").is_err());
    }
}
