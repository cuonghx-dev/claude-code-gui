use app_core::types::{Message, Page, SessionSummary};
use app_core::AppError;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn sessions_list_for_project(
    state: State<'_, AppState>,
    name: String,
) -> Result<Vec<SessionSummary>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::sessions::list_for_project(&claude_dir, &name)
}

#[tauri::command]
pub async fn sessions_messages(
    state: State<'_, AppState>,
    project_name: String,
    session_id: String,
    after_index: Option<usize>,
    limit: Option<usize>,
) -> Result<Page<Message>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::sessions::messages(&claude_dir, &project_name, &session_id, after_index, limit)
}
