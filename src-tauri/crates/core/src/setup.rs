//! First-run wizard backend.
//!
//! Persists wizard answers to two places:
//! - `<claude_dir>/settings.json` for fields the Claude CLI itself reads
//!   (`defaultModel`, `defaultPermissionMode`).
//! - `AppConfig` for fields private to claude-code-gui (`theme`,
//!   `claude_dir_override`). The caller is responsible for writing the
//!   `AppConfig` to its store; this module returns the merged value so the
//!   IPC layer can hand it to `tauri-plugin-store`.

use std::path::Path;

use crate::settings;
use crate::types::{AppConfig, SetupPayload, Settings};
use crate::AppError;

/// Apply a `SetupPayload`. Always marks `onboardingCompleted = true` so the
/// frontend stops showing the wizard. Returns the updated `AppConfig` for
/// the IPC layer to persist via `tauri-plugin-store`.
pub fn finalize(
    claude_dir: &Path,
    config: AppConfig,
    payload: SetupPayload,
) -> Result<(AppConfig, Settings), AppError> {
    let mut s = settings::get(claude_dir)?;
    if let Some(m) = payload.default_model {
        s.default_model = Some(m);
    }
    if let Some(p) = payload.default_permission_mode {
        s.default_permission_mode = Some(p);
    }
    s.onboarding_completed = Some(true);
    settings::put(claude_dir, &s)?;

    let mut c = config;
    if let Some(t) = payload.theme {
        c.theme = Some(t);
    }
    if let Some(d) = payload.claude_dir_override {
        c.claude_dir_override = Some(d);
    }
    Ok((c, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_settings_and_returns_config() {
        let td = tempfile::tempdir().unwrap();
        let cfg = AppConfig::default();
        let payload = SetupPayload {
            claude_dir_override: None,
            default_model: Some("sonnet".into()),
            default_permission_mode: Some("default".into()),
            theme: Some("dark".into()),
        };
        let (out_cfg, out_settings) = finalize(td.path(), cfg, payload).unwrap();
        assert_eq!(out_cfg.theme.as_deref(), Some("dark"));
        assert_eq!(out_settings.default_model.as_deref(), Some("sonnet"));
        assert_eq!(out_settings.onboarding_completed, Some(true));
        // Persisted on disk.
        let on_disk = settings::get(td.path()).unwrap();
        assert_eq!(on_disk.default_model.as_deref(), Some("sonnet"));
    }
}
