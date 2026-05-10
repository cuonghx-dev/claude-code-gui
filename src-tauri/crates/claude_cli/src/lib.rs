//! `claude -p` subprocess wrapper. Phase 0 skeleton — full implementation
//! in Phase 3 (`improve_instructions`) per SPEC §6.
//!
//! Phase 3 adds:
//! - `improve_instructions(app, request_id, input)` spawning
//!   `claude -p --output-format stream-json --input-format stream-json`
//! - tolerant stream-json deserializer that emits `claude:improve:{id}`
//!   events with `kind: 'delta' | 'done' | 'error'`
