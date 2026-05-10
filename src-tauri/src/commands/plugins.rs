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
