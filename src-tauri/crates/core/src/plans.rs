//! Read-side logic for `~/.claude/plans/`. Plain markdown — no frontmatter.

use std::path::Path;
use std::time::SystemTime;

use crate::types::Plan;
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
}
