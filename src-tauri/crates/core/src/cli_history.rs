//! Read-only view of `~/.claude/cli-history/`.
//!
//! Unlike most of `~/.claude`, this app *writes* these files: `crates/pty`
//! snapshots a session's ring buffer to `<session-uuid>.json` when the terminal
//! exits. So this is "replay my own past terminal sessions", and the schema is
//! ours.
//!
//! `lastLines` is raw scrollback — up to 10k lines of whatever was on screen,
//! ANSI escapes and all. `list` never loads it; `get` does, and the frontend
//! replays it locally through xterm.

use std::path::{Path, PathBuf};

use crate::types::{CliHistoryDetail, CliHistoryEntry};
use crate::AppError;

const HISTORY_SUBDIR: &str = "cli-history";
const PREVIEW_CHARS: usize = 120;

pub fn list(claude_dir: &Path) -> Result<Vec<CliHistoryEntry>, AppError> {
    let root = claude_dir.join(HISTORY_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match read_one(&path) {
            Ok(e) => out.push(e),
            Err(e) => {
                tracing::warn!(error = %e, path = %path.display(), "skipping unreadable terminal history")
            }
        }
    }
    // Newest first, by when the session ended.
    out.sort_by(|a, b| b.ended_at.cmp(&a.ended_at).then(a.id.cmp(&b.id)));
    Ok(out)
}

pub fn get(claude_dir: &Path, id: &str) -> Result<CliHistoryDetail, AppError> {
    let path = file_path(claude_dir, id)?;
    if !path.is_file() {
        return Err(AppError::not_found(format!(
            "terminal history '{id}' not found"
        )));
    }
    let v = read_json(&path)?;
    Ok(CliHistoryDetail {
        entry: parse_entry(&path, &v)?,
        lines: lines(&v),
    })
}

/// Session ids are uuids. Validating the charset keeps a crafted id from
/// walking out of the history directory.
fn file_path(claude_dir: &Path, id: &str) -> Result<PathBuf, AppError> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    if !valid {
        return Err(AppError::invalid(format!("invalid session id '{id}'")));
    }
    Ok(claude_dir.join(HISTORY_SUBDIR).join(format!("{id}.json")))
}

fn read_json(path: &Path) -> Result<serde_json::Value, AppError> {
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

/// Reads the whole file and drops `lastLines` immediately. Streaming past the
/// array would be faster, but these snapshots are capped at 10k lines and the
/// list is short; measure before adding a partial parser.
fn read_one(path: &Path) -> Result<CliHistoryEntry, AppError> {
    let v = read_json(path)?;
    parse_entry(path, &v)
}

fn parse_entry(path: &Path, v: &serde_json::Value) -> Result<CliHistoryEntry, AppError> {
    let meta = v.get("meta").cloned().unwrap_or(serde_json::Value::Null);
    let id = str_field(v, "id")
        .or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(str::to_string)
        })
        .unwrap_or_default();
    let all_lines = lines(v);

    Ok(CliHistoryEntry {
        id,
        agent_slug: str_field(&meta, "agentSlug"),
        model: str_field(&meta, "model"),
        working_dir: str_field(&meta, "workingDir"),
        started_at: str_field(&meta, "startedAt"),
        ended_at: str_field(v, "endedAt"),
        last_activity: str_field(&meta, "lastActivity"),
        exit_code: v.get("exitCode").and_then(|x| x.as_i64()),
        cols: meta.get("cols").and_then(|x| x.as_i64()),
        rows: meta.get("rows").and_then(|x| x.as_i64()),
        line_count: all_lines.len(),
        preview: preview(&all_lines),
    })
}

