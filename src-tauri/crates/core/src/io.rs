//! Filesystem write helpers shared by every domain CRUD module.
//!
//! Atomic write: temp file in same dir, fsync, rename. Avoids torn writes
//! and partial files visible to readers (the file watcher would otherwise
//! emit a `create` then `modify` for every save).

use std::path::Path;

use crate::{AppError, ErrorCode};

/// Replace `path` atomically. Same-directory tempfile + rename. Caller is
/// responsible for ensuring the parent dir exists.
pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::invalid(format!("path has no parent: {}", path.display())))?;
    std::fs::create_dir_all(parent)?;

    let tmp = path.with_extension(format!(
        "{}.tmp.{}",
        path.extension().and_then(|s| s.to_str()).unwrap_or(""),
        uuid::Uuid::new_v4().simple()
    ));
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Remove a file. NotFound is propagated (callers decide if it's an error).
pub fn remove_file(path: &Path) -> Result<(), AppError> {
    std::fs::remove_file(path)?;
    Ok(())
}

/// Recursively remove a directory.
pub fn remove_dir_all(path: &Path) -> Result<(), AppError> {
    std::fs::remove_dir_all(path)?;
    Ok(())
}

/// Slug validator. Allowed: `[a-z0-9-_]`, 1-64 chars, no leading/trailing `-`,
/// no `..`, no path separators. Rejects empty.
pub fn validate_slug(slug: &str) -> Result<(), AppError> {
    if slug.is_empty() {
        return Err(AppError::invalid("slug cannot be empty"));
    }
    if slug.len() > 64 {
        return Err(AppError::invalid("slug too long (max 64)"));
    }
    if slug.contains("..") || slug.contains('/') || slug.contains('\\') {
        return Err(AppError::invalid("slug must not contain path separators or '..'"));
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(AppError::invalid("slug must not start or end with '-'"));
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::invalid(
            "slug must only contain ASCII letters, digits, '-', '_'",
        ));
    }
    Ok(())
}

/// Validate a relative subdirectory under a domain root (e.g. `agents/`).
/// Empty allowed. Forbids absolute paths and `..` traversal.
pub fn validate_relative_dir(rel: &str) -> Result<(), AppError> {
    if rel.is_empty() {
        return Ok(());
    }
    if rel.starts_with('/') || rel.starts_with('\\') {
        return Err(AppError::invalid("directory must be relative"));
    }
    for seg in rel.split(['/', '\\']) {
        if seg.is_empty() || seg == "." || seg == ".." {
            return Err(AppError::invalid("directory contains invalid segment"));
        }
        if !seg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(AppError::new(
                ErrorCode::InvalidInput,
                "directory segment has invalid characters",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_creates_and_replaces() {
        let td = tempfile::tempdir().unwrap();
        let f = td.path().join("nested/foo.md");
        atomic_write(&f, b"hello").unwrap();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "hello");
        atomic_write(&f, b"world").unwrap();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "world");
    }

    #[test]
    fn slug_rejects_garbage() {
        assert!(validate_slug("ok-slug_1").is_ok());
        assert!(validate_slug("").is_err());
        assert!(validate_slug("..").is_err());
        assert!(validate_slug("a/b").is_err());
        assert!(validate_slug("-x").is_err());
        assert!(validate_slug("x-").is_err());
        assert!(validate_slug("foo bar").is_err());
        assert!(validate_slug(&"x".repeat(65)).is_err());
    }

    #[test]
    fn rel_dir_rejects_traversal() {
        assert!(validate_relative_dir("").is_ok());
        assert!(validate_relative_dir("a/b").is_ok());
        assert!(validate_relative_dir("a/../b").is_err());
        assert!(validate_relative_dir("/abs").is_err());
        assert!(validate_relative_dir("a/.b").is_ok()); // leading dot OK
    }
}
