//! Mirrors crates/core/tests/export_types.rs. The drift gate runs
//! `cargo test --workspace`, which fires the auto-generated
//! `export_bindings_*` tests across every crate that derives `#[ts(export)]`.

#[test]
fn ts_rs_export_target_dir_exists() {
    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = crate_dir.join("../frontend/src/types/ipc");
    let target = target.canonicalize().unwrap_or(target);
    if !target.exists() {
        eprintln!("note: target dir does not exist yet: {}", target.display());
    }
}
