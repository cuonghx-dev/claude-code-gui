//! Checkpoints: the join between a session's transcript and the file blobs
//! under `~/.claude/file-history/<session-id>/`.
//!
//! The transcript carries the index — `file-history-snapshot` records mark a
//! checkpoint, `file-history-delta` records name a tracked file, the blob that
//! holds its previous contents (`backup.backupFileName`, e.g.
//! `0b642a3eaf179c2b@v1`) and which snapshot it belongs to
//! (`snapshotMessageId`). The blobs themselves are the raw file bytes, with no
//! metadata of their own.
//!
//! Diffing happens here rather than in the frontend: shipping two full file
//! blobs over IPC for every comparison would be far more traffic than the
//! hunks that come back.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::transcript_scan;
use crate::types::{
    Checkpoint, CheckpointFile, DiffHunk, DiffLine, DiffResult, DiffSide, DiffTag,
};
use crate::AppError;

const PROJECTS_SUBDIR: &str = "projects";
const FILE_HISTORY_SUBDIR: &str = "file-history";

/// Above this, a side is treated as too large to diff.
const MAX_DIFF_BYTES: u64 = 2 * 1024 * 1024;
/// Emitted diff lines, after which the result is marked truncated.
const MAX_DIFF_LINES: usize = 2000;
/// Lines of unchanged context kept around each change.
const DIFF_CONTEXT: usize = 3;

pub fn list(
    claude_dir: &Path,
    project_name: &str,
    session_id: &str,
) -> Result<Vec<Checkpoint>, AppError> {
    let transcript = claude_dir
        .join(PROJECTS_SUBDIR)
        .join(project_name)
        .join(format!("{session_id}.jsonl"));
    if !transcript.is_file() {
        return Err(AppError::not_found(format!(
            "session '{session_id}' not found"
        )));
    }
    let blob_dir = blob_dir(claude_dir, session_id)?;

    // snapshot message id -> checkpoint
    let mut checkpoints: BTreeMap<String, Checkpoint> = BTreeMap::new();

    transcript_scan::for_each_line(&transcript, |_, line| {
        // Cheap prefilter: most lines are conversation.
        if !line.contains("file-history-") {
            return true;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            return true;
        };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("file-history-snapshot") => {
                let Some(id) = str_field(&v, "messageId") else {
                    return true;
                };
                let ts = v
                    .get("snapshot")
                    .and_then(|s| s.get("timestamp"))
                    .and_then(|t| t.as_str())
                    .map(str::to_string);
                checkpoints.entry(id.clone()).or_insert(Checkpoint {
                    message_id: id,
                    timestamp: ts,
                    files: vec![],
                });
            }
            Some("file-history-delta") => {
                // A delta can arrive before its snapshot record, so the entry
                // is created on demand either way.
                let Some(snapshot_id) = str_field(&v, "snapshotMessageId") else {
                    return true;
                };
                let Some(backup) = v.get("backup") else {
                    return true;
                };
                let Some(name) = str_field(backup, "backupFileName") else {
                    return true;
                };
                let blob = blob_dir.join(&name);
                let size_bytes = std::fs::metadata(&blob).map(|m| m.len()).unwrap_or(0);
                let entry = checkpoints
                    .entry(snapshot_id.clone())
                    .or_insert_with(|| Checkpoint {
                        message_id: snapshot_id,
                        timestamp: str_field(&v, "timestamp"),
                        files: vec![],
                    });
                entry.files.push(CheckpointFile {
                    tracking_path: str_field(&v, "trackingPath").unwrap_or_default(),
                    version: backup
                        .get("version")
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0) as u32,
                    backup_time: str_field(backup, "backupTime"),
                    exists: blob.is_file(),
                    size_bytes,
                    backup_file_name: name,
                });
            }
            _ => {}
        }
        true
    })?;

    // A checkpoint with no tracked files is bookkeeping, not something to show.
    let mut out: Vec<Checkpoint> = checkpoints
        .into_values()
        .filter(|c| !c.files.is_empty())
        .collect();
    out.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(out)
}

pub fn read_blob(
    claude_dir: &Path,
    session_id: &str,
    backup_file_name: &str,
) -> Result<String, AppError> {
    let path = blob_path(claude_dir, session_id, backup_file_name)?;
    Ok(std::fs::read_to_string(&path)?)
}

