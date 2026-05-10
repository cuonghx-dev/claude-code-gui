//! Phase 0 stub. Phase 4 fills in the full target-resolution logic per SPEC §6
//! and ADR/§E of the implementation plan.
//!
//! Invariant (locked from SPEC §6 revisions): the terminal subsystem is a
//! strict Claude wrapper. `compose_command` MUST return `InvalidInput` if
//! none of `agent_slug` / `resume_session_id` / `command_template` is set.
//! The `$SHELL` fallback was removed deliberately — see ADR 0011 / SPEC §8.

use app_core::{AppError, ErrorCode};

/// Phase 0 placeholder. Full signature in Phase 4:
/// `pub async fn compose_command(opts: &TerminalOpts) -> Result<CommandBuilder, AppError>`.
pub fn compose_command() -> Result<(), AppError> {
    Err(AppError::new(
        ErrorCode::InvalidInput,
        "compose_command is a Phase 0 stub; Phase 4 wires the full impl.",
    ))
}
