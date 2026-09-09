use std::path::PathBuf;

use app_core::types::{SettingsScope, StatusLine, StatusLinePreview};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn statusline_get(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
) -> Result<StatusLine, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::statusline::get(&claude_dir, scope, working_dir.map(PathBuf::from).as_deref())
}

#[tauri::command]
pub async fn statusline_put(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
    status_line: StatusLine,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::statusline::put(
        &claude_dir,
        scope,
        working_dir.map(PathBuf::from).as_deref(),
        &status_line,
        expected_mtime_ms,
    )
}

#[tauri::command]
pub async fn statusline_delete(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::statusline::delete(
        &claude_dir,
        scope,
        working_dir.map(PathBuf::from).as_deref(),
        expected_mtime_ms,
    )
}

/// Runs the configured command once. Only ever invoked from an explicit
/// "Test run" button — this app never executes a status line on its own.
#[tauri::command]
pub async fn statusline_preview(
    state: State<'_, AppState>,
    status_line: StatusLine,
    working_dir: Option<String>,
) -> Result<StatusLinePreview, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app_core::statusline::preview(
            &claude_dir,
            working_dir.map(PathBuf::from).as_deref(),
            &status_line,
        )
    })
    .await
    .map_err(|e| AppError::internal(format!("status line preview panicked: {e}")))?
}
