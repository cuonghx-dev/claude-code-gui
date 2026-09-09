//! Hooks configured under the `hooks` key of a settings file, across every
//! scope.
//!
//! settings.json shape (events keyed top-level under `hooks`):
//!
//! ```jsonc
//! {
//!   "hooks": {
//!     "PostToolUse": [
//!       { "matcher": "Bash", "hooks": [{ "type": "command", "command": "..." }] }
//!     ],
//!     "SessionStart": [ { "hooks": [...] } ]
//!   }
//! }
//! ```
//!
//! Writes replace one event's whole array, because an RFC 7386 merge patch
//! cannot remove a single array element — and replacing the array is exactly
//! the semantics we want. Removing the last group for an event patches the
//! event to `null`, which deletes the key.
//!
//! Every hook runs an arbitrary shell command on tool calls, so this module
//! never executes one; it only reads and writes configuration.

use std::path::Path;

use serde_json::{json, Value};

use crate::settings_scope;
use crate::types::{HookEntry, HookGroup, HookInput, SettingsScope};
use crate::AppError;

/// Seconds. Beyond this a hook would stall the CLI for minutes.
const MAX_TIMEOUT: u64 = 600;

pub fn list(claude_dir: &Path, working_dir: Option<&Path>) -> Result<Vec<HookGroup>, AppError> {
    let mut out = Vec::new();
    for &scope in SettingsScope::ALL.iter() {
        let Ok(path) = settings_scope::scope_path(claude_dir, scope, working_dir) else {
            continue;
        };
        extend_from(&path, scope, &mut out);
    }
    out.sort_by(|a, b| {
        event_order(&a.event)
            .cmp(&event_order(&b.event))
            .then(a.event.cmp(&b.event))
            .then(a.scope.cmp(&b.scope))
            .then(a.index.cmp(&b.index))
    });
    Ok(out)
}

pub fn get(
    claude_dir: &Path,
    working_dir: Option<&Path>,
    id: &str,
) -> Result<HookGroup, AppError> {
    list(claude_dir, working_dir)?
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| AppError::not_found(format!("hook '{id}' not found")))
}

pub fn create(claude_dir: &Path, input: &HookInput) -> Result<HookGroup, AppError> {
    validate(input)?;
    let wd = input.working_dir.as_ref().map(Path::new);
    let mut groups = raw_groups(claude_dir, input.scope, wd, &input.event)?;
    groups.push(group_value(input));
    let index = groups.len() - 1;
    write_event(claude_dir, input, &groups)?;
    get(claude_dir, wd, &group_id(input.scope, &input.event, index))
}

pub fn update(claude_dir: &Path, id: &str, input: &HookInput) -> Result<HookGroup, AppError> {
    validate(input)?;
    let (scope, event, index) = parse_id(id)?;
    let wd = input.working_dir.as_ref().map(Path::new);

    // Moving a group to another event or scope is a delete plus a create.
    if scope != input.scope || event != input.event {
        delete(claude_dir, wd, id)?;
        return create(claude_dir, input);
    }

    let mut groups = raw_groups(claude_dir, scope, wd, &event)?;
    if index >= groups.len() {
        return Err(AppError::not_found(format!("hook '{id}' no longer exists")));
    }
    groups[index] = group_value(input);
    write_event(claude_dir, input, &groups)?;
    get(claude_dir, wd, id)
}

pub fn delete(claude_dir: &Path, working_dir: Option<&Path>, id: &str) -> Result<(), AppError> {
    let (scope, event, index) = parse_id(id)?;
    let mut groups = raw_groups(claude_dir, scope, working_dir, &event)?;
    if index >= groups.len() {
        return Err(AppError::not_found(format!("hook '{id}' no longer exists")));
    }
    groups.remove(index);

    let patch = if groups.is_empty() {
        // Deleting the last group removes the event key rather than leaving an
        // empty array behind.
        json!({ "hooks": { event.clone(): Value::Null } })
    } else {
        json!({ "hooks": { event.clone(): groups } })
    };
    settings_scope::patch(claude_dir, scope, working_dir, &patch, None)?;
    Ok(())
}

