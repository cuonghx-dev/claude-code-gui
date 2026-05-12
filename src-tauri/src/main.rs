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
            // macOS GUI apps inherit only the bare launchd PATH; pull in the
            // login-shell PATH so `claude`, `git`, `node`, etc. resolve.
            // Must run before any binary lookup.
            app_core::claude_cli::inherit_login_path();

            // Resolve claude_dir (env > default ~/.claude) and ensure it exists.
            let claude_dir = app_core::claude_dir::resolve(None)?;
            app_core::claude_dir::ensure(&claude_dir)?;

            // Best-effort probe; None just surfaces a setup banner in the UI.
            let claude_cli = app_core::claude_cli::probe();

            // Persisted AppConfig (theme, override, …). Best-effort load.
            let config = state::load_config(&claude_dir);

            // Real watcher: emit callback bridges into Tauri events.
            let app_for_emit = app.handle().clone();
            let emit: watcher::Emit = std::sync::Arc::new(move |name, payload| {
                if let Err(e) = app_for_emit.emit(name, payload) {
                    tracing::warn!(error = %e, event = %name, "failed to emit watcher event");
                }
            });
            let watcher_handle = watcher::start_global(emit)?;
            // Subscribe to ~/.claude/** unconditionally; project subs are
            // added on demand via `watch_project_dir`.
            if let Err(e) = watcher_handle.watch_claude_dir(&claude_dir) {
                tracing::warn!(error = %e, "watcher could not subscribe to claude_dir");
            }

            // PTY manager: shares the emit-callback pattern. cli-history
            // dir lives under claude_dir.
            let app_for_pty_emit = app.handle().clone();
            let pty_emit: pty::Emit = std::sync::Arc::new(move |name, payload| {
                if let Err(e) = app_for_pty_emit.emit(name, payload) {
                    tracing::warn!(error = %e, event = %name, "pty emit failed");
                }
            });
            let pty_manager = pty::PtyManager::new(pty_emit, &claude_dir);

            // Graceful shutdown: kill all PTYs when the main window
            // requests close. The webview teardown otherwise races with
            // PTY reader threads.
            let pty_for_shutdown = std::sync::Arc::new(pty_manager);
            let pty_for_event = std::sync::Arc::clone(&pty_for_shutdown);
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        let pty = std::sync::Arc::clone(&pty_for_event);
                        // best-effort: spawn on the runtime if possible.
                        tauri::async_runtime::spawn(async move {
                            pty.shutdown_all().await;
                        });
                    }
                });
            }

            app.manage(AppState {
                claude_dir: Arc::new(RwLock::new(claude_dir)),
                claude_cli: Arc::new(RwLock::new(claude_cli)),
                config: Arc::new(RwLock::new(config)),
                watcher: Arc::new(watcher_handle),
                pty: pty_for_shutdown,
            });

            tracing::info!("claude-code-gui ready");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agents::agents_list,
            commands::agents::agents_get,
            commands::agents::agents_skill_counts,
            commands::agents::agents_create,
            commands::agents::agents_update,
            commands::agents::agents_update_raw,
            commands::agents::agents_delete,
            commands::agents::agents_export,
            commands::agents::agents_import,
            commands::agents::agents_improve_instructions,
            commands::cmds::commands_list,
            commands::cmds::commands_get,
            commands::cmds::commands_create,
            commands::cmds::commands_update,
            commands::cmds::commands_update_raw,
            commands::cmds::commands_import_raw,
            commands::cmds::commands_export,
            commands::cmds::commands_delete,
            commands::skills::skills_list,
            commands::skills::skills_get,
            commands::skills::skills_create,
            commands::skills::skills_create_raw,
            commands::skills::skills_update,
            commands::skills::skills_update_raw,
            commands::skills::skills_read_raw,
            commands::skills::skills_delete,
            commands::skills::skills_export,
            commands::skills::skills_import,
            commands::plans::plans_list,
            commands::plans::plans_get,
            commands::plans::plans_create,
            commands::plans::plans_update,
            commands::plans::plans_delete,
            commands::hooks::hooks_list,
            commands::output_styles::output_styles_list,
            commands::output_styles::output_styles_get,
            commands::output_styles::output_styles_create,
            commands::output_styles::output_styles_delete,
            commands::mcp::mcp_list,
            commands::mcp::mcp_get,
            commands::mcp::mcp_create,
            commands::mcp::mcp_delete,
            commands::mcp::mcp_import,
            commands::mcp::mcp_capabilities,
            commands::relationships::relationships_graph,
            commands::plugins::plugins_list,
            commands::plugins::plugins_get,
            commands::plugins::plugins_delete,
            commands::plugins::plugins_set_enabled,
            commands::plugins::plugins_update_skills,
            commands::marketplace::marketplace_available,
            commands::marketplace::marketplace_sources_list,
            commands::marketplace::marketplace_sources_add,
            commands::marketplace::marketplace_sources_remove,
            commands::marketplace::marketplace_sources_update,
            commands::marketplace::marketplace_install,
            commands::marketplace::marketplace_uninstall,
            commands::projects::projects_list,
            commands::projects::projects_get,
            commands::projects::projects_resolve,
            commands::projects::projects_files,
            commands::projects::projects_create,
            commands::projects::projects_rename,
            commands::projects::projects_delete,
            commands::projects::projects_git_status,
            commands::projects::projects_settings_get,
            commands::projects::projects_settings_put,
            commands::projects::projects_claude_md_get,
            commands::projects::projects_claude_md_put,
            commands::sessions::sessions_list_for_project,
            commands::sessions::sessions_messages,
            commands::settings::settings_get,
            commands::settings::settings_put,
            commands::settings::config_get,
            commands::settings::config_set,
            commands::settings::setup_finalize,
            commands::files::directories_list,
            commands::files::files_read,
            commands::files::fs_home_dir,
            commands::files::watch_project_dir,
            commands::files::unwatch_path,
            commands::terminal::terminal_session_create,
            commands::terminal::terminal_session_input,
            commands::terminal::terminal_session_resize,
            commands::terminal::terminal_session_kill,
            commands::terminal::terminal_sessions_list,
            commands::terminal::terminal_session_get,
            commands::terminal::commands_execute,
            commands::debug::debug_claude_cli,
            commands::debug::app_version,
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
