//! Read-only view of `~/.claude/jobs/`, the CLI's background-job store.
//!
//! Layout: `jobs/<8-hex id>/{state.json, timeline.jsonl}` plus a top-level
//! `jobs/pins.json` listing pinned ids. This app never writes here.
//!
//! `state.json` holds `providerEnv`, a map of provider credentials. Only its
//! key names leave this module — see `Job::provider_env_keys`.

use std::path::{Path, PathBuf};

use crate::types::{Job, JobDetail, JobLink, TimelineEvent};
use crate::AppError;

const JOBS_SUBDIR: &str = "jobs";
const STATE_FILE: &str = "state.json";
const TIMELINE_FILE: &str = "timeline.jsonl";
const PINS_FILE: &str = "pins.json";

pub fn list(claude_dir: &Path) -> Result<Vec<Job>, AppError> {
    let root = claude_dir.join(JOBS_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let pins = read_pins(&root);
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        match read_state(&dir, &pins) {
            Ok(Some(job)) => out.push(job),
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = %e, path = %dir.display(), "skipping unreadable job")
            }
        }
    }
    // Most recently touched first — that is what a job list is for.
    out.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(out)
}

pub fn get(claude_dir: &Path, job_id: &str) -> Result<JobDetail, AppError> {
    let dir = job_dir(claude_dir, job_id)?;
    let pins = read_pins(&claude_dir.join(JOBS_SUBDIR));
    let job = read_state(&dir, &pins)?
        .ok_or_else(|| AppError::not_found(format!("job '{job_id}' not found")))?;
    let (timeline, malformed_lines) = read_timeline(&dir.join(TIMELINE_FILE));
    Ok(JobDetail {
        job,
        timeline,
        malformed_lines,
    })
}

/// Job ids are 8 lowercase hex chars. Validating before joining keeps a crafted
/// id from walking out of the jobs directory.
fn job_dir(claude_dir: &Path, job_id: &str) -> Result<PathBuf, AppError> {
    let valid = job_id.len() == 8 && job_id.chars().all(|c| c.is_ascii_hexdigit());
    if !valid {
        return Err(AppError::invalid(format!("invalid job id '{job_id}'")));
    }
    Ok(claude_dir.join(JOBS_SUBDIR).join(job_id))
}

fn read_pins(root: &Path) -> Vec<String> {
    let path = root.join(PINS_FILE);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return vec![];
    };
    serde_json::from_str::<Vec<String>>(&raw).unwrap_or_default()
}

