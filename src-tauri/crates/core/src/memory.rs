//! CLAUDE.md files and `.claude/rules/`, the instructions Claude Code reads at
//! the start of every session.
//!
//! Sources, in the order they apply:
//!
//! * `~/.claude/CLAUDE.md` and `~/.claude/rules/**/*.md` — every project
//! * `<project>/CLAUDE.md`, `<project>/CLAUDE.local.md` (deprecated but still
//!   read when present) and `<project>/.claude/rules/**/*.md`
//!
//! A memory file can pull in others with `@path` on its own line. This module
//! resolves those so the UI can show what a file actually contributes, cycles
//! included — a cycle is marked, not treated as an error, because the file is
//! still perfectly readable.
//!
//! This is a separate module from `projects` on purpose: that one is the
//! project *registry* (path encoding, sessions, git), while memory spans two
//! roots and needs import resolution.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::io;
use crate::types::{MemoryDoc, MemoryFile, MemoryImport, MemoryKind, MemoryPreview, MemoryScope};
use crate::AppError;

const CLAUDE_MD: &str = "CLAUDE.md";
const CLAUDE_LOCAL_MD: &str = "CLAUDE.local.md";
const CLAUDE_SUBDIR: &str = ".claude";
const RULES_SUBDIR: &str = "rules";
const AGENT_MEMORY_SUBDIR: &str = "agent-memory";

/// Import depth cap. Deep chains are almost always a mistake, and the cap keeps
/// a pathological tree from stalling the UI.
const MAX_IMPORT_DEPTH: u32 = 5;
/// Flattened preview cap.
const MAX_PREVIEW_BYTES: usize = 1024 * 1024;

pub fn list(claude_dir: &Path, working_dir: Option<&Path>) -> Result<Vec<MemoryFile>, AppError> {
    let mut out = Vec::new();

    push_if_known(
        &mut out,
        claude_dir.join(CLAUDE_MD),
        MemoryKind::ClaudeMd,
        MemoryScope::User,
        claude_dir,
    );
    collect_rules(
        &mut out,
        &claude_dir.join(RULES_SUBDIR),
        MemoryScope::User,
        claude_dir,
        MemoryKind::Rule,
    );

    if let Some(wd) = working_dir {
        push_if_known(&mut out, wd.join(CLAUDE_MD), MemoryKind::ClaudeMd, MemoryScope::Project, wd);
        // Deprecated, so it is listed only when it exists rather than offered.
        let local = wd.join(CLAUDE_LOCAL_MD);
        if local.is_file() {
            push_if_known(&mut out, local, MemoryKind::ClaudeLocalMd, MemoryScope::Project, wd);
        }
        collect_rules(
            &mut out,
            &wd.join(CLAUDE_SUBDIR).join(RULES_SUBDIR),
            MemoryScope::Project,
            wd,
            MemoryKind::Rule,
        );
    }

    Ok(out)
}

/// Per-agent memory written by the CLI. Read-only here: this app has no
/// business editing what an agent recorded for itself.
pub fn agent_memory_list(
    claude_dir: &Path,
    working_dir: Option<&Path>,
) -> Result<Vec<MemoryFile>, AppError> {
    let mut out = Vec::new();
    collect_rules(
        &mut out,
        &claude_dir.join(AGENT_MEMORY_SUBDIR),
        MemoryScope::User,
        claude_dir,
        MemoryKind::AgentMemory,
    );
    if let Some(wd) = working_dir {
        collect_rules(
            &mut out,
            &wd.join(CLAUDE_SUBDIR).join(AGENT_MEMORY_SUBDIR),
            MemoryScope::Project,
            wd,
            MemoryKind::AgentMemory,
        );
    }
    Ok(out)
}

pub fn get(
    claude_dir: &Path,
    working_dir: Option<&Path>,
    id: &str,
) -> Result<MemoryDoc, AppError> {
    let file = resolve(claude_dir, working_dir, id)?;
    let content = if file.exists {
        std::fs::read_to_string(&file.path)?
    } else {
        String::new()
    };
    let imports = resolve_imports(Path::new(&file.path), &content);
    Ok(MemoryDoc {
        file,
        content,
        imports,
    })
}

