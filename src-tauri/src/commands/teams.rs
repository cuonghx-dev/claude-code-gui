use app_core::types::Team;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn teams_list(state: State<'_, AppState>) -> Result<Vec<Team>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::teams::list(&claude_dir)
}

#[tauri::command]
pub async fn teams_get(state: State<'_, AppState>, id: String) -> Result<Team, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::teams::get(&claude_dir, &id)
}
