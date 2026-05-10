//! Read-side logic for `~/.claude/commands/`. Phase 1: list + get.

use std::path::Path;

use walkdir::WalkDir;

use crate::frontmatter::{self, Document};
use crate::types::{Command, CommandFrontmatter};
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
}
