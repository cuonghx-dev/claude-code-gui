//! Read+write logic for `~/.claude/agents/`.
//!
//! List/get walks `agents/**/*.md` recursively, parses frontmatter via
//! `frontmatter::parse::<AgentFrontmatter>`, and tolerates unknown keys
//! by stashing them in `frontmatter.extra`.
//!
//! Create/update/delete write through `io::atomic_write` so a partial save
//! never appears to the file watcher. Update treats slug+directory as the
//! identity; if either changes the old file is removed after the new one
//! is written.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::frontmatter::{self, Document};
use crate::io;
use crate::types::{Agent, AgentFrontmatter, AgentImport, AgentInput};
use crate::{AppError, ErrorCode};

const AGENTS_SUBDIR: &str = "agents";

/// List every `.md` file under `<claude_dir>/agents/`. Returns sorted by
/// `(directory, slug)` for stable UI ordering.
pub fn list(claude_dir: &Path) -> Result<Vec<Agent>, AppError> {
    let root = claude_dir.join(AGENTS_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        match read_one(&root, path) {
            Ok(agent) => out.push(agent),
            Err(e) => tracing::warn!(error = %e, path = %path.display(), "skipping unparseable agent"),
        }
    }
    out.sort_by(|a, b| (&a.directory, &a.slug).cmp(&(&b.directory, &b.slug)));
    Ok(out)
}

/// Look up a single agent by slug. Errors with `NotFound` if no match.
pub fn get(claude_dir: &Path, slug: &str) -> Result<Agent, AppError> {
    list(claude_dir)?
        .into_iter()
        .find(|a| a.slug == slug)
        .ok_or_else(|| AppError::not_found(format!("agent '{slug}' not found")))
}

fn read_one(root: &Path, path: &Path) -> Result<Agent, AppError> {
    let src = std::fs::read_to_string(path)?;
    let Document { frontmatter, body }: Document<AgentFrontmatter> = frontmatter::parse(&src)?;

    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::new(ErrorCode::InvalidInput, "invalid agent filename"))?
        .to_string();

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    let directory = relative_dir(root, path);

    let has_memory = matches!(
        frontmatter.memory,
        Some(crate::types::AgentMemory::User)
            | Some(crate::types::AgentMemory::Project)
            | Some(crate::types::AgentMemory::Local)
    );

    Ok(Agent {
        slug,
        filename,
        directory,
        frontmatter,
        body,
        has_memory,
        file_path: path.to_string_lossy().into_owned(),
    })
}

