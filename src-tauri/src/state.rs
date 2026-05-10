use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;

use app_core::claude_cli::ClaudeCliInfo;
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
}

const CONFIG_FILE: &str = ".app-config.json";

pub fn load_config(claude_dir: &Path) -> AppConfig {
    let path = claude_dir.join(CONFIG_FILE);
    if !path.is_file() {
        return AppConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(claude_dir: &Path, config: &AppConfig) -> Result<(), AppError> {
    let path = claude_dir.join(CONFIG_FILE);
    let serialized = serde_json::to_string_pretty(config)?;
    app_core::io::atomic_write(&path, serialized.as_bytes())
}
