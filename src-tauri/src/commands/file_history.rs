use std::path::PathBuf;

use app_core::types::{Checkpoint, DiffResult, DiffSide};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn file_history_checkpoints(
    state: State<'_, AppState>,
    project_name: String,
    session_id: String,
) -> Result<Vec<Checkpoint>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::file_history::list(&claude_dir, &project_name, &session_id)
}

#[tauri::command]
pub async fn file_history_diff(
    state: State<'_, AppState>,
    session_id: String,
    left: DiffSide,
    right: DiffSide,
) -> Result<DiffResult, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::file_history::diff(&claude_dir, &session_id, &left, &right)
}

#[tauri::command]
pub async fn file_history_blob(
    state: State<'_, AppState>,
    session_id: String,
    backup_file_name: String,
) -> Result<String, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::file_history::read_blob(&claude_dir, &session_id, &backup_file_name)
}

/// Overwrites a real file on disk. The core layer refuses any destination
/// other than the path the version was taken from; the UI confirms first.
#[tauri::command]
pub async fn file_history_restore(
    state: State<'_, AppState>,
    project_name: String,
    session_id: String,
    backup_file_name: String,
    dest: String,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::file_history::restore(
        &claude_dir,
        &project_name,
        &session_id,
        &backup_file_name,
        &PathBuf::from(dest),
    )
}
