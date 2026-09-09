use app_core::types::{CliHistoryDetail, CliHistoryEntry};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn cli_history_list(state: State<'_, AppState>) -> Result<Vec<CliHistoryEntry>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::cli_history::list(&claude_dir)
}

#[tauri::command]
pub async fn cli_history_get(
    state: State<'_, AppState>,
    id: String,
) -> Result<CliHistoryDetail, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::cli_history::get(&claude_dir, &id)
}
