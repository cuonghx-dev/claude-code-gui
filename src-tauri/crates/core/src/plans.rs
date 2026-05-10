//! Read+write logic for `~/.claude/plans/`. Plain markdown — no frontmatter.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::io;
use crate::types::{Plan, PlanInput};
use crate::AppError;

const PLANS_SUBDIR: &str = "plans";

pub fn list(claude_dir: &Path) -> Result<Vec<Plan>, AppError> {
    let root = claude_dir.join(PLANS_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&root)?.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        match read_one(&path) {
            Ok(p) => out.push(p),
            Err(e) => tracing::warn!(error = %e, path = %path.display(), "skipping unreadable plan"),
        }
    }
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

pub fn get(claude_dir: &Path, slug: &str) -> Result<Plan, AppError> {
    list(claude_dir)?
        .into_iter()
        .find(|p| p.slug == slug)
        .ok_or_else(|| AppError::not_found(format!("plan '{slug}' not found")))
}

fn read_one(path: &Path) -> Result<Plan, AppError> {
    let body = std::fs::read_to_string(path)?;
    let slug = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let title = first_heading(&body).unwrap_or_else(|| slug.clone());
    let meta = std::fs::metadata(path)?;
    let modified_at = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::<chrono::Utc>::from_timestamp(d.as_secs() as i64, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    Ok(Plan {
        slug,
        filename,
        body,
        file_path: path.to_string_lossy().into_owned(),
        title,
        size_bytes: meta.len(),
        modified_at,
    })
}

pub fn create(claude_dir: &Path, input: PlanInput) -> Result<Plan, AppError> {
    io::validate_slug(&input.slug)?;
    let path = file_path(claude_dir, &input.slug);
    if path.exists() {
        return Err(AppError::invalid(format!(
            "plan '{}' already exists",
            input.slug
        )));
    }
    io::atomic_write(&path, input.body.as_bytes())?;
    get(claude_dir, &input.slug)
}

pub fn update(claude_dir: &Path, slug: &str, input: PlanInput) -> Result<Plan, AppError> {
    io::validate_slug(&input.slug)?;
    let existing = get(claude_dir, slug)?;
    let new_path = file_path(claude_dir, &input.slug);
    let old_path = PathBuf::from(&existing.file_path);
    if old_path != new_path && new_path.exists() {
        return Err(AppError::invalid(format!(
            "target slug '{}' already exists",
            input.slug
        )));
    }
    io::atomic_write(&new_path, input.body.as_bytes())?;
    if old_path != new_path {
        let _ = io::remove_file(&old_path);
    }
    get(claude_dir, &input.slug)
}

pub fn delete(claude_dir: &Path, slug: &str) -> Result<(), AppError> {
    let existing = get(claude_dir, slug)?;
    io::remove_file(Path::new(&existing.file_path))
}

fn file_path(claude_dir: &Path, slug: &str) -> PathBuf {
    claude_dir.join(PLANS_SUBDIR).join(format!("{slug}.md"))
}

fn first_heading(md: &str) -> Option<String> {
    md.lines()
        .find_map(|l| l.strip_prefix("# ").map(str::trim).map(str::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_and_get() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path().join("plans");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("first.md"), "# First Plan\n\nbody").unwrap();
        std::fs::write(dir.join("second.md"), "no heading").unwrap();

        let plans = list(td.path()).unwrap();
        assert_eq!(plans.len(), 2);
        let first = get(td.path(), "first").unwrap();
        assert_eq!(first.title, "First Plan");
        let second = get(td.path(), "second").unwrap();
        assert_eq!(second.title, "second");
    }

    #[test]
    fn write_round_trip() {
        let td = tempfile::tempdir().unwrap();
        create(
            td.path(),
            PlanInput {
                slug: "draft".into(),
                body: "# Draft\n\nbody".into(),
            },
        )
        .unwrap();
        let p = get(td.path(), "draft").unwrap();
        assert_eq!(p.title, "Draft");

        update(
            td.path(),
            "draft",
            PlanInput {
                slug: "final".into(),
                body: "# Final".into(),
            },
        )
        .unwrap();
        assert!(get(td.path(), "draft").is_err());
        assert_eq!(get(td.path(), "final").unwrap().title, "Final");

        delete(td.path(), "final").unwrap();
        assert!(get(td.path(), "final").is_err());
    }
}