pub fn put(
    claude_dir: &Path,
    working_dir: Option<&Path>,
    id: &str,
    content: &str,
    expected_mtime_ms: Option<i64>,
) -> Result<MemoryDoc, AppError> {
    let file = resolve(claude_dir, working_dir, id)?;
    if matches!(file.kind, MemoryKind::AgentMemory) {
        return Err(AppError::invalid(
            "agent memory is written by the agent and is read-only here",
        ));
    }
    let path = PathBuf::from(&file.path);
    io::check_unchanged(&path, expected_mtime_ms)?;
    io::atomic_write(&path, content.as_bytes())?;
    get(claude_dir, working_dir, id)
}

pub fn delete(claude_dir: &Path, working_dir: Option<&Path>, id: &str) -> Result<(), AppError> {
    let file = resolve(claude_dir, working_dir, id)?;
    if !file.exists {
        return Err(AppError::not_found(format!("{} does not exist", file.path)));
    }
    io::remove_file(Path::new(&file.path))
}

/// The file with its imports inlined, in the order Claude Code would read
/// them, each marked with where it came from.
pub fn preview(
    claude_dir: &Path,
    working_dir: Option<&Path>,
    id: &str,
) -> Result<MemoryPreview, AppError> {
    let doc = get(claude_dir, working_dir, id)?;
    let mut out = String::new();
    let mut sources = vec![doc.file.path.clone()];
    out.push_str(&format!("<!-- from: {} -->\n", doc.file.path));
    out.push_str(&doc.content);

    let mut truncated = false;
    for import in &doc.imports {
        if !import.exists || import.cyclic {
            continue;
        }
        let Ok(body) = std::fs::read_to_string(&import.resolved_path) else {
            continue;
        };
        if out.len() + body.len() > MAX_PREVIEW_BYTES {
            truncated = true;
            break;
        }
        out.push_str(&format!("\n\n<!-- from: {} -->\n", import.resolved_path));
        out.push_str(&body);
        sources.push(import.resolved_path.clone());
    }

    Ok(MemoryPreview {
        flattened: out,
        sources,
        truncated,
    })
}

/// Find `@path` imports, depth-first, marking missing and cyclic ones.
pub fn resolve_imports(root: &Path, content: &str) -> Vec<MemoryImport> {
    let mut out = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();
    visited.insert(root.canonicalize().unwrap_or_else(|_| root.to_path_buf()));
    walk_imports(root, content, 1, &mut visited, &mut out);
    out
}

fn walk_imports(
    parent: &Path,
    content: &str,
    depth: u32,
    visited: &mut HashSet<PathBuf>,
    out: &mut Vec<MemoryImport>,
) {
    if depth > MAX_IMPORT_DEPTH {
        return;
    }
    for raw in parse_import_lines(content) {
        let resolved = resolve_import_path(parent, &raw);
        let canonical = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
        let cyclic = visited.contains(&canonical);
        let exists = resolved.is_file();

        out.push(MemoryImport {
            raw: raw.clone(),
            resolved_path: resolved.to_string_lossy().into_owned(),
            exists,
            depth,
            cyclic,
            parent_path: parent.to_string_lossy().into_owned(),
        });

        if cyclic || !exists {
            continue;
        }
        visited.insert(canonical);
        if let Ok(body) = std::fs::read_to_string(&resolved) {
            walk_imports(&resolved, &body, depth + 1, visited, out);
        }
    }
}

/// `@path` is an import only at the start of a line (after optional list
/// markers or whitespace) and outside fenced code blocks — an `@` inside a
/// snippet or an email address is not an import.
fn parse_import_lines(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let candidate = trimmed
            .trim_start_matches(['-', '*', '+'])
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .trim_start_matches(['.', ')'])
            .trim_start();
        let Some(rest) = candidate.strip_prefix('@') else {
            continue;
        };
        let path = rest.split_whitespace().next().unwrap_or("");
        if !path.is_empty() {
            out.push(path.to_string());
        }
    }
    out
}

fn resolve_import_path(parent: &Path, raw: &str) -> PathBuf {
    let expanded = crate::files::expand_tilde(raw);
    if expanded.is_absolute() {
        return expanded;
    }
    parent
        .parent()
        .map(|d| d.join(&expanded))
        .unwrap_or(expanded)
}

