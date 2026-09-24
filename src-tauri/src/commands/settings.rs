use app_core::types::{AppConfig, SetupPayload, Settings};
use app_core::AppError;
use tauri::{AppHandle, Emitter, State};

use crate::state::{self, AppState};

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<Settings, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::settings::get(&claude_dir)
}


#[tauri::command]
pub async fn config_get(state: State<'_, AppState>) -> Result<AppConfig, AppError> {
    Ok(state.config.read().await.clone())
}

#[tauri::command]
pub async fn config_set(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), AppError> {
    let claude_dir_now = state.claude_dir.read().await.clone();
    let new_dir = state::claude_dir_for(&config)?;

    state::save_config(&state.config_path, &config)?;
    *state.config.write().await = config.clone();

    if new_dir != claude_dir_now {
        app_core::claude_dir::ensure(&new_dir)?;
        *state.claude_dir.write().await = new_dir.clone();

        // Move the recursive watch to the new root so fs:change keeps flowing.
        let new_watch = match state.watcher.watch_claude_dir(&new_dir) {
            Ok(id) => Some(id),
            Err(e) => {
                tracing::warn!(error = %e, "watcher could not subscribe to new claude_dir");
                None
            }
        };
        let old_watch = std::mem::replace(
            &mut *state.claude_dir_watch.lock().expect("watch lock poisoned"),
            new_watch,
        );
        if let Some(id) = old_watch {
            let _ = state.watcher.unwatch(id);
        }

        let _ = app.emit(
            "app:claude_dir_changed",
            serde_json::json!({ "path": new_dir.to_string_lossy() }),
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn setup_finalize(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: SetupPayload,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let cfg_now = state.config.read().await.clone();
    let (new_cfg, _settings) = app_core::setup::finalize(&claude_dir, cfg_now, payload)?;
    config_set(app, state, new_cfg).await
}
