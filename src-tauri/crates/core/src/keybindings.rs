//! `~/.claude/keybindings.json`.
//!
//! Shape, per the keybindings docs:
//!
//! ```jsonc
//! {
//!   "$schema": "https://www.schemastore.org/claude-code-keybindings.json",
//!   "bindings": [
//!     { "context": "Chat", "bindings": { "ctrl+e": "chat:externalEditor", "ctrl+u": null } }
//!   ]
//! }
//! ```
//!
//! An action of `null` unbinds a default. Chords are keystrokes separated by
//! spaces (`ctrl+k ctrl+s`); modifiers join with `+`. The file often does not
//! exist — `/keybindings` creates it — so everything here handles absence as a
//! normal state rather than an error.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::io;
use crate::types::{ChordValidation, Keybinding, KeybindingsDoc};
use crate::AppError;

const FILE: &str = "keybindings.json";
const SCHEMA_URL: &str = "https://www.schemastore.org/claude-code-keybindings.json";
const DOCS_URL: &str = "https://code.claude.com/docs/en/keybindings";

/// Documented contexts. An unknown one is still accepted — Claude Code adds
/// contexts, and it warns about the ones it does not know.
const KNOWN_CONTEXTS: &[&str] = &[
    "Global", "Chat", "Autocomplete", "Settings", "Confirmation", "Tabs", "Help", "Transcript",
    "HistorySearch", "Task", "ThemePicker", "Attachments", "Footer", "MessageSelector",
    "DiffDialog", "DiffPanel", "ModelPicker", "Select", "Plugin", "Scroll",
];

const MODIFIERS: &[&str] = &[
    "ctrl", "control", "shift", "alt", "opt", "option", "meta", "cmd", "command", "super", "win",
];

const NAMED_KEYS: &[&str] = &[
    "escape", "esc", "enter", "return", "tab", "space", "up", "down", "left", "right", "pageup",
    "pagedown", "home", "end", "backspace", "delete", "wheelup", "wheeldown",
];

/// Cannot be rebound, per the docs.
const RESERVED: &[&str] = &["ctrl+c", "ctrl+d", "ctrl+m", "ctrl+[", "ctrl+i", "ctrl+h"];

pub fn path(claude_dir: &Path) -> PathBuf {
    claude_dir.join(FILE)
}

pub fn get(claude_dir: &Path) -> Result<KeybindingsDoc, AppError> {
    let p = path(claude_dir);
    let doc = io::read_json_doc(&p)?;

    let mut bindings = Vec::new();
    if let Some(blocks) = doc.value.get("bindings").and_then(|b| b.as_array()) {
        for block in blocks {
            let context = block
                .get("context")
                .and_then(|c| c.as_str())
                .unwrap_or("Global")
                .to_string();
            let Some(map) = block.get("bindings").and_then(|b| b.as_object()) else {
                continue;
            };
            for (key, action) in map {
                bindings.push(Keybinding {
                    context: context.clone(),
                    key: key.clone(),
                    // `null` means "unbind the default", which is a binding in
                    // its own right and must survive a round trip.
                    action: action.as_str().map(str::to_string),
                });
            }
        }
    }

    Ok(KeybindingsDoc {
        path: p.to_string_lossy().into_owned(),
        exists: doc.exists,
        mtime_ms: doc.mtime_ms,
        bindings,
    })
}

