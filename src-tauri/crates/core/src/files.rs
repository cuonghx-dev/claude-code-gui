//! Filesystem utility commands. Frontend uses these for the project picker
//! (`directories_list` populates the breadcrumb dropdown) and the settings
//! page (`files_read` shows hooks JSON inline).

use std::path::{Path, PathBuf};

use crate::types::DirEntry;
use crate::AppError;

/// Shallow directory listing. Sorted dirs-first, hidden entries excluded.
pub fn directories_list(parent: &Path) -> Result<Vec<DirEntry>, AppError> {
    if !parent.is_dir() {
        return Err(AppError::not_found(format!(
            "not a directory: {}",
            parent.display()
        )));
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(parent)?.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) if !n.starts_with('.') => n.to_string(),
            _ => continue,
        };
        out.push(DirEntry {
            name,
            path: path.to_string_lossy().into_owned(),
            is_dir: path.is_dir(),
        });
    }
    out.sort_by(|a, b| (b.is_dir, a.name.clone()).cmp(&(a.is_dir, b.name.clone())));
    Ok(out)
}

/// Read a UTF-8 text file. Caller is responsible for not asking for binaries.
/// Caps at 8 MiB to keep WebView serialization predictable.
pub fn files_read(path: &Path) -> Result<String, AppError> {
    let meta = std::fs::metadata(path)?;
    const MAX: u64 = 8 * 1024 * 1024;
    if meta.len() > MAX {
        return Err(AppError::invalid(format!(
            "file too large: {} bytes (max {MAX})",
            meta.len()
        )));
    }
    let raw = std::fs::read_to_string(path)?;
    Ok(raw)
}

/// Resolve `~` to `$HOME` for user-typed paths.
pub fn expand_tilde(input: &str) -> PathBuf {
    if let Some(rest) = input.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_dirs_first_excluding_dotfiles() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir(td.path().join("sub")).unwrap();
        std::fs::write(td.path().join("file.txt"), "x").unwrap();
        std::fs::write(td.path().join(".hidden"), "x").unwrap();
        let entries = directories_list(td.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0].is_dir);
    }

    #[test]
    fn read_round_trip() {
        let td = tempfile::tempdir().unwrap();
        let f = td.path().join("a.txt");
        std::fs::write(&f, "hello").unwrap();
        assert_eq!(files_read(&f).unwrap(), "hello");
    }
}
