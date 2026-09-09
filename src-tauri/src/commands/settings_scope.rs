use std::path::PathBuf;

use app_core::types::{EffectiveEntry, RawDoc, ScopeInfo, SettingsScope};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

fn wd(working_dir: Option<String>) -> Option<PathBuf> {
    working_dir.map(PathBuf::from)
}

#[tauri::command]
pub async fn settings_scopes(
    state: State<'_, AppState>,
    working_dir: Option<String>,
) -> Result<Vec<ScopeInfo>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    Ok(app_core::settings_scope::scopes(
        &claude_dir,
        wd(working_dir).as_deref(),
    ))
}

#[tauri::command]
pub async fn settings_raw_get(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
) -> Result<RawDoc, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::settings_scope::raw_get(&claude_dir, scope, wd(working_dir).as_deref())
}

#[tauri::command]
pub async fn settings_raw_put(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
    content: String,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::settings_scope::raw_put(
        &claude_dir,
        scope,
        wd(working_dir).as_deref(),
        &content,
        expected_mtime_ms,
    )
}

/// RFC 7386 merge patch. Callers send only the keys they own, so a key this
/// app has never heard of cannot be dropped by a save.
#[tauri::command]
pub async fn settings_patch(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
    patch: serde_json::Value,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::settings_scope::patch(
        &claude_dir,
        scope,
        wd(working_dir).as_deref(),
        &patch,
        expected_mtime_ms,
    )
}

#[tauri::command]
pub async fn settings_effective(
    state: State<'_, AppState>,
    working_dir: Option<String>,
) -> Result<Vec<EffectiveEntry>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::settings_scope::effective(&claude_dir, wd(working_dir).as_deref())
}