pub fn put(
    claude_dir: &Path,
    bindings: &[Keybinding],
    expected_mtime_ms: Option<i64>,
) -> Result<KeybindingsDoc, AppError> {
    for b in bindings {
        if b.key.trim().is_empty() {
            return Err(AppError::invalid("a binding needs a key"));
        }
        if RESERVED.contains(&normalize_chord(&b.key).as_str()) {
            return Err(AppError::invalid(format!(
                "'{}' is reserved by Claude Code and cannot be rebound",
                b.key
            )));
        }
    }

    let p = path(claude_dir);
    let existing = io::read_json_doc(&p)?;

    // Regroup the flat list back into per-context blocks.
    let mut blocks: Vec<(String, serde_json::Map<String, Value>)> = Vec::new();
    for b in bindings {
        let slot = match blocks.iter_mut().find(|(c, _)| c == &b.context) {
            Some(s) => &mut s.1,
            None => {
                blocks.push((b.context.clone(), serde_json::Map::new()));
                &mut blocks.last_mut().expect("just pushed").1
            }
        };
        slot.insert(
            b.key.clone(),
            b.action
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
    }

    let mut value = if existing.value.is_object() {
        existing.value
    } else {
        Value::Object(Default::default())
    };
    let obj = value.as_object_mut().expect("object");
    // Keep the schema pointers so editors keep autocompleting.
    obj.entry("$schema".to_string())
        .or_insert_with(|| json!(SCHEMA_URL));
    obj.entry("$docs".to_string())
        .or_insert_with(|| json!(DOCS_URL));
    obj.insert(
        "bindings".into(),
        Value::Array(
            blocks
                .into_iter()
                .map(|(context, map)| json!({ "context": context, "bindings": map }))
                .collect(),
        ),
    );

    io::atomic_write_json(&p, &value, expected_mtime_ms)?;
    get(claude_dir)
}

/// Write the empty skeleton for a machine that has no keybindings file yet.
pub fn create_default(claude_dir: &Path) -> Result<KeybindingsDoc, AppError> {
    let p = path(claude_dir);
    if p.is_file() {
        return Err(AppError::invalid("keybindings.json already exists"));
    }
    io::atomic_write_json(
        &p,
        &json!({ "$schema": SCHEMA_URL, "$docs": DOCS_URL, "bindings": [] }),
        None,
    )?;
    get(claude_dir)
}

pub fn raw_get(claude_dir: &Path) -> Result<String, AppError> {
    let p = path(claude_dir);
    Ok(std::fs::read_to_string(&p).unwrap_or_default())
}

pub fn raw_put(
    claude_dir: &Path,
    content: &str,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let value: Value = serde_json::from_str(content)?;
    if !value.is_object() {
        return Err(AppError::invalid("keybindings.json must be a JSON object"));
    }
    let p = path(claude_dir);
    io::check_unchanged(&p, expected_mtime_ms)?;
    io::backup_sibling(&p)?;
    io::atomic_write(&p, content.as_bytes())?;
    Ok(io::mtime_ms(&p).unwrap_or(0))
}

/// Check one chord and return its canonical spelling.
pub fn validate_chord(chord: &str) -> ChordValidation {
    let mut issues = Vec::new();
    let trimmed = chord.trim();
    if trimmed.is_empty() {
        return ChordValidation {
            valid: false,
            normalized: None,
            issues: vec!["chord cannot be empty".into()],
        };
    }

    for stroke in trimmed.split_whitespace() {
        let parts: Vec<&str> = stroke.split('+').collect();
        // A trailing `+` is how you bind the plus key itself: "ctrl++".
        let (mods, key) = match parts.as_slice() {
            [] => (&[][..], ""),
            [.., ""] if parts.len() > 1 => (&parts[..parts.len() - 2], "+"),
            [.., last] => (&parts[..parts.len() - 1], *last),
        };
        for m in mods {
            if !MODIFIERS.contains(&m.to_ascii_lowercase().as_str()) {
                issues.push(format!("'{m}' is not a modifier"));
            }
        }
        let lower = key.to_ascii_lowercase();
        let known = NAMED_KEYS.contains(&lower.as_str())
            || lower.chars().count() == 1
            || (lower.starts_with('f') && lower[1..].parse::<u8>().is_ok());
        if !known {
            issues.push(format!("'{key}' is not a key name"));
        }
    }

    let normalized = normalize_chord(trimmed);
    if RESERVED.contains(&normalized.as_str()) {
        issues.push(format!("{normalized} is reserved and cannot be rebound"));
    }

    ChordValidation {
        valid: issues.is_empty(),
        normalized: Some(normalized),
        issues,
    }
}

/// Lowercase, canonical modifier order, single spaces between strokes. Key
/// names are matched case-insensitively by Claude Code, so `Ctrl+K` and
/// `ctrl+k` are the same binding and must compare equal here.
pub fn normalize_chord(chord: &str) -> String {
    chord
        .split_whitespace()
        .map(|stroke| {
            let parts: Vec<String> = stroke.split('+').map(|p| p.to_ascii_lowercase()).collect();
            let (mods, key) = match parts.as_slice() {
                [.., last] if last.is_empty() && parts.len() > 1 => {
                    (parts[..parts.len() - 2].to_vec(), "+".to_string())
                }
                [.., last] => (parts[..parts.len() - 1].to_vec(), last.clone()),
                [] => (vec![], String::new()),
            };
            let mut mods: Vec<String> = mods;
            mods.sort_by_key(|m| MODIFIERS.iter().position(|k| k == m).unwrap_or(usize::MAX));
            mods.push(key);
            mods.join("+")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Bindings that collide: the same normalized chord twice in one context.
pub fn duplicates(bindings: &[Keybinding]) -> Vec<String> {
    let mut seen: Vec<(String, String)> = Vec::new();
    let mut dupes = Vec::new();
    for b in bindings {
        let key = (b.context.clone(), normalize_chord(&b.key));
        if seen.contains(&key) {
            dupes.push(format!("{} in {}", b.key, b.context));
        } else {
            seen.push(key);
        }
    }
    dupes
}

/// Contexts that are not in the documented list. Not an error — just worth
/// surfacing, since a typo silently does nothing.
pub fn unknown_contexts(bindings: &[Keybinding]) -> Vec<String> {
    let mut out: Vec<String> = bindings
        .iter()
        .map(|b| b.context.clone())
        .filter(|c| !KNOWN_CONTEXTS.contains(&c.as_str()))
        .collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"{
      "$schema": "https://www.schemastore.org/claude-code-keybindings.json",
      "bindings": [
        { "context": "Chat", "bindings": { "ctrl+e": "chat:externalEditor", "ctrl+u": null } }
      ]
    }"#;

    #[test]
    fn a_missing_file_reads_as_absent_not_an_error() {
        let td = tempfile::tempdir().unwrap();
        let doc = get(td.path()).unwrap();
        assert!(!doc.exists);
        assert!(doc.bindings.is_empty());
        assert!(doc.path.ends_with("keybindings.json"));
    }

    #[test]
    fn create_default_writes_a_valid_skeleton() {
        let td = tempfile::tempdir().unwrap();
        let doc = create_default(td.path()).unwrap();
        assert!(doc.exists);
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(path(td.path())).unwrap()).unwrap();
        assert_eq!(v["bindings"], json!([]));
        assert!(v["$schema"].is_string());
        // A second call must not clobber the file.
        assert!(create_default(td.path()).is_err());
    }

    #[test]
    fn unbindings_survive_a_round_trip() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(path(td.path()), DOC).unwrap();

        let doc = get(td.path()).unwrap();
        assert_eq!(doc.bindings.len(), 2);
        let unbound = doc.bindings.iter().find(|b| b.key == "ctrl+u").unwrap();
        assert!(unbound.action.is_none(), "null means unbind the default");

        put(td.path(), &doc.bindings, None).unwrap();
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(path(td.path())).unwrap()).unwrap();
        assert_eq!(v["bindings"][0]["bindings"]["ctrl+u"], Value::Null);
        assert_eq!(v["bindings"][0]["bindings"]["ctrl+e"], "chat:externalEditor");
        assert!(v["$schema"].is_string(), "schema pointer kept");
    }

    #[test]
    fn put_regroups_bindings_by_context() {
        let td = tempfile::tempdir().unwrap();
        let bindings = vec![
            Keybinding { context: "Chat".into(), key: "ctrl+e".into(), action: Some("chat:submit".into()) },
            Keybinding { context: "Task".into(), key: "ctrl+b".into(), action: Some("task:background".into()) },
            Keybinding { context: "Chat".into(), key: "ctrl+j".into(), action: Some("chat:newline".into()) },
        ];
        put(td.path(), &bindings, None).unwrap();
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(path(td.path())).unwrap()).unwrap();
        let blocks = v["bindings"].as_array().unwrap();
        assert_eq!(blocks.len(), 2, "one block per context");
        assert_eq!(blocks[0]["bindings"].as_object().unwrap().len(), 2);
    }

    #[test]
    fn chords_normalize_case_and_modifier_order() {
        assert_eq!(normalize_chord("Ctrl+K"), "ctrl+k");
        assert_eq!(normalize_chord("shift+ctrl+p"), "ctrl+shift+p");
        assert_eq!(normalize_chord("ctrl+k  ctrl+s"), "ctrl+k ctrl+s");
        assert_eq!(normalize_chord("ctrl++"), "ctrl++");
    }

    #[test]
    fn chord_validation_accepts_the_documented_forms() {
        for good in ["ctrl+s", "Cmd+Shift+P", "ctrl+k ctrl+s", "escape", "f12", "wheelup"] {
            let v = validate_chord(good);
            assert!(v.valid, "{good}: {:?}", v.issues);
        }
        for bad in ["", "hyper+k", "ctrl+notakey"] {
            assert!(!validate_chord(bad).valid, "expected '{bad}' to fail");
        }
    }

    #[test]
    fn reserved_shortcuts_are_refused() {
        let td = tempfile::tempdir().unwrap();
        assert!(!validate_chord("ctrl+c").valid);
        let bindings = vec![Keybinding {
            context: "Chat".into(),
            key: "Ctrl+D".into(),
            action: Some("chat:submit".into()),
        }];
        assert!(put(td.path(), &bindings, None).is_err());
    }

    #[test]
    fn duplicates_and_unknown_contexts_are_reported() {
        let bindings = vec![
            Keybinding { context: "Chat".into(), key: "ctrl+e".into(), action: Some("a".into()) },
            Keybinding { context: "Chat".into(), key: "Ctrl+E".into(), action: Some("b".into()) },
            Keybinding { context: "Nowhere".into(), key: "ctrl+e".into(), action: Some("c".into()) },
        ];
        assert_eq!(duplicates(&bindings).len(), 1, "case differs but the chord is the same");
        assert_eq!(unknown_contexts(&bindings), vec!["Nowhere"]);
    }

    #[test]
    fn raw_put_validates_and_backs_up() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(path(td.path()), DOC).unwrap();
        assert!(raw_put(td.path(), "{ nope", None).is_err());
        assert!(raw_put(td.path(), "[]", None).is_err());

        raw_put(td.path(), r#"{"bindings":[]}"#, None).unwrap();
        assert!(td.path().join("keybindings.json.ccg.bak").is_file());
    }
}
