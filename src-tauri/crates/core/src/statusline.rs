//! The `statusLine` settings key, plus a one-shot test run.
//!
//! Claude Code invokes the configured command on every render and feeds it a
//! JSON payload on stdin. `preview` reproduces that once, on demand, so a
//! person can see what their command prints before committing to it — nothing
//! here runs automatically.

use std::path::Path;
use std::time::Duration;

use serde_json::json;

use crate::exec;
use crate::settings_scope;
use crate::types::{SettingsScope, StatusLine, StatusLinePreview};
use crate::AppError;

/// Claude Code renders the status line constantly, so a slow command is a
/// problem the person should see immediately.
const PREVIEW_TIMEOUT: Duration = Duration::from_secs(5);

pub fn get(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
) -> Result<StatusLine, AppError> {
    let path = settings_scope::scope_path(claude_dir, scope, working_dir)?;
    let doc = crate::io::read_json_doc(&path)?;
    Ok(doc
        .value
        .get("statusLine")
        .cloned()
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default())
}

pub fn put(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
    status_line: &StatusLine,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    if status_line
        .command
        .as_deref()
        .map(str::trim)
        .is_none_or(str::is_empty)
    {
        return Err(AppError::invalid("a status line needs a command"));
    }
    let mut value = serde_json::to_value(status_line)?;
    if let Some(obj) = value.as_object_mut() {
        obj.entry("type".to_string())
            .or_insert_with(|| json!("command"));
    }
    settings_scope::patch(
        claude_dir,
        scope,
        working_dir,
        &json!({ "statusLine": value }),
        expected_mtime_ms,
    )
}

pub fn delete(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    settings_scope::patch(
        claude_dir,
        scope,
        working_dir,
        &json!({ "statusLine": serde_json::Value::Null }),
        expected_mtime_ms,
    )
}

/// Run the command once with a representative payload on stdin.
///
/// The payload mirrors what Claude Code sends; values are taken from real
/// state where this app has it and are plausible placeholders otherwise, so
/// the command sees the shape it will get in practice.
pub fn preview(
    claude_dir: &Path,
    working_dir: Option<&Path>,
    status_line: &StatusLine,
) -> Result<StatusLinePreview, AppError> {
    let command = status_line
        .command
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .ok_or_else(|| AppError::invalid("no command to run"))?;

    let cwd = working_dir.unwrap_or(claude_dir);
    let payload = json!({
        "hook_event_name": "Status",
        "session_id": "00000000-0000-0000-0000-000000000000",
        "transcript_path": "",
        "cwd": cwd.to_string_lossy(),
        "model": { "id": "claude-opus-5", "display_name": "Opus" },
        "workspace": {
            "current_dir": cwd.to_string_lossy(),
            "project_dir": cwd.to_string_lossy(),
        },
        "version": env!("CARGO_PKG_VERSION"),
        "output_style": { "name": "default" },
    });

    let result = exec::run_with_timeout(
        command,
        cwd,
        &serde_json::to_string(&payload)?,
        PREVIEW_TIMEOUT,
    )?;

    Ok(StatusLinePreview {
        stdout: result.stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
        duration_ms: result.duration_ms,
        timed_out: result.timed_out,
        truncated: result.truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn line(command: &str) -> StatusLine {
        StatusLine {
            kind: None,
            command: Some(command.into()),
            padding: Some(0),
            extra: Default::default(),
        }
    }

    #[test]
    fn put_and_get_round_trip_without_disturbing_siblings() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"model":"opus","permissions":{"allow":["Bash(ls:*)"]}}"#,
        )
        .unwrap();

        put(td.path(), SettingsScope::User, None, &line("~/bin/statusline"), None).unwrap();
        let got = get(td.path(), SettingsScope::User, None).unwrap();
        assert_eq!(got.command.as_deref(), Some("~/bin/statusline"));
        // `type` defaults to command so Claude Code accepts the block.
        assert_eq!(got.kind.as_deref(), Some("command"));

        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(td.path().join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(v["model"], "opus");
        assert_eq!(v["permissions"]["allow"][0], "Bash(ls:*)");
    }

    #[test]
    fn delete_removes_the_key_entirely() {
        let td = tempfile::tempdir().unwrap();
        put(td.path(), SettingsScope::User, None, &line("x"), None).unwrap();
        delete(td.path(), SettingsScope::User, None, None).unwrap();
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(td.path().join("settings.json")).unwrap())
                .unwrap();
        assert!(v.get("statusLine").is_none());
    }

    #[test]
    fn an_empty_command_is_rejected() {
        let td = tempfile::tempdir().unwrap();
        assert!(put(td.path(), SettingsScope::User, None, &line("  "), None).is_err());
        assert!(preview(td.path(), None, &line("")).is_err());
    }

    #[test]
    fn preview_feeds_the_payload_on_stdin() {
        let td = tempfile::tempdir().unwrap();
        let r = preview(td.path(), None, &line("cat")).unwrap();
        let echoed: Value = serde_json::from_str(&r.stdout).unwrap();
        assert_eq!(echoed["hook_event_name"], "Status");
        assert!(echoed["workspace"]["current_dir"].is_string());
        assert_eq!(r.exit_code, Some(0));
        assert!(!r.timed_out);
    }

    #[test]
    fn preview_reports_a_failing_command_rather_than_erroring() {
        let td = tempfile::tempdir().unwrap();
        let r = preview(td.path(), None, &line("echo broken >&2; exit 1")).unwrap();
        assert_eq!(r.exit_code, Some(1));
        assert_eq!(r.stderr.trim(), "broken");
    }

    #[test]
    fn preview_kills_a_hanging_command() {
        let td = tempfile::tempdir().unwrap();
        // The 5s cap is what protects the UI from a command that never returns.
        let r = preview(td.path(), None, &line("sleep 30")).unwrap();
        assert!(r.timed_out);
        assert!(r.duration_ms < 10_000);
    }
}
