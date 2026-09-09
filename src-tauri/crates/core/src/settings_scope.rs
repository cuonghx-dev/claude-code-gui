//! Scoped access to Claude Code's settings files.
//!
//! Precedence, highest first (per the settings docs): managed settings, then
//! `--settings` (a CLI flag, not a file this app owns), then project-local
//! `.claude/settings.local.json`, then shared `.claude/settings.json`, then
//! user `~/.claude/settings.json`.
//!
//! Two behaviors from the docs shape this module:
//!
//! * **Lists merge instead of overriding.** A key such as `permissions.allow`
//!   set in several files is the union of them all, not the highest file's
//!   value. `effective()` reproduces that.
//! * **Settings files are strict JSON** — a comment or trailing comma is a
//!   syntax error to Claude Code — so structured writes cannot destroy
//!   comments, because there are never any to destroy.
//!
//! Writes are RFC 7386 merge patches over the file's parsed JSON. The write
//! path never deserializes into a typed struct, so a key this app has never
//! heard of cannot be dropped by round-tripping through one.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::io;
use crate::types::{EffectiveEntry, RawDoc, ScopeInfo, SettingsScope};
use crate::{AppError, ErrorCode};

const SETTINGS_FILE: &str = "settings.json";
const LOCAL_SETTINGS_FILE: &str = "settings.local.json";
const CLAUDE_SUBDIR: &str = ".claude";

/// List keys that merge across scopes rather than overriding.
///
/// The docs call out `permissions.*` lists; the model-list keys have their own
/// rules and are deliberately not merged here.
const MERGING_KEYS: &[&str] = &["permissions"];

pub fn managed_path() -> PathBuf {
    // Verified against the managed-settings docs. The legacy Windows
    // ProgramData path is deliberately not read — Claude Code dropped it.
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json")
    }
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Program Files\ClaudeCode\managed-settings.json")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        PathBuf::from("/etc/claude-code/managed-settings.json")
    }
}

pub fn scope_path(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
) -> Result<PathBuf, AppError> {
    Ok(match scope {
        SettingsScope::Managed => managed_path(),
        SettingsScope::User => claude_dir.join(SETTINGS_FILE),
        SettingsScope::Project => working_dir
            .ok_or_else(|| AppError::invalid("project scope requires a working directory"))?
            .join(CLAUDE_SUBDIR)
            .join(SETTINGS_FILE),
        // Project-local is the documented location. A user-level
        // settings.local.json is undocumented but does occur on real machines,
        // so it is what `Local` resolves to when no project is selected.
        SettingsScope::Local => match working_dir {
            Some(wd) => wd.join(CLAUDE_SUBDIR).join(LOCAL_SETTINGS_FILE),
            None => claude_dir.join(LOCAL_SETTINGS_FILE),
        },
    })
}

pub fn scopes(claude_dir: &Path, working_dir: Option<&Path>) -> Vec<ScopeInfo> {
    SettingsScope::ALL
        .iter()
        .filter_map(|&scope| {
            let path = scope_path(claude_dir, scope, working_dir).ok()?;
            Some(ScopeInfo {
                scope,
                exists: path.is_file(),
                writable: scope.writable(),
                mtime_ms: io::mtime_ms(&path),
                path: path.to_string_lossy().into_owned(),
            })
        })
        .collect()
}

pub fn raw_get(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
) -> Result<RawDoc, AppError> {
    let path = scope_path(claude_dir, scope, working_dir)?;
    let exists = path.is_file();
    let content = if exists {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };
    Ok(RawDoc {
        scope,
        exists,
        mtime_ms: io::mtime_ms(&path),
        content,
        path: path.to_string_lossy().into_owned(),
    })
}

pub fn raw_put(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
    content: &str,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let path = writable_path(claude_dir, scope, working_dir)?;
    // Parse before writing: a settings file Claude Code cannot read is worse
    // than a rejected save.
    let value: Value = serde_json::from_str(content)?;
    if !value.is_object() {
        return Err(AppError::invalid("settings must be a JSON object"));
    }
    io::check_unchanged(&path, expected_mtime_ms)?;
    io::backup_sibling(&path)?;
    io::atomic_write(&path, content.as_bytes())?;
    Ok(io::mtime_ms(&path).unwrap_or(0))
}

/// Apply an RFC 7386 merge patch. `null` deletes a key; arrays replace
/// wholesale.
pub fn patch(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
    patch: &Value,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let path = writable_path(claude_dir, scope, working_dir)?;
    let mut doc = io::read_json_doc(&path)?;
    if !doc.value.is_object() {
        doc.value = Value::Object(Default::default());
    }
    merge_patch(&mut doc.value, patch);
    io::atomic_write_json(&path, &doc.value, expected_mtime_ms)
}

