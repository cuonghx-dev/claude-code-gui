use std::path::PathBuf;

use app_core::types::{HookGroup, HookInput, SettingsScope};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn hooks_list(
    state: State<'_, AppState>,
    working_dir: Option<String>,
) -> Result<Vec<HookGroup>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::hooks::list(&claude_dir, working_dir.map(PathBuf::from).as_deref())
}

#[tauri::command]
pub async fn hooks_get(
    state: State<'_, AppState>,
    id: String,
    working_dir: Option<String>,
) -> Result<HookGroup, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::hooks::get(&claude_dir, working_dir.map(PathBuf::from).as_deref(), &id)
}

#[tauri::command]
pub async fn hooks_create(
    state: State<'_, AppState>,
    input: HookInput,
) -> Result<HookGroup, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::hooks::create(&claude_dir, &input)
}

#[tauri::command]
pub async fn hooks_update(
    state: State<'_, AppState>,
    id: String,
    input: HookInput,
) -> Result<HookGroup, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::hooks::update(&claude_dir, &id, &input)
}

#[tauri::command]
pub async fn hooks_delete(
    state: State<'_, AppState>,
    id: String,
    working_dir: Option<String>,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::hooks::delete(&claude_dir, working_dir.map(PathBuf::from).as_deref(), &id)
}

#[tauri::command]
pub async fn hooks_raw_get(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
) -> Result<String, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::hooks::raw_get(&claude_dir, scope, working_dir.map(PathBuf::from).as_deref())
}

#[tauri::command]
pub async fn hooks_raw_put(
    state: State<'_, AppState>,
    scope: SettingsScope,
    working_dir: Option<String>,
    content: String,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::hooks::raw_put(
        &claude_dir,
        scope,
        working_dir.map(PathBuf::from).as_deref(),
        &content,
        expected_mtime_ms,
    )
}
