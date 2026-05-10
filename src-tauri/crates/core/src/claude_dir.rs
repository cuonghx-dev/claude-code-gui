use std::path::{Path, PathBuf};

use crate::{AppError, ErrorCode};

/// Resolve the `~/.claude/` directory. Precedence:
/// 1. `CLAUDE_DIR` env var
/// 2. caller-supplied override (e.g., from `AppConfig.claude_dir_override`)
/// 3. `$HOME/.claude`
pub fn resolve(override_path: Option<&Path>) -> Result<PathBuf, AppError> {
    if let Ok(env) = std::env::var("CLAUDE_DIR") {
        return Ok(PathBuf::from(env));
    }
    if let Some(p) = override_path {
        return Ok(p.to_path_buf());
    }
    home()
        .map(|h| h.join(".claude"))
        .ok_or_else(|| AppError::new(ErrorCode::Internal, "could not resolve $HOME"))
}

/// Ensure the directory exists. Idempotent.
pub fn ensure(path: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_override_wins() {
        let td = tempfile::tempdir().unwrap();
        std::env::set_var("CLAUDE_DIR", td.path());
        let resolved = resolve(None).unwrap();
        assert_eq!(resolved, td.path());
        std::env::remove_var("CLAUDE_DIR");
    }

    #[test]
    fn explicit_override_used_when_no_env() {
        std::env::remove_var("CLAUDE_DIR");
        let custom = PathBuf::from("/tmp/custom-claude");
        let resolved = resolve(Some(&custom)).unwrap();
        assert_eq!(resolved, custom);
    }

    #[test]
    fn ensure_creates_dir() {
        let td = tempfile::tempdir().unwrap();
        let nested = td.path().join("a/b/c");
        ensure(&nested).unwrap();
        assert!(nested.is_dir());
    }
}
