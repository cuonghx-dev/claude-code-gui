use app_core::types::Settings;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<Settings, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::settings::get(&claude_dir)
}
