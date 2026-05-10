use app_core::types::Skill;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn skills_list(state: State<'_, AppState>) -> Result<Vec<Skill>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::list(&claude_dir)
}

#[tauri::command]
pub async fn skills_get(state: State<'_, AppState>, slug: String) -> Result<Skill, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::skills::get(&claude_dir, &slug)
}