fn lines(v: &serde_json::Value) -> Vec<String> {
    v.get("lastLines")
        .and_then(|l| l.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// First line with visible content, ANSI stripped. A terminal's opening lines
/// are usually pure escape sequences, so "non-empty" has to mean non-empty
/// *after* stripping.
fn preview(lines: &[String]) -> String {
    for line in lines {
        let cleaned = strip_ansi(line);
        let trimmed = cleaned.trim();
        if !trimmed.is_empty() {
            return truncate(trimmed, PREVIEW_CHARS);
        }
    }
    String::new()
}

pub fn strip_ansi(s: &str) -> String {
    String::from_utf8(strip_ansi_escapes::strip(s.as_bytes())).unwrap_or_else(|_| s.to_string())
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars).collect();
    out.push('…');
    out
}

fn str_field(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(claude_dir: &Path, id: &str, body: &str) {
        let dir = claude_dir.join(HISTORY_SUBDIR);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{id}.json")), body).unwrap();
    }

    const SNAPSHOT: &str = r#"{
      "id": "3a5d9134-d900-4227-ae65-d7f4a01d07b1",
      "endedAt": "2026-05-12T02:04:52Z",
      "exitCode": 0,
      "lastLines": ["\u001b[?25h", "  ", "\u001b[32mhello\u001b[0m world", "second"],
      "meta": {
        "id": "3a5d9134-d900-4227-ae65-d7f4a01d07b1",
        "agentSlug": null,
        "alive": false,
        "cols": 98,
        "rows": 54,
        "model": null,
        "startedAt": "2026-05-12T02:00:00Z",
        "lastActivity": "2026-05-12T02:04:52Z",
        "workingDir": "/Users/x/proj"
      }
    }"#;

    #[test]
    fn preview_strips_ansi_and_skips_blank_lines() {
        let td = tempfile::tempdir().unwrap();
        write(td.path(), "3a5d9134-d900-4227-ae65-d7f4a01d07b1", SNAPSHOT);

        let e = &list(td.path()).unwrap()[0];
        assert_eq!(e.preview, "hello world");
        assert_eq!(e.line_count, 4);
        assert_eq!(e.cols, Some(98));
        assert_eq!(e.working_dir.as_deref(), Some("/Users/x/proj"));
        assert_eq!(e.exit_code, Some(0));
    }

    #[test]
    fn list_carries_no_scrollback() {
        let td = tempfile::tempdir().unwrap();
        write(td.path(), "3a5d9134-d900-4227-ae65-d7f4a01d07b1", SNAPSHOT);
        // The entry type has no `lines` field at all, so the list response
        // cannot carry 10k lines of terminal output by accident.
        let wire = serde_json::to_string(&list(td.path()).unwrap()).unwrap();
        assert!(!wire.contains("second"));

        let detail = get(td.path(), "3a5d9134-d900-4227-ae65-d7f4a01d07b1").unwrap();
        assert_eq!(detail.lines.len(), 4);
        assert!(detail.lines[2].contains('\u{1b}'), "escapes kept for replay");
    }

    #[test]
    fn missing_last_lines_yields_empty_preview() {
        let td = tempfile::tempdir().unwrap();
        write(td.path(), "abc", r#"{"endedAt":"2026-01-01T00:00:00Z"}"#);
        let e = &list(td.path()).unwrap()[0];
        assert_eq!(e.id, "abc"); // falls back to the file stem
        assert_eq!(e.line_count, 0);
        assert_eq!(e.preview, "");
    }

    #[test]
    fn corrupt_file_is_skipped_not_fatal() {
        let td = tempfile::tempdir().unwrap();
        write(td.path(), "bad", "{not json");
        write(td.path(), "ok", SNAPSHOT);
        assert_eq!(list(td.path()).unwrap().len(), 1);
    }

    #[test]
    fn list_sorted_newest_first() {
        let td = tempfile::tempdir().unwrap();
        write(td.path(), "old", r#"{"endedAt":"2026-01-01T00:00:00Z"}"#);
        write(td.path(), "new", r#"{"endedAt":"2026-06-01T00:00:00Z"}"#);
        assert_eq!(list(td.path()).unwrap()[0].id, "new");
    }

    #[test]
    fn id_traversal_rejected() {
        let td = tempfile::tempdir().unwrap();
        assert!(get(td.path(), "../../etc/passwd").is_err());
        assert!(get(td.path(), "").is_err());
        assert!(get(td.path(), "missing").is_err()); // well-formed, absent
    }

    #[test]
    fn missing_dir_is_empty() {
        let td = tempfile::tempdir().unwrap();
        assert!(list(td.path()).unwrap().is_empty());
    }
}
