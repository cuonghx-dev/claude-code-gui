use app_core::types::{Plugin, PluginDetail};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn plugins_list(state: State<'_, AppState>) -> Result<Vec<Plugin>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plugins::list(&claude_dir)
}

#[tauri::command]
pub async fn plugins_get(state: State<'_, AppState>, id: String) -> Result<PluginDetail, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plugins::get(&claude_dir, &id)
}

#[tauri::command]
pub async fn plugins_delete(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plugins::delete(&claude_dir, &id)
}

#[tauri::command]
pub async fn plugins_set_enabled(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plugins::set_enabled(&claude_dir, &id, enabled)
}

#[tauri::command]
pub async fn plugins_update_skills(
    state: State<'_, AppState>,
    id: String,
    slugs: Vec<String>,
) -> Result<(), AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::plugins::update_skills(&claude_dir, &id, slugs)
}
