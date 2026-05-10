use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use app_core::claude_cli::ClaudeCliInfo;

/// Held by Tauri's `State<AppState>`. Phase 0 has the minimum required to
/// boot. Phase 1+ adds `pty: Arc<PtyManager>`, `config: Arc<RwLock<AppConfig>>`,
/// etc., per SPEC §2.
#[allow(dead_code)] // Phase 0; consumers land in Phase 1+.
pub struct AppState {
    pub claude_dir: Arc<RwLock<PathBuf>>,
    pub claude_cli: Arc<RwLock<Option<ClaudeCliInfo>>>,
    pub watcher: Arc<watcher::WatcherHandle>,
}
