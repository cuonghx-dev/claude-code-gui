//! Read-only listing of hooks configured in `~/.claude/settings.json`
//! (and optionally `<wd>/.claude/settings.json`).
//!
//! settings.json shape (events keyed top-level under `hooks`):
//!
//! ```jsonc
//! {
//!   "hooks": {
//!     "PostToolUse": [
//!       { "matcher": "Bash", "hooks": [{ "type": "command", "command": "..." }] }
//!     ],
//!     "SessionStart": [
//!       { "hooks": [...] }
//!     ]
//!   }
//! }
//! ```

use std::path::Path;

use serde_json::Value;

use crate::types::{HookEntry, HookGroup};
use crate::AppError;

pub fn list(claude_dir: &Path, working_dir: Option<&Path>) -> Result<Vec<HookGroup>, AppError> {
    let mut out = Vec::new();
    extend_from(&claude_dir.join("settings.json"), &mut out);
    if let Some(wd) = working_dir {
        extend_from(&wd.join(".claude").join("settings.json"), &mut out);
    }
    out.sort_by(|a, b| event_order(&a.event).cmp(&event_order(&b.event)).then(a.event.cmp(&b.event)));
    Ok(out)
}

fn extend_from(path: &Path, out: &mut Vec<HookGroup>) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return;
    };
    let Some(hooks_obj) = v.get("hooks").and_then(|h| h.as_object()) else {
        return;
    };
    for (event, value) in hooks_obj {
        let Some(arr) = value.as_array() else {
            continue;
        };
        for group in arr {
            let matcher = group
                .get("matcher")
                .and_then(|m| m.as_str())
                .map(|s| s.to_string());
            let entries = group
                .get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|e| serde_json::from_value::<HookEntry>(e.clone()).ok())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if entries.is_empty() {
                continue;
            }
            out.push(HookGroup {
                event: event.clone(),
                matcher,
                entries,
            });
        }
    }
}

/// Stable display order. Unknown events sort to the end alphabetically.
fn event_order(event: &str) -> u8 {
    match event {
        "PostToolUse" => 0,
        "PreToolUse" => 1,
        "UserPromptSubmit" => 2,
        "SessionStart" => 3,
        "SessionEnd" => 4,
        "Notification" => 5,
        "Stop" => 6,
        "SubagentStop" => 7,
        "PreCompact" => 8,
        _ => 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_global_hooks() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{
              "hooks": {
                "PostToolUse": [
                  {"matcher":"Bash","hooks":[{"type":"command","command":"echo hi","timeout":5}]}
                ],
                "SessionStart": [
                  {"hooks":[{"type":"command","command":"echo start","statusMessage":"loading"}]}
                ]
              }
            }"#,
        )
        .unwrap();
        let groups = list(td.path(), None).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].event, "PostToolUse");
        assert_eq!(groups[0].matcher.as_deref(), Some("Bash"));
        assert_eq!(groups[0].entries[0].command.as_deref(), Some("echo hi"));
        assert_eq!(groups[0].entries[0].timeout, Some(5));
        assert_eq!(groups[1].event, "SessionStart");
        assert!(groups[1].matcher.is_none());
        assert_eq!(
            groups[1].entries[0].status_message.as_deref(),
            Some("loading"),
        );
    }

    #[test]
    fn missing_settings_returns_empty() {
        let td = tempfile::tempdir().unwrap();
        let groups = list(td.path(), None).unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn empty_entries_dropped() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"hooks":{"PreToolUse":[{"matcher":"X","hooks":[]}]}}"#,
        )
        .unwrap();
        assert!(list(td.path(), None).unwrap().is_empty());
    }
}
