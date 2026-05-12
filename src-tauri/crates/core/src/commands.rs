//! Read+write logic for `~/.claude/commands/`. CRUD mirrors `agents` (markdown
//! file with YAML frontmatter, slug = filename stem). No import/export by spec.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::frontmatter::{self, Document};
use crate::io;
use crate::types::{Command, CommandFrontmatter, CommandInput};
use crate::{AppError, ErrorCode};

const COMMANDS_SUBDIR: &str = "commands";

pub fn list(claude_dir: &Path) -> Result<Vec<Command>, AppError> {
    let root = claude_dir.join(COMMANDS_SUBDIR);
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
            Ok(c) => out.push(c),
            Err(e) => tracing::warn!(error = %e, path = %path.display(), "skipping unparseable command"),
        }
    }
    out.sort_by(|a, b| (&a.directory, &a.slug).cmp(&(&b.directory, &b.slug)));
    Ok(out)
}

pub fn get(claude_dir: &Path, slug: &str) -> Result<Command, AppError> {
    list(claude_dir)?
        .into_iter()
        .find(|c| c.slug == slug)
        .ok_or_else(|| AppError::not_found(format!("command '{slug}' not found")))
}

fn read_one(root: &Path, path: &Path) -> Result<Command, AppError> {
    let src = std::fs::read_to_string(path)?;
    let Document { frontmatter, body }: Document<CommandFrontmatter> = frontmatter::parse(&src)?;

    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::new(ErrorCode::InvalidInput, "invalid command filename"))?
        .to_string();

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();

    let directory = path
        .parent()
        .and_then(|p| p.strip_prefix(root).ok())
        .map(|r| {
            r.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_default();

    Ok(Command {
        slug,
        filename,
        directory,
        frontmatter,
        body,
        file_path: path.to_string_lossy().into_owned(),
    })
}

pub fn create(claude_dir: &Path, input: CommandInput) -> Result<Command, AppError> {
    io::validate_slug(&input.slug)?;
    io::validate_relative_dir(&input.directory)?;
    let path = file_path(claude_dir, &input.directory, &input.slug);
    if path.exists() {
        return Err(AppError::invalid(format!(
            "command '{}' already exists",
            input.slug
        )));
    }
    write_doc(&path, &input)?;
    get(claude_dir, &input.slug)
}

pub fn update(claude_dir: &Path, slug: &str, input: CommandInput) -> Result<Command, AppError> {
    io::validate_slug(&input.slug)?;
    io::validate_relative_dir(&input.directory)?;
    let existing = get(claude_dir, slug)?;
    let new_path = file_path(claude_dir, &input.directory, &input.slug);
    let old_path = PathBuf::from(&existing.file_path);
    if old_path != new_path && new_path.exists() {
        return Err(AppError::invalid(format!(
            "target slug '{}' already exists",
            input.slug
        )));
    }
    write_doc(&new_path, &input)?;
    if old_path != new_path {
        let _ = io::remove_file(&old_path);
    }
    get(claude_dir, &input.slug)
}

/// Import a command from raw markdown source: validates frontmatter parses,
/// then writes verbatim. Mirrors `agents::import`.
pub fn import_raw(
    claude_dir: &Path,
    slug: &str,
    directory: &str,
    content: &str,
) -> Result<Command, AppError> {
    io::validate_slug(slug)?;
    io::validate_relative_dir(directory)?;
    let _: Document<CommandFrontmatter> = frontmatter::parse(content)?;
    let path = file_path(claude_dir, directory, slug);
    if path.exists() {
        return Err(AppError::invalid(format!(
            "command '{slug}' already exists"
        )));
    }
    io::atomic_write(&path, content.as_bytes())?;
    get(claude_dir, slug)
}

/// Update an existing command by replacing file contents with `content`.
/// Validates that `content` parses before writing.
pub fn update_raw(claude_dir: &Path, slug: &str, content: &str) -> Result<Command, AppError> {
    let existing = get(claude_dir, slug)?;
    let _: Document<CommandFrontmatter> = frontmatter::parse(content)?;
    io::atomic_write(Path::new(&existing.file_path), content.as_bytes())?;
    get(claude_dir, slug)
}

/// Return on-disk markdown source for a command.
pub fn export(claude_dir: &Path, slug: &str) -> Result<String, AppError> {
    let c = get(claude_dir, slug)?;
    let raw = std::fs::read_to_string(&c.file_path)?;
    Ok(raw)
}

pub fn delete(claude_dir: &Path, slug: &str) -> Result<(), AppError> {
    let existing = get(claude_dir, slug)?;
    io::remove_file(Path::new(&existing.file_path))
}

fn file_path(claude_dir: &Path, directory: &str, slug: &str) -> PathBuf {
    let mut p = claude_dir.join(COMMANDS_SUBDIR);
    if !directory.is_empty() {
        p.push(directory);
    }
    p.push(format!("{slug}.md"));
    p
}

fn write_doc(path: &Path, input: &CommandInput) -> Result<(), AppError> {
    let doc = Document {
        frontmatter: input.frontmatter.clone(),
        body: input.body.clone(),
    };
    let serialized = frontmatter::serialize(&doc)?;
    io::atomic_write(path, serialized.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_and_get() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path().join("commands");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("review-pr.md"),
            "---\nname: review-pr\ndescription: Review the current PR\nargument-hint: \"[pr-number]\"\nallowed-tools: [Read, Bash]\nagent: code-reviewer\n---\n\nRun a review against {{args}}\n",
        )
        .unwrap();

        let cmds = list(td.path()).unwrap();
        assert_eq!(cmds.len(), 1);
        let c = get(td.path(), "review-pr").unwrap();
        assert_eq!(c.frontmatter.agent.as_deref(), Some("code-reviewer"));
        assert_eq!(c.frontmatter.argument_hint.as_deref(), Some("[pr-number]"));
        assert_eq!(c.frontmatter.allowed_tools, vec!["Read", "Bash"]);
    }

    #[test]
    fn create_update_delete() {
        let td = tempfile::tempdir().unwrap();
        let input = CommandInput {
            slug: "deploy".into(),
            directory: String::new(),
            frontmatter: CommandFrontmatter {
                name: Some("deploy".into()),
                ..Default::default()
            },
            body: "deploy now".into(),
        };
        let created = create(td.path(), input).unwrap();
        assert_eq!(created.slug, "deploy");

        let updated = update(
            td.path(),
            "deploy",
            CommandInput {
                slug: "shipit".into(),
                directory: String::new(),
                frontmatter: CommandFrontmatter::default(),
                body: "go".into(),
            },
        )
        .unwrap();
        assert_eq!(updated.slug, "shipit");
        assert!(get(td.path(), "deploy").is_err());

        delete(td.path(), "shipit").unwrap();
        assert!(get(td.path(), "shipit").is_err());
    }
}