pub fn diff(
    claude_dir: &Path,
    session_id: &str,
    left: &DiffSide,
    right: &DiffSide,
) -> Result<DiffResult, AppError> {
    let (left_text, left_label, left_missing, left_binary) = load_side(claude_dir, session_id, left)?;
    let (right_text, right_label, right_missing, right_binary) =
        load_side(claude_dir, session_id, right)?;

    if left_binary || right_binary {
        return Ok(DiffResult {
            left_label,
            right_label,
            hunks: vec![],
            truncated: false,
            binary: true,
            left_missing,
            right_missing,
        });
    }

    let text_diff = similar::TextDiff::from_lines(&left_text, &right_text);
    let mut hunks = Vec::new();
    let mut emitted = 0usize;
    let mut truncated = false;

    'groups: for group in text_diff.grouped_ops(DIFF_CONTEXT) {
        let mut lines = Vec::new();
        let (mut left_start, mut right_start) = (0u32, 0u32);
        let mut started = false;

        for op in &group {
            for change in text_diff.iter_changes(op) {
                let left_no = change.old_index().map(|i| i as u32 + 1);
                let right_no = change.new_index().map(|i| i as u32 + 1);
                if !started {
                    left_start = left_no.unwrap_or(0);
                    right_start = right_no.unwrap_or(0);
                    started = true;
                }
                lines.push(DiffLine {
                    tag: match change.tag() {
                        similar::ChangeTag::Equal => DiffTag::Equal,
                        similar::ChangeTag::Insert => DiffTag::Insert,
                        similar::ChangeTag::Delete => DiffTag::Delete,
                    },
                    left_no,
                    right_no,
                    text: change.value().trim_end_matches(['\n', '\r']).to_string(),
                });
                emitted += 1;
                if emitted >= MAX_DIFF_LINES {
                    truncated = true;
                    hunks.push(DiffHunk {
                        left_start,
                        right_start,
                        lines,
                    });
                    break 'groups;
                }
            }
        }
        if !lines.is_empty() {
            hunks.push(DiffHunk {
                left_start,
                right_start,
                lines,
            });
        }
    }

    Ok(DiffResult {
        left_label,
        right_label,
        hunks,
        truncated,
        binary: false,
        left_missing,
        right_missing,
    })
}

/// Restore a stored version over the file it came from.
///
/// Destructive, so it is deliberately narrow: the destination must be the
/// `trackingPath` the transcript recorded for that blob, it must not be a
/// symlink, and its parent must already exist. The frontend gates this behind
/// a confirmation naming the exact path.
pub fn restore(
    claude_dir: &Path,
    project_name: &str,
    session_id: &str,
    backup_file_name: &str,
    dest: &Path,
) -> Result<(), AppError> {
    let recorded = list(claude_dir, project_name, session_id)?
        .into_iter()
        .flat_map(|c| c.files)
        .find(|f| f.backup_file_name == backup_file_name)
        .ok_or_else(|| {
            AppError::not_found(format!("no checkpoint records blob '{backup_file_name}'"))
        })?;

    if Path::new(&recorded.tracking_path) != dest {
        return Err(AppError::invalid(format!(
            "'{}' is not the path this version was taken from",
            dest.display()
        )));
    }
    if dest.is_symlink() {
        return Err(AppError::invalid(format!(
            "refusing to write through a symlink: {}",
            dest.display()
        )));
    }
    let Some(parent) = dest.parent() else {
        return Err(AppError::invalid("destination has no parent directory"));
    };
    if !parent.is_dir() {
        return Err(AppError::not_found(format!(
            "{} no longer exists",
            parent.display()
        )));
    }

    let contents = std::fs::read(blob_path(claude_dir, session_id, backup_file_name)?)?;
    crate::io::atomic_write(dest, &contents)
}

/// Returns (text, label, missing, looks_binary).
fn load_side(
    claude_dir: &Path,
    session_id: &str,
    side: &DiffSide,
) -> Result<(String, String, bool, bool), AppError> {
    let (path, label) = match side {
        DiffSide::Blob { backup_file_name } => (
            blob_path(claude_dir, session_id, backup_file_name)?,
            backup_file_name.clone(),
        ),
        DiffSide::WorkingTree { path } => (PathBuf::from(path), "working tree".to_string()),
    };
    if !path.is_file() {
        return Ok((String::new(), label, true, false));
    }
    let size = std::fs::metadata(&path)?.len();
    if size > MAX_DIFF_BYTES {
        return Ok((String::new(), label, false, true));
    }
    let bytes = std::fs::read(&path)?;
    // A NUL in the first 8 KiB is the usual "this is not text" signal.
    if bytes.iter().take(8 * 1024).any(|b| *b == 0) {
        return Ok((String::new(), label, false, true));
    }
    match String::from_utf8(bytes) {
        Ok(text) => Ok((text, label, false, false)),
        Err(_) => Ok((String::new(), label, false, true)),
    }
}

