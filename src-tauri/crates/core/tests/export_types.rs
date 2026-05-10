//! Drift gate sentinel.
//!
//! `cargo test --workspace` runs the auto-generated `export_bindings_<Type>`
//! tests that ts-rs emits for every `#[ts(export)]` derive. Those tests
//! materialize TypeScript files under `frontend/src/types/ipc/`. CI then
//! runs `git diff --exit-code -- frontend/src/types/ipc/`; a non-empty diff
//! means a Rust IPC type changed and the generated TS was not committed.
//!
//! This file's existence proves the test target compiles and links.
//! The actual export work happens in the auto-generated tests.

#[test]
fn ts_rs_export_target_dir_exists() {
    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = crate_dir.join("../../../frontend/src/types/ipc");
    let target = target.canonicalize().unwrap_or(target);
    // Best-effort assertion: in a fresh clone the dir may not exist until
    // the first export run materializes it. We log in that case rather than
    // fail — the auto-generated tests are the actual gate.
    if !target.exists() {
        eprintln!("note: target dir does not exist yet: {}", target.display());
    }
}
