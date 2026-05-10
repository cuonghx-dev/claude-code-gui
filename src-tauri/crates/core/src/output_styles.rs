//! Read-side logic for output styles. Phase 1: list + get with dual scope.
//!
//! Global: `~/.claude/output-styles/<id>.md`.
//! Project: `<project>/.claude/output-styles/<id>.md`.

use std::path::Path;

use crate::frontmatter::{self, Document};
use crate::types::{OutputStyle, OutputStyleFrontmatter, OutputStyleScope};
use crate::AppError;

const SUBDIR: &str = "output-styles";

pub fn list(claude_dir: &Path, working_dir: Option<&Path>) -> Result<Vec<OutputStyle>, AppError> {
    let mut out = Vec::new();
    list_into(&claude_dir.join(SUBDIR), OutputStyleScope::Global, &mut out)?;
    if let Some(wd) = working_dir {
        list_into(&wd.join(".claude").join(SUBDIR), OutputStyleScope::Project, &mut out)?;
    }
    out.sort_by(|a, b| (a.scope as u8, a.id.clone()).cmp(&(b.scope as u8, b.id.clone())));
    Ok(out)
}

pub fn get(
    claude_dir: &Path,
    id: &str,
    scope: OutputStyleScope,
    working_dir: Option<&Path>,
) -> Result<OutputStyle, AppError> {
    list(claude_dir, working_dir)?
        .into_iter()
        .find(|s| s.id == id && s.scope == scope)
        .ok_or_else(|| AppError::not_found(format!("output style '{id}' not found in {scope:?}")))
}

fn list_into(root: &Path, scope: OutputStyleScope, out: &mut Vec<OutputStyle>) -> Result<(), AppError> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)?.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        match read_one(&path, scope) {
            Ok(s) => out.push(s),
            Err(e) => tracing::warn!(error = %e, path = %path.display(), "skipping unparseable output style"),
        }
    }
    Ok(())
}

fn read_one(path: &Path, scope: OutputStyleScope) -> Result<OutputStyle, AppError> {
    let src = std::fs::read_to_string(path)?;
    let Document { frontmatter, body }: Document<OutputStyleFrontmatter> = frontmatter::parse(&src)?;
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    Ok(OutputStyle {
        id,
        scope,
        frontmatter,
        body,
        file_path: path.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_scope() {
        let td = tempfile::tempdir().unwrap();
        let global = td.path().join("output-styles");
        std::fs::create_dir_all(&global).unwrap();
        std::fs::write(global.join("concise.md"), "---\nname: Concise\n---\n\nbe terse").unwrap();

        let project = td.path().join("proj");
        let project_styles = project.join(".claude/output-styles");
        std::fs::create_dir_all(&project_styles).unwrap();
        std::fs::write(project_styles.join("verbose.md"), "---\nname: Verbose\n---\n\n").unwrap();

        let styles = list(td.path(), Some(&project)).unwrap();
        assert_eq!(styles.len(), 2);
        assert_eq!(styles[0].scope, OutputStyleScope::Global);
        assert_eq!(styles[1].scope, OutputStyleScope::Project);
    }
}
