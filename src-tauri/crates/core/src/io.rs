//! Filesystem write helpers shared by every domain CRUD module.
//!
//! Atomic write: temp file in same dir, fsync, rename. Avoids torn writes
//! and partial files visible to readers (the file watcher would otherwise
//! emit a `create` then `modify` for every save).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

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
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(contents)?;
        // The module doc has always promised fsync; without it a crash between
        // rename and writeback leaves a zero-length file on some filesystems.
        f.sync_all()?;
    }
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

/// A JSON file as read from disk: parsed value, original bytes, and the mtime
/// used for optimistic-concurrency checks on write.
///
/// A missing file is not an error — it yields `exists: false` and an empty
/// object, so callers can patch a file into existence.
#[derive(Debug, Clone)]
pub struct JsonDoc {
    pub value: serde_json::Value,
    pub raw: String,
    pub mtime_ms: Option<i64>,
    pub exists: bool,
}

pub fn read_json_doc(path: &Path) -> Result<JsonDoc, AppError> {
    if !path.is_file() {
        return Ok(JsonDoc {
            value: serde_json::Value::Object(Default::default()),
            raw: String::new(),
            mtime_ms: None,
            exists: false,
        });
    }
    let raw = std::fs::read_to_string(path)?;
    let value = serde_json::from_str(&raw)?;
    Ok(JsonDoc {
        value,
        raw,
        mtime_ms: mtime_ms(path),
        exists: true,
    })
}

/// Modification time in epoch milliseconds. `None` when the file is missing or
/// the platform has no usable mtime.
pub fn mtime_ms(path: &Path) -> Option<i64> {
    let meta = std::fs::metadata(path).ok()?;
    let dur = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    Some(dur.as_millis() as i64)
}

/// Optimistic-concurrency guard. The Claude CLI writes the same files while the
/// GUI is open and honors no lock protocol, so every write re-checks the mtime
/// the caller last read and refuses to clobber a newer version.
///
/// `expected: None` means "no expectation" and always passes.
pub fn check_unchanged(path: &Path, expected: Option<i64>) -> Result<(), AppError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let current = mtime_ms(path);
    if current == Some(expected) {
        return Ok(());
    }
    Err(AppError::new(
        ErrorCode::Conflict,
        format!(
            "{} changed on disk since it was read — reload before saving",
            path.display()
        ),
    ))
}

/// Copy `path` to `<name>.ccg.bak` before overwriting it. Returns `None` when
/// there was nothing to back up.
///
/// The `.ccg` infix matters: the CLI owns `<name>.bak` and `<name>.bak.<stamp>`
/// next to its own settings files, and must not have them overwritten.
pub fn backup_sibling(path: &Path) -> Result<Option<PathBuf>, AppError> {
    if !path.is_file() {
        return Ok(None);
    }
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::invalid(format!("path has no file name: {}", path.display())))?;
    let dest = path.with_file_name(format!("{name}.ccg.bak"));
    std::fs::copy(path, &dest)?;
    Ok(Some(dest))
}

/// Read-modify-write helper for JSON config files: concurrency check, backup,
/// pretty-print with a trailing newline, atomic replace. Returns the new mtime
/// so the caller can hand it straight back to the frontend.
///
/// Key order is preserved because `serde_json` is built with `preserve_order`.
pub fn atomic_write_json(
    path: &Path,
    value: &serde_json::Value,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    check_unchanged(path, expected_mtime_ms)?;
    backup_sibling(path)?;
    let mut serialized = serde_json::to_string_pretty(value)?;
    serialized.push('\n');
    atomic_write(path, serialized.as_bytes())?;
    Ok(mtime_ms(path).unwrap_or(0))
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
    fn read_json_doc_tolerates_missing_file() {
        let td = tempfile::tempdir().unwrap();
        let doc = read_json_doc(&td.path().join("nope.json")).unwrap();
        assert!(!doc.exists);
        assert!(doc.mtime_ms.is_none());
        assert_eq!(doc.value, serde_json::json!({}));
    }

    #[test]
    fn write_json_preserves_key_order() {
        let td = tempfile::tempdir().unwrap();
        let f = td.path().join("settings.json");
        // Deliberately not alphabetical: `preserve_order` must keep it as-is.
        let v: serde_json::Value = serde_json::from_str(r#"{"zeta":1,"alpha":2,"mid":3}"#).unwrap();
        atomic_write_json(&f, &v, None).unwrap();
        let raw = std::fs::read_to_string(&f).unwrap();
        assert!(raw.find("zeta").unwrap() < raw.find("alpha").unwrap());
        assert!(raw.ends_with("}\n"), "expected trailing newline");
    }

    #[test]
    fn backup_written_only_when_file_exists() {
        let td = tempfile::tempdir().unwrap();
        let f = td.path().join("settings.json");
        assert!(backup_sibling(&f).unwrap().is_none());

        atomic_write_json(&f, &serde_json::json!({"a": 1}), None).unwrap();
        atomic_write_json(&f, &serde_json::json!({"a": 2}), None).unwrap();

        let bak = std::fs::read_to_string(td.path().join("settings.json.ccg.bak")).unwrap();
        assert!(bak.contains("\"a\": 1"), "backup holds the pre-write content");
    }

    #[test]
    fn write_rejects_stale_mtime() {
        let td = tempfile::tempdir().unwrap();
        let f = td.path().join("settings.json");
        let mtime = atomic_write_json(&f, &serde_json::json!({"a": 1}), None).unwrap();

        // No expectation always passes.
        assert!(check_unchanged(&f, None).is_ok());
        assert!(check_unchanged(&f, Some(mtime)).is_ok());

        let err = atomic_write_json(&f, &serde_json::json!({"a": 2}), Some(mtime - 1)).unwrap_err();
        assert_eq!(err.code, ErrorCode::Conflict);
        // The rejected write must not have touched the file.
        assert!(std::fs::read_to_string(&f).unwrap().contains("\"a\": 1"));
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