/// `<scope>:<kind>:<relative path>`.
fn make_id(scope: MemoryScope, kind: MemoryKind, rel: &str) -> String {
    let s = match scope {
        MemoryScope::User => "user",
        MemoryScope::Project => "project",
    };
    let k = match kind {
        MemoryKind::ClaudeMd => "claudeMd",
        MemoryKind::ClaudeLocalMd => "claudeLocalMd",
        MemoryKind::Rule => "rule",
        MemoryKind::AgentMemory => "agentMemory",
    };
    format!("{s}:{k}:{rel}")
}

/// Rebuild a `MemoryFile` from its id, validating the relative path so a
/// crafted id cannot escape its root.
fn resolve(
    claude_dir: &Path,
    working_dir: Option<&Path>,
    id: &str,
) -> Result<MemoryFile, AppError> {
    let mut parts = id.splitn(3, ':');
    let scope = match parts.next() {
        Some("user") => MemoryScope::User,
        Some("project") => MemoryScope::Project,
        _ => return Err(AppError::invalid(format!("malformed memory id '{id}'"))),
    };
    let kind = match parts.next() {
        Some("claudeMd") => MemoryKind::ClaudeMd,
        Some("claudeLocalMd") => MemoryKind::ClaudeLocalMd,
        Some("rule") => MemoryKind::Rule,
        Some("agentMemory") => MemoryKind::AgentMemory,
        _ => return Err(AppError::invalid(format!("malformed memory id '{id}'"))),
    };
    let rel = parts
        .next()
        .ok_or_else(|| AppError::invalid(format!("malformed memory id '{id}'")))?;
    if rel.contains("..") || rel.starts_with('/') || !rel.ends_with(".md") {
        return Err(AppError::invalid(format!("invalid memory path '{rel}'")));
    }

    let root = match scope {
        MemoryScope::User => claude_dir.to_path_buf(),
        MemoryScope::Project => working_dir
            .ok_or_else(|| AppError::invalid("project memory requires a working directory"))?
            .to_path_buf(),
    };
    Ok(describe(root.join(rel), kind, scope, &root))
}

fn push_if_known(
    out: &mut Vec<MemoryFile>,
    path: PathBuf,
    kind: MemoryKind,
    scope: MemoryScope,
    root: &Path,
) {
    out.push(describe(path, kind, scope, root));
}

/// Walk a rules-style directory for `.md` files.
fn collect_rules(
    out: &mut Vec<MemoryFile>,
    dir: &Path,
    scope: MemoryScope,
    root: &Path,
    kind: MemoryKind,
) {
    if !dir.is_dir() {
        return;
    }
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(describe(path.to_path_buf(), kind, scope, root));
        }
    }
}

