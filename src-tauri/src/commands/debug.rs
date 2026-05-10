use app_core::claude_cli::ClaudeCliInfo;
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn debug_claude_cli(state: State<'_, AppState>) -> Result<Option<ClaudeCliInfo>, AppError> {
    Ok(state.claude_cli.read().await.clone())
}

#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
