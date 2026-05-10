//! PTY session manager.
//!
//! Each session owns:
//! - a `portable_pty::MasterPty` and child handle
//! - a Tokio writer half (`tokio::sync::Mutex<Box<dyn Write + Send>>`)
//! - a reader thread (`spawn_blocking`) that pumps stdout chunks back as
//!   `pty:output:{id}` events while threading the same chunk through the
//!   context monitor for `context:tokens:{id}` / `context:tool:{id}`
//! - an exit watcher (`spawn_blocking`) that fires `pty:exit:{id}` and
//!   snapshots the ring buffer to `~/.claude/cli-history/<id>.json`
//! - an idle killer (`tokio::spawn`) that kills the child after 30 min
//!   of no input/output activity
//!
//! The crate is tauri-free: callers supply an `Emit` callback. This keeps
//! the per-session machinery testable without a Tauri runtime.

pub mod compose;
pub mod monitor;

use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use portable_pty::{native_pty_system, ChildKiller, MasterPty, PtySize};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use app_core::types::{TerminalOpts, TerminalSession};
use app_core::{AppError, ErrorCode};

pub use compose::compose;

const RING_CAPACITY_LINES: usize = 10_000;
const IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const SOFT_CAP_SESSIONS: usize = 16;
const HISTORY_SUBDIR: &str = "cli-history";

/// Callback invoked for every event emission. Wired in main.rs to `app.emit`.
pub type Emit = Arc<dyn Fn(&str, serde_json::Value) + Send + Sync + 'static>;

pub struct PtyManager {
    sessions: Arc<RwLock<HashMap<Uuid, Arc<Session>>>>,
    emit: Emit,
    history_dir: PathBuf,
}

struct Session {
    id: Uuid,
    writer: Mutex<Box<dyn Write + Send>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
    meta: Mutex<TerminalSession>,
    /// Last 10K output lines as a sliding window. Used by `cli-history` snapshots.
    ring: Mutex<VecDeque<String>>,
    last_activity: Mutex<Instant>,
    alive: Mutex<bool>,
}

