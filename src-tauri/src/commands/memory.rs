use std::path::PathBuf;

use app_core::types::{MemoryDoc, MemoryFile, MemoryPreview};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn memory_list(
    state: State<'_, AppState>,
    working_dir: Option<String>,
) -> Result<Vec<MemoryFile>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::memory::list(&claude_dir, working_dir.map(PathBuf::from).as_deref())
}

#[tauri::command]
pub async fn memory_agent_list(
    state: State<'_, AppState>,
    working_dir: Option<String>,
) -> Result<Vec<MemoryFile>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::memory::agent_memory_list(&claude_dir, working_dir.map(PathBuf::from).as_deref())
}

#[tauri::command]
pub async fn memory_get(
    state: State<'_, AppState>,
    id: String,
    working_dir: Option<String>,
) -> Result<MemoryDoc, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::memory::get(&claude_dir, working_dir.map(PathBuf::from).as_deref(), &id)
}

#[tauri::command]
pub async fn memory_put(
    state: State<'_, AppState>,
    id: String,
    working_dir: Option<String>,
    content: String,
    expected_mtime_ms: Option<i64>,
) -> Result<MemoryDoc, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::memory::put(
        &claude_dir,
        working_dir.map(PathBuf::from).as_deref(),
        &id,
        &content,
        expected_mtime_ms,
    )
}

#[tauri::command]
pub async fn memory_delete(
    state: State<'_, AppState>,
    id: String,
    working_dir: Option<String>,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::memory::delete(&claude_dir, working_dir.map(PathBuf::from).as_deref(), &id)
}

#[tauri::command]
pub async fn memory_preview(
    state: State<'_, AppState>,
    id: String,
    working_dir: Option<String>,
) -> Result<MemoryPreview, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::memory::preview(&claude_dir, working_dir.map(PathBuf::from).as_deref(), &id)
}
