//! Filesystem watcher.
//!
//! Architecture:
//! ```text
//!   notify (raw events) → notify-debouncer-mini (200ms)
//!     → matcher (gitignore + global denylist) per-subscription
//!     → rate-limiter (1s sliding window, 500 ev cap)
//!     → emit("fs:change", { path, kind })
//! ```
//! When the rate-limiter trips it emits `fs:flood` and pauses the offending
//! subscription for 5s. `~/.claude/**` subscriptions skip the matcher
//! (every file there is editable user content); project subscriptions layer
//! the project's `.gitignore` chain on top of a hard-coded global denylist.
//!
//! The crate is intentionally tauri-free: a callback is supplied by the
//! caller (the Tauri app wires `app.emit`). This keeps `watcher` testable
//! with `cargo test` and preserves the `core has no tauri deps` rule
//! (this crate has neither, which is even stricter).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, DebouncedEventKind, Debouncer};
use uuid::Uuid;

use app_core::{AppError, ErrorCode};

/// Callback invoked for every successful event emission. Wired in main.rs
/// to `app.emit(name, payload)`.
pub type Emit = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync + 'static>;

/// Rate limit: 500 events per 1s window, paused 5s when tripped.
const RATE_WINDOW: Duration = Duration::from_secs(1);
const RATE_CAP: usize = 500;
const RATE_PAUSE: Duration = Duration::from_secs(5);

/// Hard-coded ignore globs that always apply to project subscriptions
/// (independent of `.gitignore`). Matched against any path component.
const GLOBAL_DENY_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".turbo",
    ".cache",
    ".venv",
    "venv",
    "__pycache__",
];
const GLOBAL_DENY_FILES: &[&str] = &[".DS_Store"];

pub struct WatcherHandle {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    debouncer: Debouncer<notify::RecommendedWatcher>,
    subs: HashMap<Uuid, Subscription>,
    /// Reverse: path → subscription, so a duplicate `watch_project` call is a no-op.
    paths: HashMap<PathBuf, Uuid>,
    emit: Emit,
}

struct Subscription {
    root: PathBuf,
    is_claude_dir: bool,
    matcher: Option<Gitignore>,
    /// Sliding-window event timestamps in the last `RATE_WINDOW`.
    bucket: Vec<Instant>,
    /// `Some(until)` while paused.
    paused_until: Option<Instant>,
}

impl WatcherHandle {
    pub fn watch_claude_dir(&self, path: &Path) -> Result<Uuid, AppError> {
        self.add(path, true)
    }

    pub fn watch_project(&self, path: &Path) -> Result<Uuid, AppError> {
        self.add(path, false)
    }

    pub fn unwatch(&self, id: Uuid) -> Result<(), AppError> {
        let mut inner = self.lock()?;
        let Some(sub) = inner.subs.remove(&id) else { return Ok(()) };
        inner.paths.remove(&sub.root);
        // Best-effort unwatch; notify returns Err if path was never added.
        let _ = inner.debouncer.watcher().unwatch(&sub.root);
        Ok(())
    }

