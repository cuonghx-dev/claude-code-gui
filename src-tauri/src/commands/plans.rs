use app_core::types::Plan;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn plans_list(state: State<'_, AppState>) -> Result<Vec<Plan>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plans::list(&claude_dir)
}

#[tauri::command]
pub async fn plans_get(state: State<'_, AppState>, slug: String) -> Result<Plan, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plans::get(&claude_dir, &slug)
}
