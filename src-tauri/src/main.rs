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
            // A second launch (often a deep link) lands here: bring the
            // existing window forward, then let the frontend route the args.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
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

            // macOS registers the scheme from Info.plist at bundle time;
            // Linux and Windows need a runtime registration (also covers
            // dev builds, which have no installer).
            #[cfg(any(target_os = "linux", windows))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                if let Err(e) = app.deep_link().register_all() {
                    tracing::warn!(error = %e, "deep-link scheme registration failed");
                }
            }

            // Persisted AppConfig (theme, override, …) lives in the OS
            // app-config dir so the override can pick claude_dir. Best-effort.
            let config_path = state::config_path(&app.path().app_config_dir()?);
            let default_claude_dir = app_core::claude_dir::resolve(None).ok();
            let config = state::load_config(&config_path, default_claude_dir.as_deref());

            // Resolve claude_dir (env > override > ~/.claude) and ensure it exists.
            let claude_dir = state::claude_dir_for(&config)?;
            app_core::claude_dir::ensure(&claude_dir)?;

            // Best-effort probe; None just surfaces a setup banner in the UI.
            let claude_cli = app_core::claude_cli::probe();

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
            let claude_dir_watch = match watcher_handle.watch_claude_dir(&claude_dir) {
                Ok(id) => Some(id),
                Err(e) => {
                    tracing::warn!(error = %e, "watcher could not subscribe to claude_dir");
                    None
                }
            };

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

            // Derived indexes live in the OS cache dir, never in ~/.claude.
            let cache_dir = app
                .path()
                .app_cache_dir()
                .map(|d| d.join("index"))
                .unwrap_or_else(|_| std::env::temp_dir().join("claude-code-gui-index"));
            if let Err(e) = std::fs::create_dir_all(&cache_dir) {
                tracing::warn!(error = %e, dir = %cache_dir.display(), "cache dir unavailable");
            }

            app.manage(commands::updater::PendingUpdate::default());
            app.manage(AppState {
                claude_dir: Arc::new(RwLock::new(claude_dir)),
                claude_cli: Arc::new(RwLock::new(claude_cli)),
                config: Arc::new(RwLock::new(config)),
                watcher: Arc::new(watcher_handle),
                pty: pty_for_shutdown,
                cache_dir: Arc::new(cache_dir),
                transcript_index: Arc::new(app_core::transcript_scan::IndexCache::new()),
                config_path: Arc::new(config_path),
                claude_dir_watch: Arc::new(std::sync::Mutex::new(claude_dir_watch)),
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
            commands::claude_directory::claude_directory_tree,
            commands::claude_directory::claude_directory_children,
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
            commands::cli_history::cli_history_list,
            commands::cli_history::cli_history_get,
            commands::jobs::jobs_list,
            commands::jobs::jobs_get,
            commands::file_history::file_history_checkpoints,
            commands::file_history::file_history_diff,
            commands::file_history::file_history_blob,
            commands::file_history::file_history_restore,
            commands::settings_scope::settings_scopes,
            commands::settings_scope::settings_raw_get,
            commands::settings_scope::settings_raw_put,
            commands::settings_scope::settings_patch,
            commands::settings_scope::settings_effective,
            commands::permissions::permissions_get,
            commands::permissions::permissions_put,
            commands::permissions::permissions_effective,
            commands::permissions::permissions_validate,
            commands::hooks::hooks_get,
            commands::hooks::hooks_create,
            commands::hooks::hooks_update,
            commands::hooks::hooks_delete,
            commands::hooks::hooks_raw_get,
            commands::hooks::hooks_raw_put,
            commands::memory::memory_list,
            commands::memory::memory_agent_list,
            commands::memory::memory_get,
            commands::memory::memory_put,
            commands::memory::memory_delete,
            commands::memory::memory_preview,
            commands::statusline::statusline_get,
            commands::statusline::statusline_put,
            commands::statusline::statusline_delete,
            commands::statusline::statusline_preview,
            commands::keybindings::keybindings_get,
            commands::keybindings::keybindings_put,
            commands::keybindings::keybindings_create,
            commands::keybindings::keybindings_raw_get,
            commands::keybindings::keybindings_raw_put,
            commands::keybindings::keybindings_validate,
            commands::usage::usage_refresh,
            commands::usage::usage_rollup,
            commands::usage::usage_activity,
            commands::teams::teams_list,
            commands::teams::teams_get,
            commands::workflows::workflows_list,
            commands::workflows::workflows_get,
            commands::workflows::workflows_create,
            commands::workflows::workflows_update,
            commands::workflows::workflows_delete,
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
            commands::projects::projects_worktrees,
            commands::projects::projects_worktree_include,
            commands::projects::projects_settings_get,
            commands::projects::projects_claude_md_get,
            commands::projects::projects_claude_md_put,
            commands::sessions::sessions_list_for_project,
            commands::sessions::sessions_messages,
            commands::sessions::sessions_threads,
            commands::sessions::sessions_thread_messages,
            commands::sessions::sessions_rename,
            commands::sessions::sessions_delete,
            commands::settings::settings_get,
            commands::settings::config_get,
            commands::settings::config_set,
            commands::settings::setup_finalize,
            commands::files::directories_list,
            commands::files::files_read,
            commands::files::fs_home_dir,
            commands::files::reveal_in_finder,
            commands::updater::updater_check,
            commands::updater::updater_install,
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
