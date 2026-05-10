//! Read+write logic for `~/.claude/projects/`. Projects are encoded
//! subdirs: `/Users/foo/app` → `-Users-foo-app`. Each contains JSONL
//! session files. The decoded path is also the working dir for spawning
//! `claude --resume`.

use std::path::{Path, PathBuf};

use crate::io;
use crate::types::{Project, ProjectInfo};
use crate::{AppError, ErrorCode};

const PROJECTS_SUBDIR: &str = "projects";
const CLAUDE_MD: &str = "CLAUDE.md";

pub fn list(claude_dir: &Path) -> Result<Vec<Project>, AppError> {
    let root = claude_dir.join(PROJECTS_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(encoded) = path.file_name().and_then(|s| s.to_str()) else { continue };
        let decoded = decode(encoded);
        let (count, last) = scan_sessions(&path);
        out.push(Project {
            name: encoded.to_string(),
            working_dir: decoded,
            session_count: count,
            last_active: last,
        });
    }
    out.sort_by(|a, b| b.last_active.cmp(&a.last_active).then(a.name.cmp(&b.name)));
    Ok(out)
}

pub fn get(claude_dir: &Path, name: &str) -> Result<Project, AppError> {
    list(claude_dir)?
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| AppError::not_found(format!("project '{name}' not found")))
}

pub fn resolve(path: &Path) -> ProjectInfo {
    let working_dir = path.to_string_lossy().into_owned();
    let name = encode(&working_dir);
    ProjectInfo {
        name,
        working_dir,
        exists_on_disk: path.is_dir(),
    }
}

/// Encode `/Users/foo/app` → `-Users-foo-app`.
pub fn encode(abs_path: &str) -> String {
    abs_path.replace('/', "-")
}

/// Decode `-Users-foo-app` → `/Users/foo/app`.
pub fn decode(encoded: &str) -> String {
    encoded.replace('-', "/")
}

fn scan_sessions(project_dir: &Path) -> (usize, Option<String>) {
    let mut count = 0;
    let mut latest: Option<std::time::SystemTime> = None;
    let Ok(entries) = std::fs::read_dir(project_dir) else { return (0, None) };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        count += 1;
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                latest = Some(latest.map(|l| l.max(modified)).unwrap_or(modified));
            }
        }
    }
    let last_active = latest.and_then(|t| {
        t.duration_since(std::time::UNIX_EPOCH).ok().and_then(|d| {
            chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                .map(|dt| dt.to_rfc3339())
        })
    });
    (count, last_active)
}

/// Resolve project files for the file-tree view. Phase 1: shallow dir read.
pub fn files(claude_dir: &Path, project_name: &str, sub_path: Option<&str>) -> Result<Vec<crate::types::FileNode>, AppError> {
    let project = get(claude_dir, project_name)?;
    let mut base = PathBuf::from(&project.working_dir);
    if let Some(s) = sub_path {
        base = base.join(s);
    }
    if !base.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&base)?.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let is_dir = path.is_dir();
        let size = if is_dir { None } else { std::fs::metadata(&path).ok().map(|m| m.len()) };
        out.push(crate::types::FileNode {
            name,
            path: path.to_string_lossy().into_owned(),
            is_dir,
            size,
        });
    }
    out.sort_by(|a, b| (b.is_dir, a.name.clone()).cmp(&(a.is_dir, b.name.clone())));
    Ok(out)
}

/// Create an empty project entry under `~/.claude/projects/`. The path is
/// recorded by encoding it; the underlying working directory is *not*
/// touched. Errors with `InvalidInput` if the project already exists.
pub fn create(claude_dir: &Path, working_path: &str) -> Result<Project, AppError> {
    if working_path.trim().is_empty() {
        return Err(AppError::invalid("project path cannot be empty"));
    }
    let name = encode(working_path);
    let dir = claude_dir.join(PROJECTS_SUBDIR).join(&name);
    if dir.exists() {
        return Err(AppError::invalid(format!(
            "project '{name}' already exists"
        )));
    }
    std::fs::create_dir_all(&dir)?;
    get(claude_dir, &name).or_else(|_| {
        // Brand-new dir has no sessions — synthesize a minimal record.
        Ok(Project {
            name: name.clone(),
            working_dir: working_path.to_string(),
            session_count: 0,
            last_active: None,
        })
    })
}

pub fn rename(claude_dir: &Path, old_name: &str, new_name: &str) -> Result<(), AppError> {
    if new_name.trim().is_empty() {
        return Err(AppError::invalid("new project name cannot be empty"));
    }
    let root = claude_dir.join(PROJECTS_SUBDIR);
    let from = root.join(old_name);
    let to = root.join(new_name);
    if !from.is_dir() {
        return Err(AppError::not_found(format!(
            "project '{old_name}' not found"
        )));
    }
    if to.exists() {
        return Err(AppError::invalid(format!(
            "project '{new_name}' already exists"
        )));
    }
    std::fs::rename(&from, &to)?;
    Ok(())
}

pub fn delete(claude_dir: &Path, name: &str) -> Result<(), AppError> {
    let dir = claude_dir.join(PROJECTS_SUBDIR).join(name);
    if !dir.is_dir() {
        return Err(AppError::not_found(format!("project '{name}' not found")));
    }
    io::remove_dir_all(&dir)
}

pub fn claude_md_get(claude_dir: &Path, name: &str) -> Result<String, AppError> {
    let project = get(claude_dir, name)?;
    let path = PathBuf::from(&project.working_dir).join(CLAUDE_MD);
    if !path.is_file() {
        return Ok(String::new());
    }
    Ok(std::fs::read_to_string(&path)?)
}

pub fn claude_md_put(claude_dir: &Path, name: &str, content: &str) -> Result<(), AppError> {
    let project = get(claude_dir, name)?;
    let wd = PathBuf::from(&project.working_dir);
    if !wd.is_dir() {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "project working directory does not exist on disk",
        ));
    }
    io::atomic_write(&wd.join(CLAUDE_MD), content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        assert_eq!(encode("/Users/foo/app"), "-Users-foo-app");
        assert_eq!(decode("-Users-foo-app"), "/Users/foo/app");
    }

    #[test]
    fn list_counts_sessions() {
        let td = tempfile::tempdir().unwrap();
        let p = td.path().join("projects/-tmp-test");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("01.jsonl"), "{}\n").unwrap();
        std::fs::write(p.join("02.jsonl"), "{}\n").unwrap();
        std::fs::write(p.join("not-a-session.txt"), "ignore").unwrap();

        let projs = list(td.path()).unwrap();
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0].name, "-tmp-test");
        assert_eq!(projs[0].working_dir, "/tmp/test");
        assert_eq!(projs[0].session_count, 2);
    }

    #[test]
    fn create_rename_delete() {
        let td = tempfile::tempdir().unwrap();
        let p = create(td.path(), "/tmp/scratch").unwrap();
        assert_eq!(p.name, "-tmp-scratch");

        rename(td.path(), "-tmp-scratch", "-tmp-other").unwrap();
        assert!(!td.path().join("projects/-tmp-scratch").exists());
        assert!(td.path().join("projects/-tmp-other").is_dir());

        delete(td.path(), "-tmp-other").unwrap();
        assert!(!td.path().join("projects/-tmp-other").exists());
    }
}