/// The merged view: one row per top-level key, naming the file it came from.
pub fn effective(
    claude_dir: &Path,
    working_dir: Option<&Path>,
) -> Result<Vec<EffectiveEntry>, AppError> {
    let mut entries: Vec<EffectiveEntry> = Vec::new();

    // Highest precedence first, so the first scope to define a key wins.
    for &scope in SettingsScope::ALL.iter() {
        let Ok(path) = scope_path(claude_dir, scope, working_dir) else {
            continue;
        };
        let Ok(doc) = io::read_json_doc(&path) else {
            // A broken settings file is reported by Claude Code itself; here it
            // just contributes nothing.
            continue;
        };
        let Some(obj) = doc.value.as_object() else {
            continue;
        };
        for (key, value) in obj {
            match entries.iter_mut().find(|e| &e.key == key) {
                Some(existing) => {
                    existing.overridden.push(scope);
                    if MERGING_KEYS.contains(&key.as_str()) {
                        union_lists(&mut existing.value, value);
                        existing.merged = true;
                    }
                }
                None => entries.push(EffectiveEntry {
                    key: key.clone(),
                    value: value.clone(),
                    source: scope,
                    overridden: vec![],
                    merged: false,
                }),
            }
        }
    }

    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(entries)
}

fn writable_path(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
) -> Result<PathBuf, AppError> {
    if !scope.writable() {
        return Err(AppError::new(
            ErrorCode::PermissionDenied,
            "managed settings are deployed by your organization and cannot be edited here",
        ));
    }
    scope_path(claude_dir, scope, working_dir)
}

/// RFC 7386. `null` in the patch deletes; objects recurse; everything else,
/// arrays included, replaces.
pub fn merge_patch(target: &mut Value, patch: &Value) {
    let Some(patch_obj) = patch.as_object() else {
        *target = patch.clone();
        return;
    };
    if !target.is_object() {
        *target = Value::Object(Default::default());
    }
    let target_obj = target.as_object_mut().expect("just ensured object");
    for (key, value) in patch_obj {
        if value.is_null() {
            target_obj.remove(key);
        } else if value.is_object() {
            let slot = target_obj
                .entry(key.clone())
                .or_insert_with(|| Value::Object(Default::default()));
            merge_patch(slot, value);
        } else {
            target_obj.insert(key.clone(), value.clone());
        }
    }
}

