//! PTY session manager. Phase 0 skeleton — full implementation in Phase 4.
//!
//! Phase 4 adds:
//! - `PtyManager` with per-session state and ring buffer
//! - reader / exit_watcher / idle_killer Tokio tasks
//! - context_monitor with ANSI-stripped regex parse_chunk
//! - history snapshot writer for ~/.claude/cli-history/<id>.json

pub mod compose;

pub use compose::compose_command;