impl PtyManager {
    pub fn new(emit: Emit, claude_dir: &Path) -> Self {
        let history_dir = claude_dir.join(HISTORY_SUBDIR);
        let _ = std::fs::create_dir_all(&history_dir);
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            emit,
            history_dir,
        }
    }

    pub async fn create(
        &self,
        claude_dir: &Path,
        opts: TerminalOpts,
    ) -> Result<Uuid, AppError> {
        {
            let map = self.sessions.read().await;
            if map.len() >= SOFT_CAP_SESSIONS {
                return Err(AppError::new(
                    ErrorCode::ResourceLimit,
                    format!(
                        "terminal session cap reached ({} active)",
                        map.len()
                    ),
                ));
            }
        }
        let cmd = compose(claude_dir, &opts)?;
        let id = Uuid::new_v4();
        let now_iso = Utc::now().to_rfc3339();
        let meta = TerminalSession {
            id: id.to_string(),
            agent_slug: opts.agent_slug.clone(),
            working_dir: opts.working_dir.clone(),
            cols: opts.cols,
            rows: opts.rows,
            model: opts.model.clone(),
            started_at: now_iso.clone(),
            last_activity: now_iso,
            alive: true,
        };

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: opts.rows.max(1),
                cols: opts.cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::new(ErrorCode::Internal, format!("openpty: {e}")))?;

        let mut child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AppError::new(ErrorCode::Internal, format!("spawn: {e}")))?;
        let killer = child.clone_killer();
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| AppError::new(ErrorCode::Internal, format!("clone_reader: {e}")))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| AppError::new(ErrorCode::Internal, format!("take_writer: {e}")))?;

        // Slave fd is no longer needed in the parent — drop it so the
        // child becomes the sole holder of the slave end. Without this the
        // PTY can stay open after the child exits.
        drop(pair.slave);

        let session = Arc::new(Session {
            id,
            writer: Mutex::new(writer),
            master: Mutex::new(pair.master),
            killer: Mutex::new(killer),
            meta: Mutex::new(meta),
            ring: Mutex::new(VecDeque::with_capacity(RING_CAPACITY_LINES)),
            last_activity: Mutex::new(Instant::now()),
            alive: Mutex::new(true),
        });

        {
            let mut map = self.sessions.write().await;
            map.insert(id, Arc::clone(&session));
        }

        spawn_reader(Arc::clone(&session), self.emit.clone(), reader);
        spawn_exit_watcher(
            Arc::clone(&session),
            self.emit.clone(),
            child,
            self.history_dir.clone(),
            self.sessions.clone(),
        );
        spawn_idle_killer(Arc::clone(&session));

        Ok(id)
    }

    pub async fn input(&self, id: Uuid, data: &[u8]) -> Result<(), AppError> {
        let session = self.get_session(id).await?;
        {
            let mut last = session.last_activity.lock().await;
            *last = Instant::now();
        }
        let mut writer = session.writer.lock().await;
        writer
            .write_all(data)
            .map_err(|e| AppError::new(ErrorCode::IoError, format!("pty input: {e}")))?;
        Ok(())
    }

    pub async fn resize(&self, id: Uuid, cols: u16, rows: u16) -> Result<(), AppError> {
        let session = self.get_session(id).await?;
        let master = session.master.lock().await;
        master
            .resize(PtySize {
                rows: rows.max(1),
                cols: cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AppError::new(ErrorCode::IoError, format!("resize: {e}")))?;
        let mut meta = session.meta.lock().await;
        meta.cols = cols;
        meta.rows = rows;
        Ok(())
    }

    pub async fn kill(&self, id: Uuid) -> Result<(), AppError> {
        let session = self.get_session(id).await?;
        let mut killer = session.killer.lock().await;
        killer
            .kill()
            .map_err(|e| AppError::new(ErrorCode::IoError, format!("kill: {e}")))?;
        Ok(())
    }

    pub async fn list(&self) -> Vec<TerminalSession> {
        let map = self.sessions.read().await;
        let mut out = Vec::with_capacity(map.len());
        for s in map.values() {
            out.push(s.meta.lock().await.clone());
        }
        out
    }

    pub async fn get(&self, id: Uuid) -> Result<TerminalSession, AppError> {
        let session = self.get_session(id).await?;
        let meta = session.meta.lock().await.clone();
        Ok(meta)
    }

    pub async fn shutdown_all(&self) {
        let map = self.sessions.read().await;
        for s in map.values() {
            let mut killer = s.killer.lock().await;
            let _ = killer.kill();
        }
    }

    async fn get_session(&self, id: Uuid) -> Result<Arc<Session>, AppError> {
        let map = self.sessions.read().await;
        map.get(&id)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("terminal session '{id}' not found")))
    }
}

/// Reader: blocks on `reader.read`, forwards bytes verbatim to the
/// frontend, and threads the same chunk through the context monitor.
fn spawn_reader(
    session: Arc<Session>,
    emit: Emit,
    mut reader: Box<dyn Read + Send>,
) {
    let id = session.id;
    let event_output = format!("pty:output:{id}");
    let event_tokens = format!("context:tokens:{id}");
    let event_tool = format!("context:tool:{id}");

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut state = monitor::MonitorState::default();
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]).into_owned();
                    // Update last activity.
                    if let Ok(mut g) = session.last_activity.try_lock() {
                        *g = Instant::now();
                    }
                    // Append to ring (line-by-line; partial lines flushed on next chunk).
                    if let Ok(mut g) = session.ring.try_lock() {
                        for line in chunk.split_inclusive('\n') {
                            if g.len() >= RING_CAPACITY_LINES {
                                g.pop_front();
                            }
                            g.push_back(line.to_string());
                        }
                    }
                    // Bump meta.last_activity so list/get reflects activity.
                    if let Ok(mut m) = session.meta.try_lock() {
                        m.last_activity = Utc::now().to_rfc3339();
                    }
                    emit(
                        &event_output,
                        serde_json::json!({ "data": chunk }),
                    );
                    let events = monitor::parse_chunk(&mut state, &chunk);
                    for ev in events {
                        match ev {
                            monitor::MonitorEvent::Tokens {
                                usage,
                                model,
                                cost,
                            } => emit(
                                &event_tokens,
                                serde_json::json!({
                                    "input": usage.input,
                                    "output": usage.output,
                                    "cached": usage.cache_read,
                                    "cacheWrite": usage.cache_write,
                                    "cost": cost,
                                    "model": model,
                                }),
                            ),
                            monitor::MonitorEvent::Tool(tc) => {
                                emit(&event_tool, serde_json::to_value(tc).unwrap_or_default())
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, session = %id, "pty reader error");
                    break;
                }
            }
        }
    });
}