fn describe(path: PathBuf, kind: MemoryKind, scope: MemoryScope, root: &Path) -> MemoryFile {
    let rel = path
        .strip_prefix(root)
        .unwrap_or(&path)
        .to_string_lossy()
        .into_owned();
    let meta = std::fs::metadata(&path).ok();
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    MemoryFile {
        id: make_id(scope, kind, &rel),
        kind,
        scope,
        exists: meta.is_some(),
        size_bytes: meta.as_ref().map(|m| m.len()),
        mtime_ms: io::mtime_ms(&path),
        title: content
            .lines()
            .find(|l| l.starts_with('#'))
            .map(|l| l.trim_start_matches('#').trim().to_string()),
        import_count: parse_import_lines(&content).len() as u32,
        rel_path: rel,
        path: path.to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_user_and_project_sources() {
        let td = tempfile::tempdir().unwrap();
        let wd = td.path().join("proj");
        std::fs::create_dir_all(wd.join(".claude/rules")).unwrap();
        std::fs::create_dir_all(td.path().join("rules/nested")).unwrap();

        std::fs::write(td.path().join(CLAUDE_MD), "# Global\n").unwrap();
        std::fs::write(td.path().join("rules/nested/style.md"), "# Style\n").unwrap();
        std::fs::write(wd.join(CLAUDE_MD), "# Project\n").unwrap();
        std::fs::write(wd.join(".claude/rules/tests.md"), "# Tests\n").unwrap();

        let files = list(td.path(), Some(&wd)).unwrap();
        let ids: Vec<&str> = files.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.contains(&"user:claudeMd:CLAUDE.md"));
        assert!(ids.contains(&"user:rule:rules/nested/style.md"));
        assert!(ids.contains(&"project:claudeMd:CLAUDE.md"));
        assert!(ids.contains(&"project:rule:.claude/rules/tests.md"));
        // CLAUDE.local.md is deprecated: absent means not listed.
        assert!(!ids.iter().any(|i| i.contains("claudeLocalMd")));

        let global = files.iter().find(|f| f.id.starts_with("user:claudeMd")).unwrap();
        assert_eq!(global.title.as_deref(), Some("Global"));
        assert!(global.exists);
    }

    #[test]
    fn a_missing_claude_md_is_listed_as_absent() {
        let td = tempfile::tempdir().unwrap();
        let files = list(td.path(), None).unwrap();
        assert_eq!(files.len(), 1);
        assert!(!files[0].exists, "so the UI can offer to create it");
        assert!(files[0].size_bytes.is_none());
    }

    #[test]
    fn imports_resolve_relative_absolute_and_tilde() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(td.path().join("other.md"), "# Other\n").unwrap();
        std::fs::write(
            td.path().join(CLAUDE_MD),
            "# Root\n@other.md\n- @./other.md\n",
        )
        .unwrap();

        let doc = get(td.path(), None, "user:claudeMd:CLAUDE.md").unwrap();
        assert_eq!(doc.imports.len(), 2);
        assert!(doc.imports.iter().all(|i| i.exists));
        assert_eq!(doc.file.import_count, 2);
    }

    #[test]
    fn imports_inside_code_fences_are_not_imports() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join(CLAUDE_MD),
            "# Root\n```\n@not-an-import.md\n```\nmail me at a@b.com\n@real.md\n",
        )
        .unwrap();
        let doc = get(td.path(), None, "user:claudeMd:CLAUDE.md").unwrap();
        assert_eq!(doc.imports.len(), 1);
        assert_eq!(doc.imports[0].raw, "real.md");
        assert!(!doc.imports[0].exists);
    }

    #[test]
    fn cycles_are_marked_not_fatal() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(td.path().join(CLAUDE_MD), "# A\n@b.md\n").unwrap();
        std::fs::write(td.path().join("b.md"), "# B\n@CLAUDE.md\n").unwrap();

        let doc = get(td.path(), None, "user:claudeMd:CLAUDE.md").unwrap();
        assert_eq!(doc.imports.len(), 2);
        assert!(doc.imports[1].cyclic, "the loop back to the root is flagged");
    }

    #[test]
    fn preview_flattens_depth_first_with_source_markers() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(td.path().join("child.md"), "child body\n").unwrap();
        std::fs::write(td.path().join(CLAUDE_MD), "# Root\n@child.md\n").unwrap();

        let p = preview(td.path(), None, "user:claudeMd:CLAUDE.md").unwrap();
        assert!(p.flattened.contains("# Root"));
        assert!(p.flattened.contains("child body"));
        assert!(p.flattened.contains("<!-- from:"));
        assert_eq!(p.sources.len(), 2);
        assert!(!p.truncated);
    }

    #[test]
    fn put_creates_and_round_trips() {
        let td = tempfile::tempdir().unwrap();
        let doc = put(td.path(), None, "user:claudeMd:CLAUDE.md", "# Hello\n", None).unwrap();
        assert_eq!(doc.content, "# Hello\n");
        assert!(td.path().join(CLAUDE_MD).is_file());

        // A stale mtime is refused.
        let stale = doc.file.mtime_ms.unwrap() - 10;
        assert!(put(td.path(), None, "user:claudeMd:CLAUDE.md", "x", Some(stale)).is_err());
    }

    #[test]
    fn ids_are_validated_against_traversal() {
        let td = tempfile::tempdir().unwrap();
        for bad in [
            "user:claudeMd:../../etc/passwd.md",
            "user:claudeMd:/etc/passwd.md",
            "user:claudeMd:notmarkdown.txt",
            "nonsense",
            "user:unknownKind:a.md",
        ] {
            assert!(get(td.path(), None, bad).is_err(), "expected '{bad}' to fail");
        }
        // Project ids need a working directory.
        assert!(get(td.path(), None, "project:claudeMd:CLAUDE.md").is_err());
    }

    #[test]
    fn agent_memory_is_read_only() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join(AGENT_MEMORY_SUBDIR)).unwrap();
        std::fs::write(td.path().join("agent-memory/notes.md"), "# Notes\n").unwrap();

        let files = agent_memory_list(td.path(), None).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].kind, MemoryKind::AgentMemory);

        let err = put(td.path(), None, &files[0].id, "edited", None).unwrap_err();
        assert!(err.message.contains("read-only"));
    }
}