/// Union the array members of two objects, keeping order and dropping exact
/// duplicates. Used for `permissions`, whose lists merge across scopes.
fn union_lists(into: &mut Value, from: &Value) {
    let (Some(into_obj), Some(from_obj)) = (into.as_object_mut(), from.as_object()) else {
        return;
    };
    for (key, value) in from_obj {
        match (into_obj.get_mut(key), value.as_array()) {
            (Some(Value::Array(existing)), Some(incoming)) => {
                for item in incoming {
                    if !existing.contains(item) {
                        existing.push(item.clone());
                    }
                }
            }
            (None, _) => {
                into_obj.insert(key.clone(), value.clone());
            }
            // A scalar such as `defaultMode` keeps the higher scope's value.
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write(path: &Path, v: Value) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    }

    #[test]
    fn scope_paths_resolve() {
        let td = tempfile::tempdir().unwrap();
        let wd = td.path().join("proj");

        assert_eq!(
            scope_path(td.path(), SettingsScope::User, None).unwrap(),
            td.path().join("settings.json")
        );
        assert_eq!(
            scope_path(td.path(), SettingsScope::Project, Some(&wd)).unwrap(),
            wd.join(".claude/settings.json")
        );
        assert_eq!(
            scope_path(td.path(), SettingsScope::Local, Some(&wd)).unwrap(),
            wd.join(".claude/settings.local.json")
        );
        // Local with no project falls back to the user-level file, which is
        // undocumented but real.
        assert_eq!(
            scope_path(td.path(), SettingsScope::Local, None).unwrap(),
            td.path().join("settings.local.json")
        );
        // Project scope needs a working directory.
        assert!(scope_path(td.path(), SettingsScope::Project, None).is_err());
    }

    #[test]
    fn patch_preserves_unknown_keys_and_order() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{"zeta":1,"model":"opus","someFutureKey":{"a":1},"hooks":{"PreToolUse":[]}}"#,
        )
        .unwrap();

        patch(
            td.path(),
            SettingsScope::User,
            None,
            &json!({"model": "sonnet"}),
            None,
        )
        .unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["model"], "sonnet");
        assert_eq!(v["someFutureKey"]["a"], 1, "unknown key survived the write");
        assert!(v["hooks"].is_object());
        // preserve_order keeps the user's key order rather than alphabetizing.
        assert!(raw.find("zeta").unwrap() < raw.find("model").unwrap());
    }

    #[test]
    fn patch_null_deletes_and_objects_recurse() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().join("settings.json");
        write(&path, json!({"statusLine": {"type": "command"}, "model": "opus"}));

        patch(
            td.path(),
            SettingsScope::User,
            None,
            &json!({"statusLine": null, "permissions": {"allow": ["Bash(ls:*)"]}}),
            None,
        )
        .unwrap();

        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(v.get("statusLine").is_none());
        assert_eq!(v["model"], "opus");
        assert_eq!(v["permissions"]["allow"][0], "Bash(ls:*)");
    }

    #[test]
    fn patch_creates_a_missing_file_and_its_parents() {
        let td = tempfile::tempdir().unwrap();
        let wd = td.path().join("proj");
        patch(
            td.path(),
            SettingsScope::Project,
            Some(&wd),
            &json!({"model": "opus"}),
            None,
        )
        .unwrap();
        assert!(wd.join(".claude/settings.json").is_file());
    }

    #[test]
    fn arrays_replace_wholesale_within_one_file() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().join("settings.json");
        write(&path, json!({"permissions": {"allow": ["a", "b"]}}));
        patch(
            td.path(),
            SettingsScope::User,
            None,
            &json!({"permissions": {"allow": ["c"]}}),
            None,
        )
        .unwrap();
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["permissions"]["allow"], json!(["c"]));
    }

    #[test]
    fn managed_scope_is_read_only() {
        let td = tempfile::tempdir().unwrap();
        let err = patch(td.path(), SettingsScope::Managed, None, &json!({}), None).unwrap_err();
        assert_eq!(err.code, ErrorCode::PermissionDenied);
        let err = raw_put(td.path(), SettingsScope::Managed, None, "{}", None).unwrap_err();
        assert_eq!(err.code, ErrorCode::PermissionDenied);

        let infos = scopes(td.path(), None);
        let managed = infos
            .iter()
            .find(|s| s.scope == SettingsScope::Managed)
            .unwrap();
        assert!(!managed.writable);
    }

    #[test]
    fn effective_reports_precedence_and_merges_permission_lists() {
        let td = tempfile::tempdir().unwrap();
        let wd = td.path().join("proj");
        write(
            &td.path().join("settings.json"),
            json!({"model": "user-model", "userOnly": 1, "permissions": {"allow": ["A"]}}),
        );
        write(
            &wd.join(".claude/settings.json"),
            json!({"model": "project-model", "permissions": {"allow": ["B"], "defaultMode": "plan"}}),
        );
        write(
            &wd.join(".claude/settings.local.json"),
            json!({"model": "local-model", "permissions": {"allow": ["C"]}}),
        );

        let eff = effective(td.path(), Some(&wd)).unwrap();
        let by_key = |k: &str| eff.iter().find(|e| e.key == k).unwrap().clone();

        // Project local sits above shared project, which sits above user.
        let model = by_key("model");
        assert_eq!(model.value, "local-model");
        assert_eq!(model.source, SettingsScope::Local);
        assert_eq!(
            model.overridden,
            vec![SettingsScope::Project, SettingsScope::User]
        );

        // Lists merge instead of overriding, so every scope's rules apply.
        let perms = by_key("permissions");
        assert!(perms.merged);
        let allow = perms.value["allow"].as_array().unwrap();
        assert_eq!(allow.len(), 3);
        assert!(allow.contains(&json!("A")) && allow.contains(&json!("C")));
        // A scalar inside a merging key still follows precedence.
        assert_eq!(perms.value["defaultMode"], "plan");

        assert_eq!(by_key("userOnly").source, SettingsScope::User);
    }

    #[test]
    fn raw_put_rejects_invalid_json_without_touching_the_file() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().join("settings.json");
        write(&path, json!({"model": "opus"}));

        assert!(raw_put(td.path(), SettingsScope::User, None, "{ nope", None).is_err());
        assert!(raw_put(td.path(), SettingsScope::User, None, "[]", None).is_err());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["model"], "opus");
    }

    #[test]
    fn writes_reject_a_stale_mtime() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().join("settings.json");
        write(&path, json!({"model": "opus"}));
        let mtime = io::mtime_ms(&path).unwrap();

        let err = patch(
            td.path(),
            SettingsScope::User,
            None,
            &json!({"model": "x"}),
            Some(mtime - 5),
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::Conflict);
    }

    #[test]
    fn missing_files_read_as_empty() {
        let td = tempfile::tempdir().unwrap();
        let doc = raw_get(td.path(), SettingsScope::User, None).unwrap();
        assert!(!doc.exists);
        assert!(doc.content.is_empty());
        assert!(effective(td.path(), None).unwrap().is_empty());
    }
}