fn spawn_exit_watcher(
    session: Arc<Session>,
    emit: Emit,
    mut child: Box<dyn portable_pty::Child + Send + Sync>,
    history_dir: PathBuf,
    sessions: Arc<RwLock<HashMap<Uuid, Arc<Session>>>>,
) {
    let id = session.id;
    std::thread::spawn(move || {
        let status = match child.wait() {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "child.wait failed");
                emit(
                    &format!("pty:exit:{id}"),
                    serde_json::json!({ "exitCode": -1 }),
                );
                return;
            }
        };
        let exit_code = status.exit_code() as i32;
        // Mark dead before emitting so the UI sees a consistent state.
        {
            // Block on the runtime's mutex via try_lock; meta is short-lived.
            let _ = futures_lite_block_on(async {
                {
                    let mut alive = session.alive.lock().await;
                    *alive = false;
                    let mut meta = session.meta.lock().await;
                    meta.alive = false;
                }
                {
                    let mut map = sessions.write().await;
                    map.remove(&id);
                }
                let ring = session.ring.lock().await;
                let snapshot = serde_json::json!({
                    "id": id.to_string(),
                    "exitCode": exit_code,
                    "endedAt": Utc::now().to_rfc3339(),
                    "lastLines": ring.iter().cloned().collect::<Vec<_>>(),
                    "meta": session.meta.lock().await.clone(),
                });
                let path = history_dir.join(format!("{id}.json"));
                if let Err(e) = std::fs::write(
                    &path,
                    serde_json::to_vec_pretty(&snapshot).unwrap_or_default(),
                ) {
                    tracing::warn!(error = %e, ?path, "history snapshot write failed");
                }
            });
        }
        emit(
            &format!("pty:exit:{id}"),
            serde_json::json!({ "exitCode": exit_code }),
        );
    });
}

/// Idle killer: every 60s, check if the session has gone quiet for
/// `IDLE_TIMEOUT` and SIGKILL if so. Spawned on the tokio runtime.
fn spawn_idle_killer(session: Arc<Session>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            let alive = *session.alive.lock().await;
            if !alive {
                break;
            }
            let last = *session.last_activity.lock().await;
            if last.elapsed() > IDLE_TIMEOUT {
                tracing::info!(session = %session.id, "idle timeout — killing");
                let mut killer = session.killer.lock().await;
                let _ = killer.kill();
                break;
            }
        }
    });
}

/// Tiny block-on helper for the synchronous exit-watcher thread. We use
/// `tokio::runtime::Handle::current().block_on` if a runtime is attached;
/// otherwise we synthesize one. The Tauri runtime is always present in
/// production; the test path may not have one.
fn futures_lite_block_on<F: std::future::Future>(fut: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(_) => {
            // We're already on the multi-thread runtime — but `block_on`
            // there panics. Spawn a single-threaded runtime to drive the
            // future to completion. Cost is one-shot: only on session exit.
            tokio::task::block_in_place(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("build current_thread runtime")
                    .block_on(fut)
            })
        }
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build current_thread runtime")
            .block_on(fut),
    }
}
