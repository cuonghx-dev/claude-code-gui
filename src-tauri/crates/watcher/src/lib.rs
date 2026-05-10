//! Filesystem watcher. Phase 0 skeleton — full implementation in Phase 3.
//!
//! Phase 3 adds:
//! - notify::Watcher → notify-debouncer-mini → ignore::Gitignore matcher
//!   → per-subscription rate limiter (sliding 1s window, cap 500 ev/s)
//!   → `app.emit("fs:change", { path, kind })`
//! - `fs:flood` event when rate limit trips; subscription paused 5s
//! - `watch_claude_dir` (no ignore) vs `watch_project` (gitignore + denylist)

use app_core::AppError;

pub struct WatcherHandle {
    _private: (),
}

pub fn start_global() -> Result<WatcherHandle, AppError> {
    // Phase 3 implementation pending.
    Ok(WatcherHandle { _private: () })
}
