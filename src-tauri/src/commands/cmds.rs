use app_core::types::{Command, CommandInput};
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

#[tauri::command]
pub async fn commands_create(
    state: State<'_, AppState>,
    input: CommandInput,
) -> Result<Command, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::commands::create(&claude_dir, input)
}

#[tauri::command]
pub async fn commands_update(
    state: State<'_, AppState>,
    slug: String,
    input: CommandInput,
) -> Result<Command, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::commands::update(&claude_dir, &slug, input)
}

#[tauri::command]
pub async fn commands_delete(state: State<'_, AppState>, slug: String) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::commands::delete(&claude_dir, &slug)
}
