use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;

use app_core::claude_cli::ClaudeCliInfo;
use app_core::transcript_scan::IndexCache;
use app_core::types::AppConfig;
use app_core::AppError;

/// Held by Tauri's `State<AppState>`. Phase 2 adds `config` (persisted as
/// JSON next to `~/.claude/.app-config.json`). PTY and watcher subscription
/// state arrive in Phase 4.
#[allow(dead_code)] // some fields land in later phases.
pub struct AppState {
    pub claude_dir: Arc<RwLock<PathBuf>>,
    pub claude_cli: Arc<RwLock<Option<ClaudeCliInfo>>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub watcher: Arc<watcher::WatcherHandle>,
    pub pty: Arc<pty::PtyManager>,
    /// App-owned scratch space for derived indexes. Deliberately *not* under
    /// `~/.claude`: the watcher subscribes there recursively, so a cache write
    /// would emit `fs:change` -> invalidate -> refetch -> rewrite, forever.
    pub cache_dir: Arc<PathBuf>,
    /// Transcript line indexes, reused across invocations.
    pub transcript_index: Arc<IndexCache>,
    /// Where `AppConfig` is persisted (OS app-config dir, outside `claude_dir`
    /// so the override that picks `claude_dir` can be read before resolving it).
    pub config_path: Arc<PathBuf>,
    /// Watcher subscription for the active `claude_dir`, swapped on override change.
    pub claude_dir_watch: Arc<std::sync::Mutex<Option<uuid::Uuid>>>,
}

const CONFIG_FILE: &str = "config.json";
/// Pre-0.2 location, read once as a fallback so existing overrides survive.
const LEGACY_CONFIG_FILE: &str = ".app-config.json";

pub fn config_path(app_config_dir: &Path) -> PathBuf {
    app_config_dir.join(CONFIG_FILE)
}

/// Load `AppConfig` from `path`, falling back to the legacy file under the
/// default `~/.claude`. Best-effort: unreadable or malformed files yield defaults.
pub fn load_config(path: &Path, legacy_claude_dir: Option<&Path>) -> AppConfig {
    let read = |p: &Path| -> Option<AppConfig> {
        let raw = std::fs::read_to_string(p).ok()?;
        serde_json::from_str(&raw).ok()
    };
    if let Some(cfg) = read(path) {
        return cfg;
    }
    legacy_claude_dir
        .and_then(|d| read(&d.join(LEGACY_CONFIG_FILE)))
        .unwrap_or_default()
}

pub fn save_config(path: &Path, config: &AppConfig) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(config)?;
    app_core::io::atomic_write(path, serialized.as_bytes())
}

/// The `claude_dir` a config selects: env > non-empty override > `~/.claude`.
pub fn claude_dir_for(config: &AppConfig) -> Result<PathBuf, AppError> {
    let override_dir = config
        .claude_dir_override
        .as_deref()
        .filter(|p| !p.trim().is_empty())
        .map(app_core::files::expand_tilde);
    app_core::claude_dir::resolve(override_dir.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trips_and_legacy_fallback() {
        let td = tempfile::tempdir().unwrap();
        let legacy = td.path().join("claude");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(
            legacy.join(LEGACY_CONFIG_FILE),
            r#"{"claudeDirOverride":"/tmp/x"}"#,
        )
        .unwrap();
        let path = config_path(&td.path().join("cfg"));

        let cfg = load_config(&path, Some(&legacy));
        assert_eq!(cfg.claude_dir_override.as_deref(), Some("/tmp/x"));

        let mut next = cfg.clone();
        next.claude_dir_override = Some("/tmp/y".into());
        save_config(&path, &next).unwrap();
        let back = load_config(&path, Some(&legacy));
        assert_eq!(back.claude_dir_override.as_deref(), Some("/tmp/y"));
    }
}
