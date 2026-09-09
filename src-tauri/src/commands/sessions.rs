use app_core::types::{Message, Page, SessionSummary, Thread};
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
    // The shared index makes a deep page a seek instead of a full re-read.
    app_core::sessions::messages(
        &claude_dir,
        &state.transcript_index,
        &project_name,
        &session_id,
        after_index,
        limit,
    )
}

#[tauri::command]
pub async fn sessions_threads(
    state: State<'_, AppState>,
    project_name: String,
    session_id: String,
) -> Result<Vec<Thread>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::sessions::threads(&claude_dir, &project_name, &session_id)
}

#[tauri::command]
pub async fn sessions_thread_messages(
    state: State<'_, AppState>,
    project_name: String,
    session_id: String,
    thread_id: String,
    after_index: Option<usize>,
    limit: Option<usize>,
) -> Result<Page<Message>, AppError> {
    let claude_dir = state.claude_dir.read().await.clone();
    app_core::sessions::thread_messages(
        &claude_dir,
        &state.transcript_index,
        &project_name,
        &session_id,
        &thread_id,
        after_index,
        limit,
    )
}