    fn add(&self, path: &Path, is_claude_dir: bool) -> Result<Uuid, AppError> {
        let canonical = path
            .canonicalize()
            .map_err(|e| AppError::new(ErrorCode::IoError, e.to_string()))?;
        let mut inner = self.lock()?;
        if let Some(existing) = inner.paths.get(&canonical).copied() {
            return Ok(existing);
        }
        let matcher = if is_claude_dir {
            None
        } else {
            Some(build_matcher(&canonical))
        };
        inner
            .debouncer
            .watcher()
            .watch(&canonical, RecursiveMode::Recursive)
            .map_err(|e| AppError::new(ErrorCode::IoError, e.to_string()))?;
        let id = Uuid::new_v4();
        let sub = Subscription {
            root: canonical.clone(),
            is_claude_dir,
            matcher,
            bucket: Vec::new(),
            paused_until: None,
        };
        inner.subs.insert(id, sub);
        inner.paths.insert(canonical, id);
        Ok(id)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>, AppError> {
        self.inner
            .lock()
            .map_err(|_| AppError::internal("watcher mutex poisoned"))
    }
}

/// Initialize the global watcher. The supplied `emit` callback is invoked
/// for every filtered event with `name in ["fs:change", "fs:flood"]`.
pub fn start_global(emit: Emit) -> Result<WatcherHandle, AppError> {
    let inner = Arc::new(Mutex::new(Inner::placeholder(emit.clone())));
    let inner_for_cb = Arc::clone(&inner);

    let debouncer = new_debouncer(Duration::from_millis(200), move |res| {
        let events: Vec<DebouncedEvent> = match res {
            Ok(events) => events,
            Err(e) => {
                tracing::warn!(error = %e, "notify error");
                return;
            }
        };
        // Acquire and process under lock. Keep the section short — debouncer
        // dispatch runs on its own thread.
        let Ok(mut guard) = inner_for_cb.lock() else { return };
        process_batch(&mut guard, events);
    })
    .map_err(|e| AppError::new(ErrorCode::IoError, e.to_string()))?;

    {
        let mut g = inner
            .lock()
            .map_err(|_| AppError::internal("watcher mutex poisoned"))?;
        g.debouncer = debouncer;
    }
    Ok(WatcherHandle { inner })
}

impl Inner {
    /// Construct an empty `Inner` with a placeholder debouncer that's
    /// immediately replaced by the real one in `start_global`. We need a
    /// concrete value first because the debouncer callback closes over the
    /// Arc<Mutex<Inner>>.
    fn placeholder(emit: Emit) -> Self {
        // Build a no-op debouncer that gets replaced before the handle is
        // exposed. Using a temp tempdir keeps notify happy on platforms that
        // require an actual path during construction.
        let tmp = std::env::temp_dir();
        let debouncer = new_debouncer(Duration::from_millis(200), |_res| ()).expect(
            "init placeholder debouncer; this should never fail on a healthy system",
        );
        // tmp not actually watched, but reference the var to avoid `unused`.
        let _ = tmp;
        Self {
            debouncer,
            subs: HashMap::new(),
            paths: HashMap::new(),
            emit,
        }
    }
}

fn process_batch(inner: &mut Inner, events: Vec<DebouncedEvent>) {
    let now = Instant::now();
    // Group post-filter events per subscription so we can apply rate
    // limiting once per group.
    let mut grouped: HashMap<Uuid, Vec<(PathBuf, &'static str)>> = HashMap::new();

    for ev in events {
        let kind = match ev.kind {
            DebouncedEventKind::Any => "modify",
            _ => continue,
        };
        let Some((id, sub)) = find_subscription(&inner.subs, &ev.path) else { continue };
        if sub.paused_until.map(|u| now < u).unwrap_or(false) {
            continue;
        }
        if !sub.is_claude_dir && is_denied(&ev.path) {
            continue;
        }
        if let Some(m) = &sub.matcher {
            // `matched_path_or_any_parents` so ignored *parent* dirs eat their children.
            if m.matched_path_or_any_parents(&ev.path, false).is_ignore() {
                continue;
            }
        }
        grouped
            .entry(id)
            .or_default()
            .push((ev.path.clone(), kind));
    }

    let emit = Arc::clone(&inner.emit);
    for (id, events) in grouped {
        let Some(sub) = inner.subs.get_mut(&id) else { continue };
        // Maintain sliding window.
        sub.bucket.retain(|t| now.duration_since(*t) <= RATE_WINDOW);
        let added = events.len();
        if sub.bucket.len() + added > RATE_CAP {
            // Trip flood: emit one fs:flood and pause.
            let payload = serde_json::json!({
                "subscriptionId": id.to_string(),
                "root": sub.root.to_string_lossy(),
                "eventsPerSec": (sub.bucket.len() + added) as u64,
            });
            sub.paused_until = Some(now + RATE_PAUSE);
            sub.bucket.clear();
            emit("fs:flood", payload);
            tracing::warn!(root = %sub.root.display(), "fs watcher flooded — paused 5s");
            continue;
        }
        sub.bucket.extend(std::iter::repeat(now).take(added));
        for (path, kind) in events {
            emit(
                "fs:change",
                serde_json::json!({
                    "path": path.to_string_lossy(),
                    "kind": kind,
                }),
            );
        }
    }
}

fn find_subscription<'a>(
    subs: &'a HashMap<Uuid, Subscription>,
    path: &Path,
) -> Option<(Uuid, &'a Subscription)> {
    // Pick the most-specific (longest) root that the path is under.
    subs.iter()
        .filter(|(_id, sub)| path.starts_with(&sub.root))
        .max_by_key(|(_id, sub)| sub.root.as_os_str().len())
        .map(|(id, sub)| (*id, sub))
}

fn build_matcher(root: &Path) -> Gitignore {
    let mut b = GitignoreBuilder::new(root);
    // Walk up the parent chain, layering each `.gitignore` we hit.
    let mut cursor = Some(root.to_path_buf());
    while let Some(dir) = cursor {
        let gi = dir.join(".gitignore");
        if gi.is_file() {
            let _ = b.add(&gi);
        }
        cursor = dir.parent().map(|p| p.to_path_buf());
    }
    b.build().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "gitignore build failed; falling back to empty");
        GitignoreBuilder::new(root).build().expect("empty gitignore should always build")
    })
}

fn is_denied(path: &Path) -> bool {
    let name = match path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n,
        None => return false,
    };
    if GLOBAL_DENY_FILES.contains(&name) {
        return true;
    }
    for comp in path.components() {
        if let std::path::Component::Normal(seg) = comp {
            if let Some(s) = seg.to_str() {
                if GLOBAL_DENY_DIRS.contains(&s) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denylist_blocks_node_modules() {
        assert!(is_denied(Path::new("/proj/node_modules/foo/index.js")));
        assert!(is_denied(Path::new("/proj/.DS_Store")));
        assert!(!is_denied(Path::new("/proj/src/main.rs")));
    }

    #[test]
    fn start_global_smoke() {
        // Just ensure it constructs without panic.
        let emit: Emit = Arc::new(|_n, _p| {});
        let _h = start_global(emit).unwrap();
    }
}