/// `Ok(None)` when the directory has no `state.json` — the jobs root also holds
/// `pins.json` and per-job `tmp/` scratch.
fn read_state(dir: &Path, pins: &[String]) -> Result<Option<Job>, AppError> {
    let path = dir.join(STATE_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;

    let id = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    Ok(Some(Job {
        name: str_field(&v, "name").unwrap_or_else(|| id.clone()),
        name_source: str_field(&v, "nameSource"),
        state: str_field(&v, "state").unwrap_or_else(|| "unknown".into()),
        detail: str_field(&v, "detail"),
        intent: str_field(&v, "intent"),
        needs: str_field(&v, "needs"),
        backend: str_field(&v, "backend"),
        template: str_field(&v, "template"),
        tempo: str_field(&v, "tempo"),
        cwd: str_field(&v, "cwd"),
        created_at: str_field(&v, "createdAt"),
        updated_at: str_field(&v, "updatedAt"),
        first_terminal_at: str_field(&v, "firstTerminalAt"),
        reaped_mid_work_at: str_field(&v, "reapedMidWorkAt"),
        session_id: str_field(&v, "sessionId"),
        resume_session_id: str_field(&v, "resumeSessionId"),
        children: parse_children(&v),
        output: v
            .get("output")
            .filter(|o| !o.is_null())
            .map(|o| serde_json::to_string_pretty(o).unwrap_or_default()),
        tokens: v.get("tokens").and_then(|t| t.as_i64()),
        pinned: pins.iter().any(|p| p == &id),
        provider_env_keys: provider_env_keys(&v),
        id,
    }))
}

/// Key names only. The values are provider credentials and must not reach the
/// frontend.
fn provider_env_keys(v: &serde_json::Value) -> Vec<String> {
    let mut keys: Vec<String> = v
        .get("providerEnv")
        .and_then(|e| e.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    keys.sort();
    keys
}

fn parse_children(v: &serde_json::Value) -> Vec<JobLink> {
    v.get("children")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    Some(JobLink {
                        id: str_field(c, "id")?,
                        href: str_field(c, "href"),
                        kind: str_field(c, "kind"),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Returns the parsed events and the number of lines that could not be read.
/// A missing file is normal — a job that never reported progress has none.
fn read_timeline(path: &Path) -> (Vec<TimelineEvent>, usize) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return (vec![], 0);
    };
    let mut events = Vec::new();
    let mut malformed = 0usize;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => events.push(TimelineEvent {
                at: str_field(&v, "at"),
                state: str_field(&v, "state"),
                detail: str_field(&v, "detail"),
                text: str_field(&v, "text"),
            }),
            Err(e) => {
                malformed += 1;
                tracing::warn!(error = %e, path = %path.display(), "skipping timeline line");
            }
        }
    }
    (events, malformed)
}

fn str_field(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "sk-do-not-leak";

    fn write_job(claude_dir: &Path, id: &str, state: &str, timeline: Option<&str>) {
        let dir = claude_dir.join(JOBS_SUBDIR).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(STATE_FILE), state).unwrap();
        if let Some(t) = timeline {
            std::fs::write(dir.join(TIMELINE_FILE), t).unwrap();
        }
    }

    fn full_state() -> String {
        format!(
            r#"{{
              "name": "Layerzero-research PR 25",
              "nameSource": "auto",
              "state": "done",
              "detail": "PR #25 merged",
              "intent": "review the PR",
              "backend": "daemon",
              "cwd": "/Users/x/proj",
              "createdAt": "2026-08-22T02:59:24.590Z",
              "updatedAt": "2026-08-22T03:07:42.272Z",
              "sessionId": "abc",
              "tokens": 17267,
              "children": [{{"id": "25", "href": "https://example.com/pull/25", "kind": "pr"}}],
              "output": {{"result": "merged"}},
              "providerEnv": {{"ANTHROPIC_API_KEY": "{SECRET}", "AWS_PROFILE": "x"}}
            }}"#
        )
    }

    #[test]
    fn provider_env_values_never_serialized() {
        let td = tempfile::tempdir().unwrap();
        write_job(td.path(), "ea3c1e27", &full_state(), None);

        let job = &list(td.path()).unwrap()[0];
        assert_eq!(job.provider_env_keys, vec!["ANTHROPIC_API_KEY", "AWS_PROFILE"]);

        let wire = serde_json::to_string(job).unwrap();
        assert!(!wire.contains(SECRET), "credential crossed the IPC boundary");
        // The key names are deliberate; the map itself must not be echoed.
        assert!(wire.contains("providerEnvKeys"));
        assert!(!wire.contains(r#""providerEnv":"#));
    }

    #[test]
    fn parses_children_output_and_tokens() {
        let td = tempfile::tempdir().unwrap();
        write_job(td.path(), "ea3c1e27", &full_state(), None);
        let job = &list(td.path()).unwrap()[0];
        assert_eq!(job.tokens, Some(17267));
        assert_eq!(job.children.len(), 1);
        assert_eq!(job.children[0].kind.as_deref(), Some("pr"));
        assert!(job.output.as_deref().unwrap().contains("merged"));
        assert!(!job.pinned);
    }

    #[test]
    fn list_sorted_by_updated_at_desc() {
        let td = tempfile::tempdir().unwrap();
        write_job(td.path(), "aaaaaaaa", r#"{"updatedAt":"2026-01-01T00:00:00Z"}"#, None);
        write_job(td.path(), "bbbbbbbb", r#"{"updatedAt":"2026-06-01T00:00:00Z"}"#, None);
        let jobs = list(td.path()).unwrap();
        assert_eq!(jobs[0].id, "bbbbbbbb");
        // A state.json without `name`/`state` still lists, using safe fallbacks.
        assert_eq!(jobs[0].name, "bbbbbbbb");
        assert_eq!(jobs[0].state, "unknown");
    }

    #[test]
    fn pins_json_marks_pinned() {
        let td = tempfile::tempdir().unwrap();
        write_job(td.path(), "ea3c1e27", &full_state(), None);
        std::fs::write(
            td.path().join(JOBS_SUBDIR).join(PINS_FILE),
            r#"["ea3c1e27"]"#,
        )
        .unwrap();
        assert!(list(td.path()).unwrap()[0].pinned);
    }

    #[test]
    fn malformed_timeline_line_is_counted_not_fatal() {
        let td = tempfile::tempdir().unwrap();
        write_job(
            td.path(),
            "4cd39432",
            &full_state(),
            Some("{\"at\":\"t1\",\"state\":\"running\"}\n{ torn\n\n{\"at\":\"t2\",\"state\":\"done\"}\n"),
        );
        let detail = get(td.path(), "4cd39432").unwrap();
        assert_eq!(detail.timeline.len(), 2);
        assert_eq!(detail.malformed_lines, 1);
    }

    #[test]
    fn missing_timeline_is_not_an_error() {
        let td = tempfile::tempdir().unwrap();
        write_job(td.path(), "ea3c1e27", &full_state(), None);
        let detail = get(td.path(), "ea3c1e27").unwrap();
        assert!(detail.timeline.is_empty());
        assert_eq!(detail.malformed_lines, 0);
    }

    #[test]
    fn dir_without_state_json_is_skipped() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join(JOBS_SUBDIR).join("deadbeef")).unwrap();
        assert!(list(td.path()).unwrap().is_empty());
    }

    #[test]
    fn job_id_traversal_rejected() {
        let td = tempfile::tempdir().unwrap();
        assert!(get(td.path(), "../../etc").is_err());
        assert!(get(td.path(), "not-hex!").is_err());
        assert!(get(td.path(), "abc").is_err()); // too short
        assert!(get(td.path(), "deadbeef").is_err()); // well-formed but absent
    }

    #[test]
    fn missing_jobs_dir_is_empty() {
        let td = tempfile::tempdir().unwrap();
        assert!(list(td.path()).unwrap().is_empty());
    }
}
