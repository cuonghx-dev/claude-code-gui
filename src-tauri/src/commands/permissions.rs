use std::path::PathBuf;

use app_core::types::{EffectivePermissions, Permissions, RuleIssue, SettingsScope};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn permissions_get(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
) -> Result<Permissions, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::permissions::get(&claude_dir, scope, working_dir.map(PathBuf::from).as_deref())
}

#[tauri::command]
pub async fn permissions_put(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
    permissions: Permissions,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::permissions::put(
        &claude_dir,
        scope,
        working_dir.map(PathBuf::from).as_deref(),
        &permissions,
        expected_mtime_ms,
    )
}

#[tauri::command]
pub async fn permissions_effective(
    state: State<'_, AppState>,
    working_dir: Option<String>,
) -> Result<EffectivePermissions, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::permissions::effective(&claude_dir, working_dir.map(PathBuf::from).as_deref())
}

#[tauri::command]
pub async fn permissions_validate(permissions: Permissions) -> Result<Vec<RuleIssue>, AppError> {
    Ok(app_core::permissions::validate(&permissions))
}
