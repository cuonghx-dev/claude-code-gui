//! Read-side logic for `~/.claude/settings.json`. Tolerant.

use std::path::Path;

use crate::types::Settings;
use crate::AppError;

pub fn get(claude_dir: &Path) -> Result<Settings, AppError> {
    let path = claude_dir.join("settings.json");
    if !path.is_file() {
        return Ok(Settings::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&raw)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_returns_default() {
        let td = tempfile::tempdir().unwrap();
        let s = get(td.path()).unwrap();
        assert!(s.default_model.is_none());
    }

    #[test]
    fn parses_and_preserves_extras() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"defaultModel":"sonnet","customField":42}"#,
        )
        .unwrap();
        let s = get(td.path()).unwrap();
        assert_eq!(s.default_model.as_deref(), Some("sonnet"));
        assert!(s.extra.contains_key("customField"));
    }
}