/// The `hooks` subtree of one scope, pretty-printed for the raw editor.
pub fn raw_get(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
) -> Result<String, AppError> {
    let path = settings_scope::scope_path(claude_dir, scope, working_dir)?;
    let doc = crate::io::read_json_doc(&path)?;
    let hooks = doc
        .value
        .get("hooks")
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));
    Ok(serde_json::to_string_pretty(&hooks)?)
}

pub fn raw_put(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
    content: &str,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let value: Value = serde_json::from_str(content)?;
    if !value.is_object() {
        return Err(AppError::invalid("hooks must be a JSON object keyed by event"));
    }
    // Replacing the whole subtree means a merge patch would leave removed
    // events behind, so clear it first.
    settings_scope::patch(
        claude_dir,
        scope,
        working_dir,
        &json!({ "hooks": Value::Null }),
        expected_mtime_ms,
    )?;
    settings_scope::patch(claude_dir, scope, working_dir, &json!({ "hooks": value }), None)
}

fn validate(input: &HookInput) -> Result<(), AppError> {
    if input.event.trim().is_empty() {
        return Err(AppError::invalid("hook event is required"));
    }
    if input.entries.is_empty() {
        return Err(AppError::invalid("a hook needs at least one command"));
    }
    if let Some(matcher) = &input.matcher {
        if !matcher.is_empty() && regex::Regex::new(matcher).is_err() {
            // Warn-shaped, but there is nowhere to put a warning on a write, and
            // an unparseable matcher silently matches nothing.
            return Err(AppError::invalid(format!(
                "matcher '{matcher}' is not a valid regular expression"
            )));
        }
    }
    for e in &input.entries {
        match e.command.as_deref() {
            None | Some("") => return Err(AppError::invalid("hook command cannot be empty")),
            _ => {}
        }
        if let Some(kind) = &e.kind {
            if kind != "command" {
                return Err(AppError::invalid(format!("unsupported hook type '{kind}'")));
            }
        }
        if let Some(t) = e.timeout {
            if t == 0 || t > MAX_TIMEOUT {
                return Err(AppError::invalid(format!(
                    "timeout must be between 1 and {MAX_TIMEOUT} seconds"
                )));
            }
        }
    }
    Ok(())
}

/// The raw `hooks.<event>` array of one scope, untouched.
fn raw_groups(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
    event: &str,
) -> Result<Vec<Value>, AppError> {
    let path = settings_scope::scope_path(claude_dir, scope, working_dir)?;
    let doc = crate::io::read_json_doc(&path)?;
    Ok(doc
        .value
        .get("hooks")
        .and_then(|h| h.get(event))
        .and_then(|e| e.as_array())
        .cloned()
        .unwrap_or_default())
}

fn write_event(claude_dir: &Path, input: &HookInput, groups: &[Value]) -> Result<(), AppError> {
    settings_scope::patch(
        claude_dir,
        input.scope,
        input.working_dir.as_ref().map(Path::new),
        &json!({ "hooks": { input.event.clone(): groups } }),
        input.expected_mtime_ms,
    )?;
    Ok(())
}

fn group_value(input: &HookInput) -> Value {
    let mut group = serde_json::Map::new();
    if let Some(m) = &input.matcher {
        if !m.is_empty() {
            group.insert("matcher".into(), Value::String(m.clone()));
        }
    }
    let entries: Vec<Value> = input
        .entries
        .iter()
        .map(|e| {
            let mut o = serde_json::Map::new();
            o.insert(
                "type".into(),
                Value::String(e.kind.clone().unwrap_or_else(|| "command".into())),
            );
            if let Some(c) = &e.command {
                o.insert("command".into(), Value::String(c.clone()));
            }
            if let Some(t) = e.timeout {
                o.insert("timeout".into(), json!(t));
            }
            if let Some(s) = &e.status_message {
                o.insert("statusMessage".into(), Value::String(s.clone()));
            }
            // Keys this app does not model survive an edit.
            for (k, v) in &e.extra {
                o.insert(k.clone(), v.clone());
            }
            Value::Object(o)
        })
        .collect();
    group.insert("hooks".into(), Value::Array(entries));
    Value::Object(group)
}

