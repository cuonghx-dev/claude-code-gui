//! Event name constants. Phase 3+ populates these as event-emitting commands
//! land. Frontend imports (via ts-rs is not used for event names — they're
//! just strings) need to mirror these in `frontend/src/lib/events.ts`.

#![allow(dead_code)]

pub const FS_CHANGE: &str = "fs:change";
pub const FS_FLOOD: &str = "fs:flood";

pub const APP_CLAUDE_DIR_CHANGED: &str = "app:claude_dir_changed";
pub const APP_SINGLE_INSTANCE: &str = "app:single_instance";

// Per-session topics use format strings: `pty:output:{session_id}`, etc.
pub const PTY_OUTPUT_PREFIX: &str = "pty:output:";
pub const PTY_EXIT_PREFIX: &str = "pty:exit:";
pub const CONTEXT_TOKENS_PREFIX: &str = "context:tokens:";
pub const CONTEXT_TOOL_PREFIX: &str = "context:tool:";
pub const CLAUDE_IMPROVE_PREFIX: &str = "claude:improve:";
pub const MARKETPLACE_INSTALL_PREFIX: &str = "marketplace:install:";
