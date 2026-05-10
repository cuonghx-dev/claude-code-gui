use std::collections::HashMap;

use app_core::types::Agent;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn agents_list(state: State<'_, AppState>) -> Result<Vec<Agent>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::list(&claude_dir)
}

#[tauri::command]
pub async fn agents_get(state: State<'_, AppState>, slug: String) -> Result<Agent, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::get(&claude_dir, &slug)
}

#[tauri::command]
pub async fn agents_skill_counts(state: State<'_, AppState>) -> Result<HashMap<String, usize>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::agents::skill_counts(&claude_dir)
}