fn blob_dir(claude_dir: &Path, session_id: &str) -> Result<PathBuf, AppError> {
    let valid = !session_id.is_empty()
        && session_id.len() <= 64
        && session_id
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '-');
    if !valid {
        return Err(AppError::invalid(format!(
            "invalid session id '{session_id}'"
        )));
    }
    Ok(claude_dir.join(FILE_HISTORY_SUBDIR).join(session_id))
}

/// Blob names are `<16 hex>@v<N>`; anything else is refused before it reaches
/// the filesystem.
fn blob_path(
    claude_dir: &Path,
    session_id: &str,
    backup_file_name: &str,
) -> Result<PathBuf, AppError> {
    let ok = match backup_file_name.split_once("@v") {
        Some((hash, version)) => {
            hash.len() == 16
                && hash.chars().all(|c| c.is_ascii_hexdigit())
                && !version.is_empty()
                && version.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    };
    if !ok {
        return Err(AppError::invalid(format!(
            "invalid backup file name '{backup_file_name}'"
        )));
    }
    let path = blob_dir(claude_dir, session_id)?.join(backup_file_name);
    if !path.is_file() {
        return Err(AppError::not_found(format!(
            "backup '{backup_file_name}' not found"
        )));
    }
    Ok(path)
}

fn str_field(v: &serde_json::Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &str = "01b6a851-f718-4d60-a2c5-d5e51194c96d";
    const BLOB: &str = "0b642a3eaf179c2b@v1";

    fn setup(td: &Path, transcript_lines: &[String], blobs: &[(&str, &str)]) {
        let dir = td.join(PROJECTS_SUBDIR).join("-tmp-p");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{SESSION}.jsonl")),
            transcript_lines.join("\n"),
        )
        .unwrap();
        let bd = td.join(FILE_HISTORY_SUBDIR).join(SESSION);
        std::fs::create_dir_all(&bd).unwrap();
        for (name, body) in blobs {
            std::fs::write(bd.join(name), body).unwrap();
        }
    }

    fn snapshot(id: &str, ts: &str) -> String {
        format!(
            r#"{{"type":"file-history-snapshot","messageId":"{id}","snapshot":{{"messageId":"{id}","trackedFileBackups":{{}},"timestamp":"{ts}"}},"isSnapshotUpdate":false}}"#
        )
    }

    fn delta(snapshot_id: &str, tracking: &str, blob: &str, version: u32) -> String {
        format!(
            r#"{{"type":"file-history-delta","messageId":"m","snapshotMessageId":"{snapshot_id}","trackingPath":"{tracking}","backup":{{"backupFileName":"{blob}","version":{version},"backupTime":"2026-08-12T03:53:37.862Z"}},"timestamp":"2026-08-12T03:53:37.862Z"}}"#
        )
    }

    #[test]
    fn checkpoints_join_transcript_records_to_blobs() {
        let td = tempfile::tempdir().unwrap();
        setup(
            td.path(),
            &[
                snapshot("snap1", "2026-08-12T03:00:00Z"),
                delta("snap1", "/tmp/a.txt", BLOB, 1),
                r#"{"type":"user","message":{"role":"user","content":"hi"}}"#.into(),
            ],
            &[(BLOB, "one\ntwo\n")],
        );

        let cps = list(td.path(), "-tmp-p", SESSION).unwrap();
        assert_eq!(cps.len(), 1);
        assert_eq!(cps[0].message_id, "snap1");
        assert_eq!(cps[0].files.len(), 1);
        let f = &cps[0].files[0];
        assert_eq!(f.backup_file_name, BLOB);
        assert_eq!(f.tracking_path, "/tmp/a.txt");
        assert_eq!(f.version, 1);
        assert!(f.exists);
        assert_eq!(f.size_bytes, 8);
    }

    #[test]
    fn missing_blob_is_surfaced_not_dropped() {
        let td = tempfile::tempdir().unwrap();
        setup(
            td.path(),
            &[snapshot("snap1", "t"), delta("snap1", "/tmp/a.txt", BLOB, 1)],
            &[],
        );
        let f = &list(td.path(), "-tmp-p", SESSION).unwrap()[0].files[0];
        assert!(!f.exists, "a referenced-but-absent blob is the interesting case");
        assert_eq!(f.size_bytes, 0);
    }

    #[test]
    fn empty_checkpoints_and_garbage_lines_are_skipped() {
        let td = tempfile::tempdir().unwrap();
        setup(
            td.path(),
            &[
                snapshot("snap-empty", "t"),
                "not json".into(),
                r#"{"type":"file-history-delta"}"#.into(),
            ],
            &[],
        );
        assert!(list(td.path(), "-tmp-p", SESSION).unwrap().is_empty());
    }

    #[test]
    fn diff_reports_added_and_removed_lines() {
        let td = tempfile::tempdir().unwrap();
        setup(
            td.path(),
            &[snapshot("snap1", "t"), delta("snap1", "/tmp/a.txt", BLOB, 1)],
            &[(BLOB, "one\ntwo\nthree\n")],
        );
        let work = td.path().join("a.txt");
        std::fs::write(&work, "one\ntwo point five\nthree\n").unwrap();

        let d = diff(
            td.path(),
            SESSION,
            &DiffSide::Blob {
                backup_file_name: BLOB.into(),
            },
            &DiffSide::WorkingTree {
                path: work.to_string_lossy().into_owned(),
            },
        )
        .unwrap();

        assert!(!d.binary && !d.truncated);
        let tags: Vec<_> = d.hunks.iter().flat_map(|h| &h.lines).map(|l| l.tag).collect();
        assert!(tags.contains(&DiffTag::Delete));
        assert!(tags.contains(&DiffTag::Insert));
        let inserted: Vec<_> = d
            .hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| l.tag == DiffTag::Insert)
            .map(|l| l.text.clone())
            .collect();
        assert_eq!(inserted, vec!["two point five"]);
    }

    #[test]
    fn binary_and_oversize_sides_short_circuit() {
        let td = tempfile::tempdir().unwrap();
        setup(
            td.path(),
            &[snapshot("s", "t"), delta("s", "/tmp/a", BLOB, 1)],
            &[(BLOB, "text\n")],
        );
        let bin = td.path().join("bin");
        std::fs::write(&bin, [0x00, 0x01, 0x02]).unwrap();

        let d = diff(
            td.path(),
            SESSION,
            &DiffSide::Blob {
                backup_file_name: BLOB.into(),
            },
            &DiffSide::WorkingTree {
                path: bin.to_string_lossy().into_owned(),
            },
        )
        .unwrap();
        assert!(d.binary);
        assert!(d.hunks.is_empty());
    }

    #[test]
    fn missing_working_tree_file_is_reported_not_an_error() {
        let td = tempfile::tempdir().unwrap();
        setup(
            td.path(),
            &[snapshot("s", "t"), delta("s", "/tmp/a", BLOB, 1)],
            &[(BLOB, "text\n")],
        );
        let d = diff(
            td.path(),
            SESSION,
            &DiffSide::Blob {
                backup_file_name: BLOB.into(),
            },
            &DiffSide::WorkingTree {
                path: "/nonexistent/file".into(),
            },
        )
        .unwrap();
        assert!(d.right_missing);
    }

    #[test]
    fn blob_names_and_session_ids_are_validated() {
        let td = tempfile::tempdir().unwrap();
        setup(td.path(), &[snapshot("s", "t")], &[(BLOB, "x")]);
        assert!(read_blob(td.path(), SESSION, "../../../etc/passwd").is_err());
        assert!(read_blob(td.path(), SESSION, "0b642a3eaf179c2b@vx").is_err());
        assert!(read_blob(td.path(), SESSION, "short@v1").is_err());
        assert!(read_blob(td.path(), "../evil", BLOB).is_err());
        assert_eq!(read_blob(td.path(), SESSION, BLOB).unwrap(), "x");
    }

    #[test]
    fn restore_only_writes_the_path_the_version_came_from() {
        let td = tempfile::tempdir().unwrap();
        let target = td.path().join("work").join("a.txt");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "current\n").unwrap();

        setup(
            td.path(),
            &[
                snapshot("snap1", "t"),
                delta("snap1", &target.to_string_lossy(), BLOB, 1),
            ],
            &[(BLOB, "original\n")],
        );

        // A different destination is refused even though the blob is valid.
        let elsewhere = td.path().join("work").join("b.txt");
        assert!(restore(td.path(), "-tmp-p", SESSION, BLOB, &elsewhere).is_err());

        restore(td.path(), "-tmp-p", SESSION, BLOB, &target).unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "original\n");
    }

    #[test]
    fn missing_session_is_not_found() {
        let td = tempfile::tempdir().unwrap();
        assert!(list(td.path(), "-tmp-p", SESSION).is_err());
    }
}
