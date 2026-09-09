use app_core::types::{ActivityDay, IndexStats, UsageQuery, UsageReport};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

/// A cold scan of every transcript takes seconds, so the work runs on the
/// blocking pool rather than stalling the webview.
#[tauri::command]
pub async fn usage_refresh(state: State<'_, AppState>) -> Result<IndexStats, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let cache_dir = state.cache_dir.as_ref().clone();
    tauri::async_runtime::spawn_blocking(move || app_core::usage::refresh_index(&claude_dir, &cache_dir))
        .await
        .map_err(|e| AppError::internal(format!("usage refresh panicked: {e}")))?
}

#[tauri::command]
pub async fn usage_rollup(
    state: State<'_, AppState>,
    query: UsageQuery,
) -> Result<UsageReport, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    let cache_dir = state.cache_dir.as_ref().clone();
    tauri::async_runtime::spawn_blocking(move || {
        app_core::usage::rollup(&claude_dir, &cache_dir, &query)
    })
    .await
    .map_err(|e| AppError::internal(format!("usage rollup panicked: {e}")))?
}

#[tauri::command]
pub async fn usage_activity(
    state: State<'_, AppState>,
    days: Option<u32>,
) -> Result<Vec<ActivityDay>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    tauri::async_runtime::spawn_blocking(move || {
        app_core::usage::activity(&claude_dir, days.unwrap_or(90))
    })
    .await
    .map_err(|e| AppError::internal(format!("usage activity panicked: {e}")))?
}
