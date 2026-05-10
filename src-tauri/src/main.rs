// Hide the console window on Windows release builds.
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod commands;
mod error;
mod events;
mod state;

use std::sync::Arc;

use tauri::{Emitter, Manager};
use tokio::sync::RwLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::state::AppState;

fn main() {
    init_tracing();
    install_panic_hook();

    if let Err(e) = run() {
        tracing::error!(error = ?e, "fatal: tauri builder failed");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    tauri::Builder::default()
        // Single-instance must be registered first per plugin docs.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // Phase 6 wires deep-link routing on subsequent launches.
            let _ = app.emit("app:single_instance", serde_json::json!({ "args": args }));
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Resolve claude_dir (env > default ~/.claude) and ensure it exists.
            let claude_dir = app_core::claude_dir::resolve(None)?;
            app_core::claude_dir::ensure(&claude_dir)?;

            // Best-effort probe; None just surfaces a setup banner in the UI.
            let claude_cli = app_core::claude_cli::probe();

            // Phase 3 wires the real watcher; Phase 0 is a no-op stub.
            let watcher_handle = watcher::start_global()?;

            app.manage(AppState {
                claude_dir: Arc::new(RwLock::new(claude_dir)),
                claude_cli: Arc::new(RwLock::new(claude_cli)),
                watcher: Arc::new(watcher_handle),
            });

            tracing::info!("claude-code-gui ready");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Phase 1+ wires command bindings here.
        ])
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(debug_assertions)]
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    #[cfg(not(debug_assertions))]
    let fmt_layer = tracing_subscriber::fmt::layer().json();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(panic = %info, "panic in tauri process");
        default(info);
    }));
}