fn extend_from(path: &Path, scope: SettingsScope, out: &mut Vec<HookGroup>) {
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
        for (index, group) in arr.iter().enumerate() {
            let entries = group
                .get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|e| serde_json::from_value::<HookEntry>(e.clone()).ok())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            // An empty group used to be hidden. With editing, it is a real
            // state a person can produce and then needs to be able to remove.
            out.push(HookGroup {
                id: group_id(scope, event, index),
                scope,
                index: index as u32,
                file_path: path.to_string_lossy().into_owned(),
                event: event.clone(),
                matcher: group
                    .get("matcher")
                    .and_then(|m| m.as_str())
                    .map(str::to_string),
                entries,
            });
        }
    }
}

fn group_id(scope: SettingsScope, event: &str, index: usize) -> String {
    let scope = serde_json::to_value(scope)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default();
    format!("{scope}:{event}:{index}")
}

fn parse_id(id: &str) -> Result<(SettingsScope, String, usize), AppError> {
    let mut parts = id.rsplitn(2, ':');
    let index = parts
        .next()
        .and_then(|i| i.parse::<usize>().ok())
        .ok_or_else(|| AppError::invalid(format!("malformed hook id '{id}'")))?;
    let head = parts
        .next()
        .ok_or_else(|| AppError::invalid(format!("malformed hook id '{id}'")))?;
    let (scope_str, event) = head
        .split_once(':')
        .ok_or_else(|| AppError::invalid(format!("malformed hook id '{id}'")))?;
    let scope: SettingsScope = serde_json::from_value(Value::String(scope_str.to_string()))
        .map_err(|_| AppError::invalid(format!("unknown scope '{scope_str}'")))?;
    Ok((scope, event.to_string(), index))
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

    fn input(scope: SettingsScope, event: &str, command: &str) -> HookInput {
        HookInput {
            scope,
            working_dir: None,
            event: event.into(),
            matcher: Some("Bash".into()),
            entries: vec![HookEntry {
                kind: Some("command".into()),
                command: Some(command.into()),
                timeout: Some(5),
                status_message: None,
                extra: Default::default(),
            }],
            expected_mtime_ms: None,
        }
    }

    #[test]
    fn parses_hooks_with_scope_attribution() {
        let td = tempfile::tempdir().unwrap();
        let wd = td.path().join("proj");
        std::fs::create_dir_all(wd.join(".claude")).unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"hooks":{"PostToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"echo user","timeout":5}]}]}}"#,
        )
        .unwrap();
        std::fs::write(
            wd.join(".claude/settings.json"),
            r#"{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"echo project","statusMessage":"loading"}]}]}}"#,
        )
        .unwrap();

        let groups = list(td.path(), Some(&wd)).unwrap();
        assert_eq!(groups.len(), 2);
        let post = groups.iter().find(|g| g.event == "PostToolUse").unwrap();
        assert_eq!(post.scope, SettingsScope::User);
        assert_eq!(post.id, "user:PostToolUse:0");
        assert!(post.file_path.ends_with("settings.json"));
        let start = groups.iter().find(|g| g.event == "SessionStart").unwrap();
        assert_eq!(start.scope, SettingsScope::Project);
        assert_eq!(
            start.entries[0].status_message.as_deref(),
            Some("loading")
        );
    }

    #[test]
    fn create_appends_and_preserves_siblings() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"model":"opus","hooks":{"PostToolUse":[{"hooks":[{"type":"command","command":"first"}]}]}}"#,
        )
        .unwrap();

        let g = create(td.path(), &input(SettingsScope::User, "PostToolUse", "second")).unwrap();
        assert_eq!(g.id, "user:PostToolUse:1");

        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(td.path().join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(v["model"], "opus", "unrelated settings survive");
        let arr = v["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["hooks"][0]["command"], "first");
        assert_eq!(arr[1]["matcher"], "Bash");
    }

    #[test]
    fn update_preserves_other_events_and_unknown_entry_keys() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"keep me"}]}],"PostToolUse":[{"hooks":[{"type":"command","command":"old","futureKey":42}]}]}}"#,
        )
        .unwrap();

        let existing = get(td.path(), None, "user:PostToolUse:0").unwrap();
        assert_eq!(existing.entries[0].extra["futureKey"], 42);

        let mut inp = input(SettingsScope::User, "PostToolUse", "new");
        inp.entries[0].extra = existing.entries[0].extra.clone();
        update(td.path(), "user:PostToolUse:0", &inp).unwrap();

        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(td.path().join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "keep me");
        assert_eq!(v["hooks"]["PostToolUse"][0]["hooks"][0]["command"], "new");
        assert_eq!(v["hooks"]["PostToolUse"][0]["hooks"][0]["futureKey"], 42);
    }

    #[test]
    fn deleting_the_last_group_removes_the_event_key() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"model":"opus","hooks":{"PostToolUse":[{"hooks":[{"type":"command","command":"a"}]},{"hooks":[{"type":"command","command":"b"}]}]}}"#,
        )
        .unwrap();

        delete(td.path(), None, "user:PostToolUse:0").unwrap();
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(td.path().join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(v["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(v["hooks"]["PostToolUse"][0]["hooks"][0]["command"], "b");

        delete(td.path(), None, "user:PostToolUse:0").unwrap();
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(td.path().join("settings.json")).unwrap())
                .unwrap();
        assert!(v["hooks"].get("PostToolUse").is_none());
        assert_eq!(v["model"], "opus");
    }

    #[test]
    fn validation_rejects_bad_input() {
        let td = tempfile::tempdir().unwrap();
        let mut empty = input(SettingsScope::User, "PostToolUse", "x");
        empty.entries.clear();
        assert!(create(td.path(), &empty).is_err());

        let mut no_cmd = input(SettingsScope::User, "PostToolUse", "");
        no_cmd.entries[0].command = Some(String::new());
        assert!(create(td.path(), &no_cmd).is_err());

        let mut bad_regex = input(SettingsScope::User, "PostToolUse", "x");
        bad_regex.matcher = Some("(unclosed".into());
        assert!(create(td.path(), &bad_regex).is_err());

        let mut bad_timeout = input(SettingsScope::User, "PostToolUse", "x");
        bad_timeout.entries[0].timeout = Some(9999);
        assert!(create(td.path(), &bad_timeout).is_err());

        // An unknown event is accepted: the CLI adds events over time.
        assert!(create(td.path(), &input(SettingsScope::User, "SomeNewEvent", "x")).is_ok());
    }

    #[test]
    fn managed_scope_cannot_be_written() {
        let td = tempfile::tempdir().unwrap();
        assert!(create(td.path(), &input(SettingsScope::Managed, "PostToolUse", "x")).is_err());
    }

    #[test]
    fn raw_round_trip_replaces_the_whole_subtree() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"model":"opus","hooks":{"PreToolUse":[{"hooks":[{"type":"command","command":"a"}]}],"Stop":[{"hooks":[{"type":"command","command":"b"}]}]}}"#,
        )
        .unwrap();

        raw_put(
            td.path(),
            SettingsScope::User,
            None,
            r#"{"Stop":[{"hooks":[{"type":"command","command":"only"}]}]}"#,
            None,
        )
        .unwrap();

        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(td.path().join("settings.json")).unwrap())
                .unwrap();
        assert!(v["hooks"].get("PreToolUse").is_none(), "removed event is gone");
        assert_eq!(v["hooks"]["Stop"][0]["hooks"][0]["command"], "only");
        assert_eq!(v["model"], "opus");

        assert!(raw_put(td.path(), SettingsScope::User, None, "[]", None).is_err());
    }

    #[test]
    fn missing_settings_returns_empty() {
        let td = tempfile::tempdir().unwrap();
        assert!(list(td.path(), None).unwrap().is_empty());
        assert!(get(td.path(), None, "user:Stop:0").is_err());
        assert!(delete(td.path(), None, "nonsense").is_err());
    }
}
