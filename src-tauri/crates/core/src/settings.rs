//! Read+write logic for `~/.claude/settings.json`. Tolerant.

use std::path::Path;

use crate::io;
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

pub fn put(claude_dir: &Path, settings: &Settings) -> Result<(), AppError> {
    let path = claude_dir.join("settings.json");
    let serialized = serde_json::to_string_pretty(settings)?;
    io::atomic_write(&path, serialized.as_bytes())
}

/// Project-scoped settings under `<wd>/.claude/settings.json`.
pub fn project_get(working_dir: &Path) -> Result<Settings, AppError> {
    let path = working_dir.join(".claude").join("settings.json");
    if !path.is_file() {
        return Ok(Settings::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn project_put(working_dir: &Path, settings: &Settings) -> Result<(), AppError> {
    let path = working_dir.join(".claude").join("settings.json");
    let serialized = serde_json::to_string_pretty(settings)?;
    io::atomic_write(&path, serialized.as_bytes())
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
    fn put_round_trip_preserves_extras() {
        let td = tempfile::tempdir().unwrap();
        let mut s = Settings::default();
        s.default_model = Some("sonnet".into());
        s.extra
            .insert("custom".into(), serde_json::json!({"x": 1}));
        put(td.path(), &s).unwrap();
        let got = get(td.path()).unwrap();
        assert_eq!(got.default_model.as_deref(), Some("sonnet"));
        assert!(got.extra.contains_key("custom"));
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