/// Returns the dir relative to `<claude_dir>/agents/`, with forward slashes.
/// Empty for top-level files.
fn relative_dir(root: &Path, file: &Path) -> String {
    let parent = match file.parent() {
        Some(p) => p,
        None => return String::new(),
    };
    let rel = match parent.strip_prefix(root) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn create(claude_dir: &Path, input: AgentInput) -> Result<Agent, AppError> {
    io::validate_slug(&input.slug)?;
    io::validate_relative_dir(&input.directory)?;
    let path = file_path(claude_dir, &input.directory, &input.slug);
    if path.exists() {
        return Err(AppError::invalid(format!(
            "agent '{}' already exists",
            input.slug
        )));
    }
    write_doc(&path, &input)?;
    get_at(claude_dir, &input.directory, &input.slug)
}

pub fn update(claude_dir: &Path, slug: &str, input: AgentInput) -> Result<Agent, AppError> {
    io::validate_slug(&input.slug)?;
    io::validate_relative_dir(&input.directory)?;
    let existing = get(claude_dir, slug)?;
    let new_path = file_path(claude_dir, &input.directory, &input.slug);
    let old_path = PathBuf::from(&existing.file_path);

    if old_path != new_path && new_path.exists() {
        return Err(AppError::invalid(format!(
            "target slug '{}' already exists at '{}'",
            input.slug,
            new_path.display()
        )));
    }
    write_doc(&new_path, &input)?;
    if old_path != new_path {
        let _ = io::remove_file(&old_path);
    }
    get_at(claude_dir, &input.directory, &input.slug)
}

pub fn delete(claude_dir: &Path, slug: &str) -> Result<(), AppError> {
    let existing = get(claude_dir, slug)?;
    io::remove_file(Path::new(&existing.file_path))
}

/// Return the on-disk markdown source for an agent (frontmatter + body).
pub fn export(claude_dir: &Path, slug: &str) -> Result<String, AppError> {
    let agent = get(claude_dir, slug)?;
    let raw = std::fs::read_to_string(&agent.file_path)?;
    Ok(raw)
}

/// Write an externally-supplied markdown source verbatim. Validates that
/// it parses as an agent before persisting.
pub fn import(claude_dir: &Path, payload: AgentImport) -> Result<Agent, AppError> {
    io::validate_slug(&payload.slug)?;
    io::validate_relative_dir(&payload.directory)?;
    let _: Document<AgentFrontmatter> = frontmatter::parse(&payload.content)?;
    let path = file_path(claude_dir, &payload.directory, &payload.slug);
    if path.exists() {
        return Err(AppError::invalid(format!(
            "agent '{}' already exists",
            payload.slug
        )));
    }
    io::atomic_write(&path, payload.content.as_bytes())?;
    get_at(claude_dir, &payload.directory, &payload.slug)
}

fn file_path(claude_dir: &Path, directory: &str, slug: &str) -> PathBuf {
    let mut p = claude_dir.join(AGENTS_SUBDIR);
    if !directory.is_empty() {
        p.push(directory);
    }
    p.push(format!("{slug}.md"));
    p
}

fn write_doc(path: &Path, input: &AgentInput) -> Result<(), AppError> {
    let doc = Document {
        frontmatter: input.frontmatter.clone(),
        body: input.body.clone(),
    };
    let serialized = frontmatter::serialize(&doc)?;
    io::atomic_write(path, serialized.as_bytes())
}

fn get_at(claude_dir: &Path, directory: &str, slug: &str) -> Result<Agent, AppError> {
    let root = claude_dir.join(AGENTS_SUBDIR);
    let path = file_path(claude_dir, directory, slug);
    read_one(&root, &path)
}

/// `agent_slug -> count of skills referenced in frontmatter`. Cheap O(n) scan.
pub fn skill_counts(claude_dir: &Path) -> Result<std::collections::HashMap<String, usize>, AppError> {
    let mut counts = std::collections::HashMap::new();
    for agent in list(claude_dir)? {
        counts.insert(agent.slug.clone(), agent.frontmatter.skills.len());
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let td = tempfile::tempdir().unwrap();
        let agents = td.path().join(AGENTS_SUBDIR);
        std::fs::create_dir_all(agents.join("nested")).unwrap();

        std::fs::write(
            agents.join("reviewer.md"),
            "---\nname: Reviewer\nmodel: sonnet\nmemory: user\nskills: [refactor]\ntools: [Read, Bash]\n---\n\nYou review code.\n",
        )
        .unwrap();
        std::fs::write(
            agents.join("nested/helper.md"),
            "---\nname: Helper\n---\n\nbody\n",
        )
        .unwrap();

        let path = td.path().to_path_buf();
        (td, path)
    }

    #[test]
    fn list_walks_recursively_and_sorts() {
        let (_td, dir) = fixture();
        let agents = list(&dir).unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].directory, "");
        assert_eq!(agents[0].slug, "reviewer");
        assert_eq!(agents[1].directory, "nested");
        assert_eq!(agents[1].slug, "helper");
    }

    #[test]
    fn get_finds_by_slug() {
        let (_td, dir) = fixture();
        let agent = get(&dir, "reviewer").unwrap();
        assert_eq!(agent.frontmatter.name.as_deref(), Some("Reviewer"));
        assert_eq!(agent.body, "You review code.\n");
        assert!(agent.has_memory);
    }

    #[test]
    fn get_returns_not_found() {
        let (_td, dir) = fixture();
        let err = get(&dir, "missing").unwrap_err();
        assert_eq!(err.code, ErrorCode::NotFound);
    }

    #[test]
    fn skill_counts_match() {
        let (_td, dir) = fixture();
        let counts = skill_counts(&dir).unwrap();
        assert_eq!(counts.get("reviewer").copied(), Some(1));
        assert_eq!(counts.get("helper").copied(), Some(0));
    }

    #[test]
    fn empty_dir_returns_empty_list() {
        let td = tempfile::tempdir().unwrap();
        assert!(list(td.path()).unwrap().is_empty());
    }

    #[test]
    fn create_then_get_round_trips() {
        let td = tempfile::tempdir().unwrap();
        let input = AgentInput {
            slug: "new-agent".into(),
            directory: String::new(),
            frontmatter: AgentFrontmatter {
                name: Some("New".into()),
                ..Default::default()
            },
            body: "hello\n".into(),
        };
        let created = create(td.path(), input).unwrap();
        assert_eq!(created.slug, "new-agent");
        let fetched = get(td.path(), "new-agent").unwrap();
        assert_eq!(fetched.body.trim_end(), "hello");
    }

    #[test]
    fn create_rejects_duplicate() {
        let (_td, dir) = fixture();
        let input = AgentInput {
            slug: "reviewer".into(),
            directory: String::new(),
            frontmatter: AgentFrontmatter::default(),
            body: String::new(),
        };
        assert_eq!(create(&dir, input).unwrap_err().code, ErrorCode::InvalidInput);
    }

    #[test]
    fn update_can_rename_slug() {
        let (_td, dir) = fixture();
        let input = AgentInput {
            slug: "renamed".into(),
            directory: String::new(),
            frontmatter: AgentFrontmatter {
                name: Some("Renamed".into()),
                ..Default::default()
            },
            body: "body\n".into(),
        };
        let updated = update(&dir, "reviewer", input).unwrap();
        assert_eq!(updated.slug, "renamed");
        assert_eq!(get(&dir, "reviewer").unwrap_err().code, ErrorCode::NotFound);
    }

    #[test]
    fn delete_removes_file() {
        let (_td, dir) = fixture();
        delete(&dir, "reviewer").unwrap();
        assert_eq!(get(&dir, "reviewer").unwrap_err().code, ErrorCode::NotFound);
    }

    #[test]
    fn export_returns_raw_source() {
        let (_td, dir) = fixture();
        let raw = export(&dir, "reviewer").unwrap();
        assert!(raw.starts_with("---"));
        assert!(raw.contains("Reviewer"));
    }

    #[test]
    fn import_parses_and_writes() {
        let td = tempfile::tempdir().unwrap();
        let payload = AgentImport {
            slug: "imported".into(),
            directory: String::new(),
            content: "---\nname: Imported\n---\n\nfrom outside\n".into(),
        };
        let agent = import(td.path(), payload).unwrap();
        assert_eq!(agent.frontmatter.name.as_deref(), Some("Imported"));
    }
}
