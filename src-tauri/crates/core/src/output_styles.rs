//! Read+write logic for output styles with dual scope.
//!
//! Global: `~/.claude/output-styles/<id>.md`.
//! Project: `<project>/.claude/output-styles/<id>.md`.

use std::path::{Path, PathBuf};

use crate::frontmatter::{self, Document};
use crate::io;
use crate::types::{OutputStyle, OutputStyleFrontmatter, OutputStyleInput, OutputStyleScope};
use crate::AppError;

const SUBDIR: &str = "output-styles";

pub fn list(claude_dir: &Path, working_dir: Option<&Path>) -> Result<Vec<OutputStyle>, AppError> {
    let mut out = builtin_styles();
    list_into(&claude_dir.join(SUBDIR), OutputStyleScope::Global, &mut out)?;
    if let Some(wd) = working_dir {
        list_into(&wd.join(".claude").join(SUBDIR), OutputStyleScope::Project, &mut out)?;
    }
    out.sort_by(|a, b| (a.scope as u8, a.id.clone()).cmp(&(b.scope as u8, b.id.clone())));
    Ok(out)
}

fn builtin_styles() -> Vec<OutputStyle> {
    use crate::types::OutputStyleFrontmatter;
    let mk = |id: &str, name: &str, desc: &str| OutputStyle {
        id: id.to_string(),
        scope: OutputStyleScope::Builtin,
        frontmatter: OutputStyleFrontmatter {
            name: Some(name.to_string()),
            description: Some(desc.to_string()),
            keep_coding_instructions: Some(true),
            extra: Default::default(),
        },
        body: String::new(),
        file_path: String::new(),
    };
    vec![
        mk(
            "default",
            "Default",
            "Claude Code's standard output style, designed for efficient software engineering tasks.",
        ),
        mk(
            "explanatory",
            "Explanatory",
            "Provides educational \"Insights\" to help you understand implementation choices and codebase patterns.",
        ),
        mk(
            "learning",
            "Learning",
            "Collaborative, learn-by-doing mode. Claude will share \"Insights\" and ask you to contribute code yourself using TODO(human) markers.",
        ),
    ]
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

pub fn create(claude_dir: &Path, input: OutputStyleInput) -> Result<OutputStyle, AppError> {
    if input.scope == OutputStyleScope::Builtin {
        return Err(AppError::invalid("cannot create a built-in output style"));
    }
    io::validate_slug(&input.id)?;
    let dir = scope_dir(claude_dir, input.scope, input.working_dir.as_deref())?;
    let path = dir.join(format!("{}.md", input.id));
    if path.exists() {
        return Err(AppError::invalid(format!(
            "output style '{}' already exists in {:?}",
            input.id, input.scope
        )));
    }
    let doc = Document {
        frontmatter: input.frontmatter.clone(),
        body: input.body.clone(),
    };
    let serialized = frontmatter::serialize(&doc)?;
    io::atomic_write(&path, serialized.as_bytes())?;
    let wd = input.working_dir.as_deref().map(Path::new);
    get(claude_dir, &input.id, input.scope, wd)
}

pub fn delete(
    claude_dir: &Path,
    id: &str,
    scope: OutputStyleScope,
    working_dir: Option<&Path>,
) -> Result<(), AppError> {
    if scope == OutputStyleScope::Builtin {
        return Err(AppError::invalid("cannot delete a built-in output style"));
    }
    let existing = get(claude_dir, id, scope, working_dir)?;
    io::remove_file(Path::new(&existing.file_path))
}

fn scope_dir(
    claude_dir: &Path,
    scope: OutputStyleScope,
    working_dir: Option<&str>,
) -> Result<PathBuf, AppError> {
    Ok(match scope {
        OutputStyleScope::Builtin => {
            return Err(AppError::invalid("built-in output styles have no filesystem path"));
        }
        OutputStyleScope::Global => claude_dir.join(SUBDIR),
        OutputStyleScope::Project => {
            let wd = working_dir.ok_or_else(|| {
                AppError::invalid("project-scoped output style requires workingDir")
            })?;
            PathBuf::from(wd).join(".claude").join(SUBDIR)
        }
    })
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
        let user: Vec<_> = styles
            .iter()
            .filter(|s| s.scope != OutputStyleScope::Builtin)
            .collect();
        assert_eq!(user.len(), 2);
        assert_eq!(user[0].scope, OutputStyleScope::Global);
        assert_eq!(user[1].scope, OutputStyleScope::Project);
        let builtins: Vec<_> = styles
            .iter()
            .filter(|s| s.scope == OutputStyleScope::Builtin)
            .collect();
        assert_eq!(builtins.len(), 3);
    }

    #[test]
    fn builtins_are_read_only() {
        let td = tempfile::tempdir().unwrap();
        let err = create(
            td.path(),
            OutputStyleInput {
                id: "default".into(),
                scope: OutputStyleScope::Builtin,
                working_dir: None,
                frontmatter: OutputStyleFrontmatter::default(),
                body: String::new(),
            },
        )
        .unwrap_err();
        assert!(format!("{err:?}").to_lowercase().contains("built-in"));

        let err = delete(td.path(), "default", OutputStyleScope::Builtin, None).unwrap_err();
        assert!(format!("{err:?}").to_lowercase().contains("built-in"));
    }

    #[test]
    fn create_and_delete_global() {
        let td = tempfile::tempdir().unwrap();
        let input = OutputStyleInput {
            id: "terse".into(),
            scope: OutputStyleScope::Global,
            working_dir: None,
            frontmatter: OutputStyleFrontmatter {
                name: Some("Terse".into()),
                ..Default::default()
            },
            body: "be terse".into(),
        };
        create(td.path(), input).unwrap();
        let s = get(td.path(), "terse", OutputStyleScope::Global, None).unwrap();
        assert_eq!(s.frontmatter.name.as_deref(), Some("Terse"));

        delete(td.path(), "terse", OutputStyleScope::Global, None).unwrap();
        assert!(get(td.path(), "terse", OutputStyleScope::Global, None).is_err());
    }
}
