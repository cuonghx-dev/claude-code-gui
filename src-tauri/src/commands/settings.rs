use std::path::PathBuf;

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
pub async fn settings_put(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::settings::put(&claude_dir, &settings)
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
    let new_dir = match config.claude_dir_override.as_deref() {
        Some(p) if !p.is_empty() => Some(PathBuf::from(app_core::files::expand_tilde(p))),
        _ => None,
    };

    state::save_config(&claude_dir_now, &config)?;
    *state.config.write().await = config.clone();

    if let Some(dir) = new_dir {
        if dir != claude_dir_now {
            app_core::claude_dir::ensure(&dir)?;
            *state.claude_dir.write().await = dir.clone();
            let _ = app.emit(
                "app:claude_dir_changed",
                serde_json::json!({ "path": dir.to_string_lossy() }),
            );
        }
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
