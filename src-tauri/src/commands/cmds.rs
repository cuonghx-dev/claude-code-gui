// Module name `cmds` to avoid clash with `tauri::command` macro hygiene.

use app_core::types::Command;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn commands_list(state: State<'_, AppState>) -> Result<Vec<Command>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::commands::list(&claude_dir)
}

#[tauri::command]
pub async fn commands_get(state: State<'_, AppState>, slug: String) -> Result<Command, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::commands::get(&claude_dir, &slug)
}
