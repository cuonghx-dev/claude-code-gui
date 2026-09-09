//! Read-only view of `~/.claude/teams/`. Each `<session-id>/config.json`
//! describes one agent team: a lead plus the members it spawned.
//!
//! Written by the CLI's agent-teams feature; this app never writes here.

use std::path::Path;

use crate::types::{Team, TeamMember};
use crate::AppError;

const TEAMS_SUBDIR: &str = "teams";
const CONFIG_FILE: &str = "config.json";

pub fn list(claude_dir: &Path) -> Result<Vec<Team>, AppError> {
    let root = claude_dir.join(TEAMS_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        match read_one(&dir) {
            Ok(Some(team)) => out.push(team),
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = %e, path = %dir.display(), "skipping unreadable team")
            }
        }
    }
    // Newest first: a stale team from months ago is rarely what you want.
    out.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms).then(a.id.cmp(&b.id)));
    Ok(out)
}

pub fn get(claude_dir: &Path, id: &str) -> Result<Team, AppError> {
    crate::io::validate_slug(id)?;
    let dir = claude_dir.join(TEAMS_SUBDIR).join(id);
    read_one(&dir)?.ok_or_else(|| AppError::not_found(format!("team '{id}' not found")))
}

/// `Ok(None)` when the directory holds no `config.json` — teams share the
/// `~/.claude` namespace with scratch dirs, so absence is not an error.
fn read_one(dir: &Path) -> Result<Option<Team>, AppError> {
    let path = dir.join(CONFIG_FILE);
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

    let members = v
        .get("members")
        .and_then(|m| m.as_array())
        .map(|arr| arr.iter().filter_map(parse_member).collect())
        .unwrap_or_default();

    Ok(Some(Team {
        name: str_field(&v, "name").unwrap_or_else(|| id.clone()),
        created_at_ms: v.get("createdAt").and_then(|x| x.as_i64()).unwrap_or(0),
        lead_agent_id: str_field(&v, "leadAgentId"),
        lead_session_id: str_field(&v, "leadSessionId"),
        members,
        id,
    }))
}

/// A member without an `agentId` has nothing to key on; drop it rather than
/// inventing an identity.
fn parse_member(v: &serde_json::Value) -> Option<TeamMember> {
    let agent_id = str_field(v, "agentId")?;
    Some(TeamMember {
        agent_id,
        name: str_field(v, "name"),
        agent_type: str_field(v, "agentType"),
        joined_at_ms: v.get("joinedAt").and_then(|x| x.as_i64()),
        tmux_pane_id: str_field(v, "tmuxPaneId"),
        cwd: str_field(v, "cwd"),
        subscriptions: v
            .get("subscriptions")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        backend_type: str_field(v, "backendType"),
    })
}

fn str_field(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_team(claude_dir: &Path, id: &str, body: &str) {
        let dir = claude_dir.join(TEAMS_SUBDIR).join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(CONFIG_FILE), body).unwrap();
    }

    const FULL: &str = r#"{
      "name": "session-3a5d9134",
      "createdAt": 1786952017216,
      "leadAgentId": "team-lead@session-3a5d9134",
      "leadSessionId": "3a5d9134-d900-4227-ae65-d7f4a01d07b1",
      "members": [{
        "agentId": "team-lead@session-3a5d9134",
        "name": "lead",
        "agentType": "claude",
        "joinedAt": 1786952017220,
        "tmuxPaneId": "%12",
        "cwd": "/Users/x/proj",
        "subscriptions": ["all"],
        "backendType": "tmux"
      }]
    }"#;

    #[test]
    fn list_parses_members() {
        let td = tempfile::tempdir().unwrap();
        write_team(td.path(), "session-3a5d9134", FULL);

        let teams = list(td.path()).unwrap();
        assert_eq!(teams.len(), 1);
        let t = &teams[0];
        assert_eq!(t.id, "session-3a5d9134");
        assert_eq!(t.created_at_ms, 1786952017216); // epoch ms preserved, not stringified
        assert_eq!(t.members.len(), 1);
        assert_eq!(t.members[0].subscriptions, vec!["all"]);
        assert_eq!(t.members[0].backend_type.as_deref(), Some("tmux"));
    }

    #[test]
    fn missing_teams_dir_is_empty_not_an_error() {
        let td = tempfile::tempdir().unwrap();
        assert!(list(td.path()).unwrap().is_empty());
    }

    #[test]
    fn dir_without_config_is_skipped() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join(TEAMS_SUBDIR).join("session-empty")).unwrap();
        write_team(td.path(), "session-ok", FULL);
        let teams = list(td.path()).unwrap();
        assert_eq!(teams.len(), 1);
        assert_eq!(teams[0].id, "session-ok");
    }

    #[test]
    fn optional_member_keys_tolerated() {
        let td = tempfile::tempdir().unwrap();
        write_team(
            td.path(),
            "session-min",
            r#"{"members":[{"agentId":"a"},{"name":"no-id"}]}"#,
        );
        let t = &list(td.path()).unwrap()[0];
        assert_eq!(t.name, "session-min"); // falls back to the directory name
        assert_eq!(t.created_at_ms, 0);
        // The member with no agentId is dropped; the bare one survives.
        assert_eq!(t.members.len(), 1);
        assert!(t.members[0].cwd.is_none());
        assert!(t.members[0].subscriptions.is_empty());
    }

    #[test]
    fn malformed_config_does_not_abort_the_listing() {
        let td = tempfile::tempdir().unwrap();
        write_team(td.path(), "session-bad", "{not json");
        write_team(td.path(), "session-ok", FULL);
        let teams = list(td.path()).unwrap();
        assert_eq!(teams.len(), 1);
    }

    #[test]
    fn list_sorted_newest_first() {
        let td = tempfile::tempdir().unwrap();
        write_team(td.path(), "session-old", r#"{"createdAt":1000,"members":[]}"#);
        write_team(td.path(), "session-new", r#"{"createdAt":2000,"members":[]}"#);
        let teams = list(td.path()).unwrap();
        assert_eq!(teams[0].id, "session-new");
    }

    #[test]
    fn get_rejects_path_traversal() {
        let td = tempfile::tempdir().unwrap();
        assert!(get(td.path(), "../../etc").is_err());
        assert!(get(td.path(), "session-missing").is_err());
    }
}
